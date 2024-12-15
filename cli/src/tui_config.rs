use commity_lib::utils::init_data_dir;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::fs;

/// Represents the symbol configuration in `TUIConfig` used in the TUI (Text User Interface).
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TuiSymbols {
    None,
    Unicode,
    NerdFont,
}

/// Represents the layout Variant in in `TUIConfig` used for the TUI.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum TUILayout {
    Inline,
    Fullscreen,
    FullscreenCenter,
}

/// Represents the configuration settings for the TUI.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TUIConfig {
    pub symbols: TuiSymbols,
    pub layout: TUILayout,
}

impl Default for TUIConfig {
    /// Provides the default configuration for the TUI.
    ///
    /// - Symbols: `Unicode`
    /// - Layout: `Inline`
    fn default() -> Self {
        TUIConfig {
            symbols: TuiSymbols::Unicode,
            layout: TUILayout::Inline,
        }
    }
}

impl TUIConfig {
    /// Loads the TUI configuration from a YAML file.
    ///
    /// If the configuration file does not exist, it falls back to the default configuration.
    ///
    /// # Errors
    /// Returns an error if the configuration file cannot be read or parsed.
    ///
    /// # Returns
    /// A `TUIConfig` instance with the loaded or default configuration.
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
