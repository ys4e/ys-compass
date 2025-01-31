#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![allow(non_snake_case)]

#[macro_use]
extern crate dotenv_codegen;

use std::fs;
use anyhow::Result;
use tauri::{generate_handler, AppHandle, Manager};
use tauri_plugin_log::TimezoneStrategy;

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

// noinspection RsUnnecessaryQualifications
fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .format(|consumer, message, record| {
                let time = time::format_description::parse("[hour]:[minute]:[second]")
                    .unwrap();
                consumer.finish(format_args!(
                    "[{}] [{}] [{}]: {}",
                    TimezoneStrategy::UseLocal.get_now().format(&time).unwrap(),
                    record.level(),
                    record.target(),
                    message
                ));
            })
            .build())
        .plugin(tauri_plugin_store::Builder::default().build())
        .invoke_handler(generate_handler![
            window::window__close,
            appearance::appearance__background,
            appearance::appearance__default_splash
        ])
        .setup(|app| {
            setup_app(app.handle())?;

            Ok(())
        })
        .run(utils::build_context())
        .expect("error while running tauri application");
}
