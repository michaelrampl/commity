use git2::Repository;
use minijinja::{value::Value, Environment};
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

//-----------------------------------------------------------------------------
// Field

#[derive(Debug, Serialize, Deserialize)]
pub enum Field {
    Text(String),
    Choice(String, String),
    Boolean(bool),
}

//-----------------------------------------------------------------------------
// FieldValue

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FieldValue {
    Text(String),
    Boolean(bool),
}

//-----------------------------------------------------------------------------
// Entry

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    Text(EntryText),
    Choice(EntryChoice),
    Boolean(EntryBoolean),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum EntryMode {
    Required,
    Optional,
    Hidden,
}

//-----------------------------------------------------------------------------
// Section

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub title: String,
    pub entries: Vec<Entry>,
}

//-----------------------------------------------------------------------------
// Entry Text

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryText {
    pub mode: EntryMode,
    pub name: String,
    pub label: String,
    pub description: String,
    pub min_length: i64,
    pub max_length: i64,
    pub multi_line: bool,
    pub default: String,
}

//-----------------------------------------------------------------------------
// Entry Choice

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryChoice {
    pub mode: EntryMode,
    pub name: String,
    pub label: String,
    pub description: String,
    pub choices: Vec<Choice>,
    pub default: String,
}

//-----------------------------------------------------------------------------
// Entry Boolean

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryBoolean {
    pub mode: EntryMode,
    pub name: String,
    pub label: String,
    pub description: String,
    pub default: bool,
}

//-----------------------------------------------------------------------------
// Config

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub sections: Vec<Section>,
    pub template: String,
}

impl Config {
    pub fn new(current_dir: &Path) -> Result<Self, Box<dyn Error>> {
        let repo_dir = find_git_repository(current_dir)?;

        let config_path_repo = repo_dir.join(".commity.yaml");
        let config_path_home = dirs::home_dir()
            .ok_or("Unable to determine home directory")?
            .join(".commity.yaml");

        let config_path = if config_path_repo.exists() {
            config_path_repo
        } else if config_path_home.exists() {
            config_path_home
        } else {
            return Err(format!(
                "No config file found in [{}] or [{}]",
                config_path_repo.display(),
                config_path_home.display()
            )
            .into());
        };

        load_config(&config_path)
    }
}

fn load_config(path: &Path) -> Result<Config, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let config: Config = serde_yml::from_reader(file)?;
    Ok(config)
}

fn find_git_repository(start_dir: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let mut dir = start_dir.to_path_buf();

    loop {
        if dir.join(".git").exists() || Repository::open(&dir).is_ok() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }

    Err("No Git repository found".into())
}

//
// Rendering

pub fn render_fields(
    data: HashMap<String, FieldValue>,
    template: &String,
) -> Result<String, Box<dyn Error>> {
    // Convert the HashMap into a MiniJinja value using `Value::from_serializable`
    let data: Value = Value::from_serialize(&data);

    // Create a MiniJinja environment
    let mut env = Environment::new();

    // Add the template to the environment
    env.add_template("template", template)?;

    // Render the template
    let tmpl = env.get_template("template")?;
    let rendered = tmpl.render(data)?;

    Ok(rendered)
}
