use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::Context;
use anyhow::{anyhow, Result};
use base64::Engine;
use base64::prelude::BASE64_STANDARD;
use sys_locale::get_locale;
use ys_sniffer::PacketSource;

/// This type is used for Tauri commands.
///
/// It might return an error message if something went wrong.
pub type MaybeError<T> = std::result::Result<T, &'static str>;

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

/// Base64 encodes binary data into a standard Base64 string.
pub fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

/// Base64 decodes a standard Base64 string into binary data.
pub fn base64_decode(string: String) -> Result<Vec<u8>> {
    BASE64_STANDARD.decode(string.as_bytes())
        .map_err(|e| anyhow!(e))
}

/// Returns the system's locale.
///
/// Defaults to English if it was unable to be detected.
pub fn system_locale() -> String {
    get_locale().unwrap_or_else(|| String::from("en-us"))
}

/// Returns the current UNIX timestamp.
pub fn unix_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time went backwards")
        .as_secs()
}

/// Writes the text content to the file at the given path.
pub fn write_file<S: AsRef<str>>(path: &PathBuf, data: S) -> Result<()> {
    std::fs::write(path, data.as_ref())
        .map_err(|e| anyhow!(e))
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

/// Returns the opposite of the given packet source.
pub fn opposite(source: PacketSource) -> PacketSource {
    match source {
        PacketSource::Client => PacketSource::Server,
        PacketSource::Server => PacketSource::Client,
    }
}

pub mod serde_base64 {
    use serde::{Serializer, Deserializer, Serialize, Deserialize};
    use crate::utils;

    pub fn serialize<S: Serializer>(v: &Vec<u8>, s: S) -> Result<S::Ok, S::Error> {
        String::serialize(&utils::base64_encode(v), s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        utils::base64_decode(String::deserialize(d)?)
            .map_err(serde::de::Error::custom)
    }
}
