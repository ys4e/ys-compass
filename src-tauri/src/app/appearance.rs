use std::fs;
use lazy_static::lazy_static;
use tauri::{AppHandle, Manager};
use log::debug;
use regex::Regex;
use serde_json::Value;

lazy_static! {
    static ref FILE_NAME_REGEX: Regex = Regex::new(r"https:\/\/.*\/(.*_.*\.webp)").unwrap();
}

/// This is the default background/splashscreen used by the launcher.
///
/// It is a fallback that is pre-bundled with the original launcher.
///
/// Found from `https://genshin-impact.fandom.com/wiki/Version/Gallery`.
const DEFAULT_SPLASH: &[u8] = include_bytes!("../../../resources/assets/default_splash.webp");

/// Saves the default splash screen to the cache directory.
///
/// This only occurs if the default splash screen is not already present.
#[tauri::command]
pub fn appearance__default_splash(app_handle: AppHandle) -> Result<String, &'static str> {
    // Resolve the cache directory.
    let Ok(app_data_dir) = app_handle.path().app_data_dir() else {
        return Err("Failed to resolve app data directory.");
    };
    let cache_dir = app_data_dir.join("cache");

    // Write the default splash screen to the cache directory.
    let file = cache_dir.join("default.webp");
    if !file.exists() {
        debug!("Wrote default splash screen to cache directory.");
        fs::write(&file, DEFAULT_SPLASH).unwrap();
    }

    // Return the asset path to the default splash screen.
    match file.to_str() {
        Some(path) => Ok(path.to_string()),
        None => Err("Failed to resolve default splash screen path."),
    }
}

/// Fetches the background for the launcher.
///
/// This will load it from the cache, if applicable.
#[tauri::command]
pub async fn appearance__background(app_handle: AppHandle) -> Result<String, &'static str> {
    // Resolve the cache directory.
    let Ok(app_data_dir) = app_handle.path().app_data_dir() else {
        return Err("Failed to resolve app data directory.");
    };
    let cache_dir = app_data_dir.join("cache");

    // Fetch the basic game information from the API.
    let Ok(response) = reqwest::get(dotenv!("BASIC_GAME_INFO_URL")).await else {
        return Err("Failed to perform request for basic game information.");
    };

    // If the request failed, try to save/use the default background.
    if !response.status().is_success() {
        return appearance__default_splash(app_handle);
    }

    // Get the background URL from the response.
    // Step 1. Parse the response as JSON data.
    let Ok(text) = response.text().await else {
        return Err("Failed to read response text.");
    };
    let Ok(response) = serde_json::from_str::<Value>(&text) else {
        return Err("Failed to parse response as JSON data.");
    };

    // Step 2. Extract the background URL from the JSON data.
    let data = &response["data"];
    let Some(data) = data["game_info_list"].as_array() else {
        return Err("Failed to extract game information list.");
    };

    let Some(game) = data.iter()
        .find(|v| &v["game"]["id"] == dotenv!("GAME_ID"))
    else {
        return Err("Failed to find game information.");
    };

    let Some(backgrounds) = game["backgrounds"].as_array() else {
        return Err("Failed to extract backgrounds.");
    };
    let Some(url) = backgrounds[0]["background"]["url"].as_str() else {
        return Err("Failed to extract background URL.");
    };
    
    // Step 3. Extract the file name from the URL & query for data.
    let file_name = FILE_NAME_REGEX.captures(url)
        .and_then(|c| c.get(1))
        .map(|m| m.as_str())
        .unwrap_or("background.webp");
    let file = cache_dir.join(file_name);

    // If the file doesn't exist, download it and save it.
    if !file.exists() {
        let Ok(response) = reqwest::get(url).await else {
            return Err("Failed to perform request for background image.");
        };
        let Ok(bytes) = response.bytes().await else {
            return Err("Failed to read response bytes.");
        };

        fs::write(&file, bytes).unwrap();
    }
    
    match file.to_str() {
        Some(path) => Ok(path.to_string()),
        None => Err("Failed to resolve background path."),
    }
}
