use std::path::PathBuf;
use tauri::Context;
use anyhow::{anyhow, Result};
use sys_locale::get_locale;

/// This is the identifier of the app.
/// 
/// This is used in path resolving.
static APP_IDENTIFIER: &str = "ys-compass";

/// Creates the context for the Tauri app.
///
/// This method is isolated as to not slow down IDE performance in `main.rs`.
pub fn build_context() -> Context {
    tauri::generate_context!()
}

/// Returns the system's locale.
/// 
/// Defaults to English if it was unable to be detected.
pub fn system_locale() -> String {
    get_locale().unwrap_or_else(|| String::from("en-us"))
}

/// Returns a path to the application's configuration directory.
pub fn app_config_dir() -> Result<PathBuf> {
    dirs::config_dir()
        .ok_or(anyhow!("unknown path"))
        .map(|dir| dir.join(APP_IDENTIFIER))
}

/// Returns a path to the application's data directory.
pub fn app_data_dir() -> Result<PathBuf> {
    dirs::data_dir()
        .ok_or(anyhow!("unknown path"))
        .map(|dir| dir.join(APP_IDENTIFIER))
}
