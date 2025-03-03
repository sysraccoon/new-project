use super::*;
use diffdir::diffcmp::DirCmp;
use test_dir::{DirBuilder, TestDir};

#[test]
fn test_basic() {
    test_template("basic");
}

#[test]
fn test_exclude() {
    test_template("exclude");
}

#[test]
fn test_context_template() {
    test_template("context-template");
}

fn test_template(name: &str) {
    let basic_suite_path = Path::new("tests").join(name);
    let expected_dir = basic_suite_path.join("dest");
    let template_dir = basic_suite_path.join("src");

    let test_dir_temp = TestDir::temp();
    let project_dir = test_dir_temp.path(".");

    let mut context = template_context(&project_dir);

    let context_override_path = basic_suite_path.join("context.yaml");
    if context_override_path.is_file() {
        let context_override_file = File::open(context_override_path).unwrap();
        let context_override: HashMap<String, String> =
            serde_yml::from_reader(context_override_file).unwrap();
        context.extend(context_override);
    }

    initialize_project_with_context(template_dir, project_dir.clone(), context).unwrap();

    let dir_cmp = DirCmp::new(&expected_dir, &project_dir, &None).compare_directories();
    let diff_text: String = dir_cmp.format_text(true).join("\n");

    if dir_cmp.are_different() {
        for diff_entry in dir_cmp.differs {
            let expected_content = fs::read_to_string(expected_dir.join(&diff_entry)).unwrap();
            let actual_content = fs::read_to_string(project_dir.join(&diff_entry)).unwrap();

            assert_eq!(expected_content, actual_content, "{}", diff_text);
        }

        assert!(false, "difference: {}", diff_text);
    }
}

