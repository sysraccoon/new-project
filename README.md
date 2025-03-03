# new-project

CLI utility for creating new projects from templates

## Usage

### Basic

Most trivial variant is directly copy all content of template directory.
As example, some basic project that can be used as template for future project:

```bash
templates/basic
├── first.txt
└── second.tx
```

Create copy of it is easy as this:

```bash
mkdir my-project
cd my-project
new-project ~/templates/basic
```

After that it create copy aff all files:

```bash
my-project
├── first.txt
└── second.tx
```

### Templating

The basic option is no better than using cp. In most cases, new projects require minor
substitutions in the template files. These substitutions may depend on the environment
in which the utility is running or on user input. Let's look at both options.

For template preprocessing, you need add `.new-project.yaml` to template directory:

```bash
templates/context
├── .new-project.yml
└── context-info.json
```

`.new-project.yaml` content:

```yaml
templates:
  - context-info.json
```

This describes that instead of direct copying `content-info.json`, preprocessing with minijinja should be used.
`content-info.json` look like this:

```json
{
  "project_directory_name": "{{ context.project_directory_name }}",
  "project_directory_path": "{{ context.project_directory_path }}",
  "current_date": "{{ context.current_date }}",
  "current_time": "{{ context.current_time }}",
  "git_user_name": "{{ context.git_user_name }}",
  "git_user_email": "{{ context.git_user_email }}"
}
```

Creating new project is the same as basic variant:

```bash
mkdir my-project
cd my-project
new-project ~/templates/context
```

As result `my-project` contains single file `context-info.json` (`.new-project.yaml` automatically ignored):

```bash
my-project
└── context-info.json
```

`context-info.json` content after preprocessing:

```json
{
  "project_directory_name": "my-project",
  "project_directory_path": "/home/test/projects/my-project",
  "current_date": "2025-03-03",
  "current_time": "15:00",
  "git_user_name": "test",
  "git_user_email": "test@example.com"
}
```

Let's look on some other example with user defined parameters:

```bash
templates/parameters
├── .new-project.yml
└── custom-parameters.json
```

`.new-project.yaml` contains additional `parameters` field:

```yaml
templates:
  - custom-parameters.json

parameters:
  project_name:
    description: "Project Name"
    default: "{{ context.project_directory_name }}"
  version:
    description: "Project Version"
    default: "0.1.0"
```

And `custom-parameters` use it simple as is:
```json
{
  "project_name": "{{ project_name }}",
  "version": "{{ version }}"
}
```

Next project initialization ask to user specific parameters:

```bash
mkdir my-project
cd my-project
new-project ~/templates/parameters
# Project Name (default my-project): my-project
# Version (default 0.1.0): 0.1.0
```

After that `custom-parameters` contains user inputs:
```json
{
  "project_name": "my-project",
  "version": "0.1.0"
}
```

## Installation

**nix**

```sh
nix profile install github:sysraccoon/new-project
```

**cargo**

```sh
# make sure you have pkg-config
cargo install --git https://github.com/sysraccoon/new-project.git
```

