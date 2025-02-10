#![feature(once_cell_get_mut)]
#![allow(non_snake_case)]

#[macro_use]
extern crate dotenv_codegen;

#[macro_use]
extern crate rust_i18n;

use std::collections::HashMap;
use std::fs;
use anyhow::Result;
use clap::Command;
use lazy_static::lazy_static;
use log::info;
use tauri::{generate_handler, AppHandle};
use tauri_plugin_log::TimezoneStrategy;
use tokio::runtime::Handle;

mod utils;
mod config;
mod app;
mod window;
mod capabilities;
mod events;
mod system;

use crate::app::{appearance, game};
use crate::config::{Config, Language};

// Generate the translation function.
i18n!(
    "../resources/lang",
    fallback = "en-us"
);

lazy_static! {
    /// The system default language, wrapped in an enum for the supported application languages.
    static ref SYSTEM_LANGUAGE: Language = Language::from_locale(utils::system_locale());
}

/// Global function used by both console and desktop
/// applications for preparing the application.
fn setup_app() -> Result<()> {
    // Check if the data directory exists.
    let app_data_dir = utils::app_data_dir()?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir(app_data_dir.join("mods"))?;
        fs::create_dir(app_data_dir.join("cache"))?;
        fs::create_dir(app_data_dir.join("dumps"))?;
        fs::create_dir(app_data_dir.join("sniffer"))?;
    }

    // Initialize the configuration.
    let config = Config::get();

    // Set the language.
    rust_i18n::set_locale(&config.language);

    // If the launcher should elevate, do so now.
    if config.launcher.always_elevate && !system::is_elevated() {
        system::elevate()?;
    }

    Ok(())
}

/// The syntax tree for the command line interface.
fn clap() -> Command {
    Command::new("ysc")
        .about("Desktop application and CLI to interact with Yuan Shen")
        .subcommand(Command::new("sniff")
            .about("Runs the packet sniffer according to the config"))
}

#[tokio::main]
async fn main() {
    // Run the application setup function.
    if let Err(error) = setup_app() {
        eprintln!("Failed to setup application: {:#?}", error);
        return;
    }

    // If we are compiling for Windows, we should remove the console if it exists.
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Console::{GetConsoleWindow, FreeConsole};

        // Check if any arguments were passed.
        let args: Vec<String> = std::env::args().collect();

        // Check if the console is attached.
        let window_handle: HWND = GetConsoleWindow();
        if !cfg!(debug_assertions) && args.len() <= 1 && !window_handle.is_invalid() {
            // Remove it if it is.
            FreeConsole().unwrap();
        }
    }

    // Run command-line argument parser.
    let matches = clap().get_matches();
    let matches = matches.subcommand();

    if matches.is_none() {
        // Set the Tauri async runtime.
        tauri::async_runtime::set(Handle::current());

        // Run the desktop app if no sub-command was provided.
        run_tauri_app();
        return;
    }

    // If we aren't running the desktop application, configure the logger.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    match matches {
        Some(("sniff", _)) => {
            info!("Type 'help' for a list of commands.");
            capabilities::sniffer::run_cli().await;
        }
        _ => panic!("unimplemented command")
    }
}

/// Translates the given key into a localized string.
#[tauri::command]
fn translate(key: String, args: Option<HashMap<String, String>>) -> String {
    let message = t!(key);

    // If no arguments were provided, return the message as-is.
    if args.is_none() {
        return message.to_string();
    }

    // Otherwise, replace the arguments in the message.
    let args = args.unwrap();
    let mut message = message.to_string();
    for (key, value) in args {
        message = message.replace(&format!("%{{{}}}", key), &value);
    }

    message
}

/// Prepares the application for use.
///
/// This is exclusive to the desktop application.
fn setup_tauri_app(_: &AppHandle) -> Result<()> {
    Ok(())
}

/// Runs the Tauri desktop application.
// noinspection RsUnnecessaryQualifications
fn run_tauri_app() {
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
            translate,
            game::game__launch,
            config::config__get,
            window::window__close,
            appearance::appearance__background,
            appearance::appearance__default_splash
        ])
        .setup(|app| {
            setup_tauri_app(app.handle())?;
            Ok(())
        })
        .run(utils::build_context())
        .expect("error while running tauri application");
}
