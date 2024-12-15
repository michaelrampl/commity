use std::fs;
use std::path::PathBuf;

/// Gets the application data directory path.
///
/// The function constructs the path to the local data directory for the application.
///
/// # Errors
///
/// Returns an error if the local data directory cannot be determined.
///
/// # Returns
///
/// A `PathBuf` pointing to the application's data directory.
pub fn get_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    Ok(dirs::data_local_dir()
        .ok_or("Unable to determine the data directory")?
        .join("commity"))
}

/// Initializes the application data directory.
///
/// Creates the directory if it does not already exist.
///
/// # Errors
///
/// Returns an error if the directory cannot be created or if the local data directory cannot be determined.
///
/// # Returns
///
/// A `PathBuf` pointing to the application's data directory.
pub fn init_data_dir() -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = get_data_dir()?;

    // Check if the directory exists, and create it if necessary
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)
            .map_err(|err| format!("Failed to create config directory: {}", err))?;
    }

    Ok(app_data_dir)
}
