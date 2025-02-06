#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#![feature(once_cell_get_mut)]
#![allow(non_snake_case)]

#[macro_use]
extern crate dotenv_codegen;

use std::fs;
use anyhow::Result;
use clap::Command;
use lazy_static::lazy_static;
use log::info;
use tauri::{generate_handler, AppHandle};
use tauri_plugin_log::TimezoneStrategy;

mod window;
mod utils;
mod app;
mod capabilities;
mod config;

use crate::app::appearance;
use crate::config::{Config, Language};

lazy_static! {
    /// The system default language, wrapped in an enum for the supported application languages.
    static ref LANGUAGE: Language = Language::from_locale(utils::system_locale());
}

/// Global function used by both console and desktop
/// applications for preparing the application.
fn setup_app() -> Result<()> {
    // Check if the data directory exists.
    let app_data_dir = utils::app_data_dir()?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir(app_data_dir.join("cache"))?;
    }

    // Initialize the configuration.
    drop(Config::get());

    Ok(())
}

/// The syntax tree for the command line interface.
fn clap() -> Command {
    Command::new("ysc")
        .about("Desktop application and CLI to interact with Yuan Shen")
        .subcommand(Command::new("sniff")
            .about("Runs the packet sniffer according to the config"))
}

fn main() {
    // Run the application setup function.
    if let Err(error) = setup_app() {
        eprintln!("Failed to setup application: {:#?}", error);
        return;
    }

    // Run command-line argument parser.
    let matches = clap().get_matches();
    let matches = matches.subcommand();

    if matches.is_none() {
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
            info!("running the sniffer...");
        }
        _ => panic!("unimplemented command")
    }
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
