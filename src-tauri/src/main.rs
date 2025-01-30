#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![allow(non_snake_case)]

#[macro_use]
extern crate dotenv_codegen;

use std::fs;
use anyhow::Result;
use tauri::{generate_handler, AppHandle, Manager};

mod window;
mod utils;
mod app;

use crate::app::appearance;

/// Prepares the application for use.
fn setup_app(app_handle: &AppHandle) -> Result<()> {
    // Check if the data directory exists.
    let app_data_dir = app_handle.path().app_data_dir()?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir(app_data_dir.join("cache"))?;
    }

    Ok(())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(generate_handler![
            window::window__close,
            appearance::appearance__background,
            appearance::appearance__default_splash
        ])
        .setup(|app| {
            setup_app(app.handle())?;

            Ok(())
        })
        // warning! this method slows down code intellisense...
        // as in like a lot...
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
