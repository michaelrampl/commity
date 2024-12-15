use git2::Repository;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use crate::utils::get_data_dir;

/// Represents a field entry in a configuration section.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Entry {
    Text(EntryText),
    Choice(EntryChoice),
    Boolean(EntryBoolean),
}

/// Represents a section in the `Configuration`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Section {
    pub title: String,
    pub entries: Vec<Entry>,
}

/// Represents a Text Entry (variant of the `Entry` enum) as part of the `Section`.
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

/// Represents a Choice Option in a `EntryChoice`.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Choice {
    pub value: String,
    pub label: String,
}

/// Represents a Choice Entry (variant of the `Entry` enum) as part of the `Section`.
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

/// Represents a Boolean Entry (variant of the `Entry` enum) as part of the `Section`.
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

/// Represents the commit configuration, including sections and a rendering template.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Configuration {
    pub sections: Vec<Section>,
    pub template: String,
}

impl Configuration {
    /// Loads the configuration file from the local or global `.commity.yaml` file.
    ///
    /// The function searches for the configuration file in the current Git repository
    /// or in the application data directory.
    ///
    /// # Arguments
    ///
    /// * `directory` - The current directory to start searching for the configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if no configuration file is found or if the file cannot be read.
    ///
    /// # Returns
    ///
    /// A `Configuration` instance with initialized values.

    pub fn load(directory: &Path) -> Result<Self, Box<dyn Error>> {
        let app_data_dir = get_data_dir()?;

        let repo_dir = Configuration::find_git_repository(directory)?;

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

        Configuration::load_config(&config_path)
    }

    /// Initializes default values for all configuration entries.
    ///
    /// Populates the `value` fields of entries with their respective default values.
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

    /// Loads a configuration file from the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - A path to the configuration file.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    ///
    /// # Returns
    ///
    /// A `Configuration` instance with initialized values.
    fn load_config(path: &Path) -> Result<Configuration, Box<dyn Error>> {
        let file = fs::File::open(path)?;
        let mut config: Configuration = serde_yml::from_reader(file)?;
        config.initialize_values();
        Ok(config)
    }

    /// Finds the nearest Git repository starting from a given directory.
    ///
    /// The function searches upwards in the directory hierarchy to find the `.git` folder or
    /// a valid Git repository.
    ///
    /// # Arguments
    ///
    /// * `start_dir` - The directory to start searching from.
    ///
    /// # Errors
    ///
    /// Returns an error if no Git repository is found.
    ///
    /// # Returns
    ///
    /// A `PathBuf` pointing to the Git repository directory.

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
}
