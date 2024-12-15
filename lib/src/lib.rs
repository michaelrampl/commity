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
    pub name: String,
    pub label: String,
    pub description: String,
    pub min_length: i64,
    pub max_length: i64,
    pub multi_line: bool,
    pub default: String,
    #[serde(skip_deserializing)]
    pub value: String,
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
    pub name: String,
    pub label: String,
    pub description: String,
    pub choices: Vec<Choice>,
    pub default: String,
    #[serde(skip_deserializing)]
    pub value: String,
}

//-----------------------------------------------------------------------------
// Entry Boolean

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EntryBoolean {
    pub name: String,
    pub label: String,
    pub description: String,
    pub default: bool,
    #[serde(skip_deserializing)]
    pub value: bool,
}

//-----------------------------------------------------------------------------
// Initialize data dir

fn init_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    // Get the data directory and append "commity"
    let app_data_dir = dirs::data_local_dir()
        .ok_or("Unable to determine the data directory")?
        .join("commity");

    // Check if the directory exists, and create it if necessary
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|err| format!("Failed to create config directory: {}", err))?;
    }

    Ok(app_data_dir)
}

//-----------------------------------------------------------------------------
// App Config

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TuiSymbols {
    None,
    Unicode,
    NerdFont,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TUILayout {
    Inline,
    Fullscreen,
    FullscreenCenter,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TUIConfig {
    pub symbols: TuiSymbols,
    pub layout: TUILayout,
}

impl Default for TUIConfig {
    fn default() -> Self {
        TUIConfig {
            symbols: TuiSymbols::NerdFont,
            layout: TUILayout::Inline,
        }
    }
}

impl TUIConfig {
    pub fn load() -> Result<Self, Box<dyn Error>> {
        let app_data_dir = init_data_dir()?;

        let config_file_path = app_data_dir.join("tui_config.yaml");

        if !config_file_path.exists() {
            return Ok(TUIConfig::default());
        }

        let file = fs::File::open(&config_file_path)?;
        let config: TUIConfig = serde_yml::from_reader(file)?;

        Ok(config)
    }
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
    pub fn load(current_dir: &Path) -> Result<Self, Box<dyn Error>> {
        let app_data_dir = init_data_dir()?;

        let repo_dir = find_git_repository(current_dir)?;

        let config_local = repo_dir.join(".commity.yaml");

        let config_global = app_data_dir.join(".commity.yaml");

        let config_path = if config_local.exists() {
            config_local
        } else if config_global.exists() {
            config_global
        } else {
            return Err(format!(
                "No config file found in {} or {}",
                config_local.display(),
                config_global.display()
            )
            .into());
        };

        load_config(&config_path)
    }

    fn initialize_values(&mut self) {
        for section in &mut self.sections {
            for entry in &mut section.entries {
                match entry {
                    Entry::Text(entry_text) => entry_text.value = entry_text.default.clone(),
                    Entry::Choice(entry_choice) => {
                        entry_choice.value = entry_choice.default.clone()
                    }
                    Entry::Boolean(entry_boolean) => {
                        entry_boolean.value = entry_boolean.default.clone()
                    }
                }
            }
        }
    }
}

fn load_config(path: &Path) -> Result<Config, Box<dyn Error>> {
    let file = fs::File::open(path)?;
    let mut config: Config = serde_yml::from_reader(file)?;
    config.initialize_values();
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

//-----------------------------------------------------------------------------
// Rendering

pub fn render_message(
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
