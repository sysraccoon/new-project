#[cfg(test)]
mod tests;

use anyhow::{anyhow, Context};
use chrono::SubsecRound;
use clap::Parser;
use indexmap::IndexMap;
use minijinja::{context, Environment, UndefinedBehavior};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{stdin, stdout, BufRead, Write};
use std::path::{Path, PathBuf};

#[derive(Parser, Debug)]
#[clap(author = "sysraccoon", version, about)]
struct Args {
    #[arg()]
    template_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TemplateConfig {
    #[serde(default)]
    templates: Vec<String>,
    #[serde(default)]
    parameters: IndexMap<String, TemplateParameterInfo>,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
struct TemplateParameterInfo {
    #[serde(default)]
    description: Option<String>,
    #[serde(default)]
    default: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let template_dir = Path::new(&args.template_dir);

    if !template_dir.exists() {
        return Err(anyhow!("Template directory not exist"));
    }

    if !template_dir.is_dir() {
        return Err(anyhow!("Template directory has invalid file type"));
    }

    let current_dir = std::env::current_dir()?;
    let entry_count = fs::read_dir(&current_dir)?.count();
    if entry_count != 0 {
        return Err(anyhow!("Current directory should be empty"));
    }

    initialize_project(template_dir.to_path_buf(), current_dir)?;

    Ok(())
}

fn initialize_project(template_dir: PathBuf, project_dir: PathBuf) -> anyhow::Result<()> {
    let context = template_context(&project_dir);
    initialize_project_with_context(template_dir, project_dir, context)
}

fn initialize_project_with_context(
    template_dir: PathBuf,
    project_dir: PathBuf,
    context: HashMap<String, String>,
) -> anyhow::Result<()> {
    let mut env = Environment::new();
    env.set_undefined_behavior(UndefinedBehavior::Strict);
    env.set_keep_trailing_newline(true);
    env.add_global("context", context);

    let template_config: TemplateConfig = load_template_config(&template_dir)?;

    let mut exclude_pathes: Vec<PathBuf> = template_config
        .exclude
        .iter()
        .map(|f| PathBuf::from(f))
        .collect();
    exclude_pathes.extend_from_slice(&default_template_config_pathes());

    let template_files: Vec<PathBuf> = template_config
        .templates
        .iter()
        .map(|f| PathBuf::from(f))
        .collect();

    let mut user_input = String::new();
    for (name, info) in template_config.parameters {
        let default_value = if let Some(value) = info.default {
            Some(env.render_str(&value, context!())?)
        } else {
            None
        };

        let description = match info.description {
            Some(description) => description,
            None => name.clone(),
        };

        if default_value.is_some() {
            print!(
                "{} (default {}): ",
                description,
                default_value.clone().unwrap()
            );
        } else {
            print!("{}: ", description);
        }

        stdout().flush()?;
        user_input.clear();
        stdin().lock().read_line(&mut user_input)?;

        let trim_input = user_input.trim();
        let parameter_value = match (trim_input, default_value.clone()) {
            ("", None) => return Err(anyhow!("property '{}' is missed", &name)),
            ("", Some(default)) => default,
            (value, _) => value.to_string(),
        };

        env.add_global(name, parameter_value);
    }

    let mut walk_targets: Vec<(PathBuf, PathBuf)> = vec![(template_dir.clone(), project_dir)];

    let mut lazy_create_dirs = Vec::new();
    let mut lazy_copy_files = Vec::new();
    let mut lazy_jinja_templates = Vec::new();

    while let Some((src, dest)) = walk_targets.pop() {
        lazy_create_dirs.push(dest.clone());

        for entry in fs::read_dir(src.clone()).context(format!(
            "try read source directory {}",
            src.clone().display()
        ))? {
            let entry = entry?;
            let dest_path = dest.join(entry.file_name());
            let file_type = entry.file_type()?;
            let relative_path = entry.path().strip_prefix(&template_dir)?.to_path_buf();

            if exclude_pathes.contains(&relative_path) {
                continue;
            }

            if file_type.is_dir() {
                let new_target = (entry.path(), dest_path);
                walk_targets.push(new_target);
                continue;
            }

            if template_files.contains(&relative_path) {
                let content = String::from_utf8(fs::read(entry.path())?).unwrap();
                let template_name = relative_path.to_str().unwrap().to_string();
                env.add_template_owned(template_name.clone(), content)?;
                lazy_jinja_templates.push((template_name, dest_path));
            } else {
                lazy_copy_files.push((entry.path(), dest_path));
            }
        }
    }

    let mut lazy_jinja_files = Vec::new();
    for (template_name, dest) in lazy_jinja_templates {
        let rendered_content = env.get_template(&template_name)?.render(context!())?;
        lazy_jinja_files.push((rendered_content, dest));
    }

    for dir in lazy_create_dirs {
        fs::create_dir_all(dir)?;
    }

    for (src, dest) in lazy_copy_files {
        fs::copy(src, dest)?;
    }

    for (rendered_content, dest) in lazy_jinja_files {
        fs::write(dest, rendered_content)?;
    }

    Ok(())
}

fn template_context(project_dir: &Path) -> HashMap<String, String> {
    let mut result: HashMap<String, String> = HashMap::new();

    let project_dir_name = project_dir
        .file_name()
        .unwrap()
        .to_os_string()
        .into_string()
        .unwrap();
    result.insert(
        String::from("project_directory_name"),
        project_dir_name.to_string(),
    );

    let project_dir_path = project_dir.to_str().unwrap();
    result.insert(
        String::from("project_directory_path"),
        project_dir_path.to_string(),
    );

    let now = chrono::Utc::now();
    result.insert(
        String::from("current_date"),
        now.format("%Y-%m-%d").to_string(),
    );
    result.insert(
        String::from("current_time"),
        now.time().round_subsecs(0).to_string(),
    );

    if let Ok(git_config) = git2::Config::open_default() {
        if let Ok(git_user_name) = git_config.get_string("user.name") {
            result.insert(String::from("git_user_name"), git_user_name);
        }

        if let Ok(git_user_email) = git_config.get_string("user.email") {
            result.insert(String::from("git_user_email"), git_user_email);
        }
    }

    result
}

fn load_template_config(template_dir: &Path) -> anyhow::Result<TemplateConfig> {
    let possible_config_pathes: Vec<PathBuf> = default_template_config_pathes()
        .iter()
        .map(|p| template_dir.join(p))
        .collect();

    for config_path in possible_config_pathes {
        if config_path.is_file() {
            let template_config_file = File::open(config_path)?;
            return Ok(serde_yml::from_reader(template_config_file)?);
        }
    }

    Ok(TemplateConfig::default())
}

fn default_template_config_pathes() -> Vec<PathBuf> {
    vec![
        PathBuf::from(".new-project.yaml"),
        PathBuf::from(".new-project.yml"),
        PathBuf::from(".new-project"),
    ]
}
