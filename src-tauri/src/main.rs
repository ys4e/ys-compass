#![feature(once_cell_get_mut)]
#![allow(non_snake_case)]

#[macro_use]
extern crate log;

#[macro_use]
extern crate dotenv_codegen;

#[macro_use]
extern crate rust_i18n;

use anyhow::Result;
use clap::{arg, Command};
use game::GameManager;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::fs;
use std::sync::RwLock;
use log::LevelFilter;
use tauri::{generate_handler, AppHandle, Manager};
use tauri_plugin_log::{Target, TargetKind, TimezoneStrategy};
use tokio::runtime::Handle;
use tokio::sync::RwLockReadGuard;

mod app;
mod capabilities;
mod cli;
mod config;
mod database;
mod events;
mod state;
mod system;
mod utils;
mod window;

use crate::app::{appearance, game, profile};
use crate::capabilities::sniffer;
use crate::config::{Config, Language};
use crate::state::*;

// Generate the translation function.
i18n!("../resources/lang", fallback = "en-us");

lazy_static! {
    /// The system default language, wrapped in an enum for the supported application languages.
    static ref SYSTEM_LANGUAGE: Language = Language::from_locale(utils::system_locale());

    /// The global state of the application.
    static ref GLOBAL_STATE: RwLock<PersistentState> = RwLock::new(PersistentState::new());
}

/// Converts a translated string into a String reference.
#[macro_export]
macro_rules! t_str {
    ($key:expr) => {
        &t!($key).to_string()
    };
}

/// Global function used by both console and desktop
/// applications for preparing the application.
async fn setup_app() -> Result<()> {
    // Initialize the configuration.
    let config = Config::fetch();

    // If the launcher should elevate, do so now.
    if config.launcher.always_elevate && !system::is_elevated() {
        system::elevate()?;
    }

    // Check if the data directory exists.
    let app_data_dir = utils::app_data_dir()?;
    if !app_data_dir.exists() {
        fs::create_dir_all(&app_data_dir)?;
        fs::create_dir(app_data_dir.join("mods"))?;
        fs::create_dir(app_data_dir.join("cache"))?;
        fs::create_dir(app_data_dir.join("dumps"))?;
        fs::create_dir(app_data_dir.join("images"))?;
        fs::create_dir(app_data_dir.join("sniffer"))?;
    }

    // Set the language.
    rust_i18n::set_locale(&config.language);

    // Create the database connection pool.
    database::initialize(&config).await?;

    // Load data.
    let mut game_manager = GameManager::get().write().await;
    game_manager.load_all().await?;

    Ok(())
}

/// The syntax tree for the command line interface.
fn clap() -> Command {
    Command::new("ysc")
        .about(t_str!("cli.about"))
        .subcommand(Command::new("sniff").about(t_str!("cli.sniff")))
        .subcommand(
            Command::new("game")
                .about(t_str!("cli.game"))
                .arg_required_else_help(true)
                .subcommand(
                    Command::new("version")
                        .about(t_str!("cli.game.version"))
                        .arg_required_else_help(true)
                        .subcommand(
                            Command::new("install").about(t_str!("cli.game.version.install")),
                        )
                        .subcommand(Command::new("locate").about(t_str!("cli.game.version.locate")))
                        .subcommand(
                            Command::new("uninstall").about(t_str!("cli.game.version.uninstall")),
                        )
                        .subcommand(Command::new("list").about(t_str!("cli.game.version.list"))),
                )
                .subcommand(
                    Command::new("profile")
                        .about(t_str!("cli.game.profile"))
                        .arg_required_else_help(true)
                        .subcommand(Command::new("new").about(t_str!("cli.game.profile.new")))
                        .subcommand(
                            Command::new("select").about(t_str!("cli.game.profile.select")),
                        ),
                )
                .subcommand(
                    Command::new("launch")
                        .about(t_str!("cli.game.launch"))
                        .arg(arg!(--profile <NAME>)),
                ),
        )
}

#[tokio::main]
async fn main() {
    // Run the application setup function.
    if let Err(error) = setup_app().await {
        eprintln!("Failed to setup application: {:#?}", error);
        return;
    }

    // If we are compiling for Windows, we should remove the console if it exists.
    #[cfg(windows)]
    unsafe {
        use windows::Win32::Foundation::HWND;
        use windows::Win32::System::Console::{FreeConsole, GetConsoleWindow};

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
        run_tauri_app().await;
        return;
    }

    // If we aren't running the desktop application, configure the logger.
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init();

    cli::run(matches).await
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
fn setup_tauri_app(
    app_handle: &AppHandle,
    game_manager: RwLockReadGuard<'_, GameManager>,
) -> Result<()> {
    // Initialize global state.
    app_handle.manage(SelectedProfile::new(game_manager));

    Ok(())
}

/// Runs the Tauri desktop application.
// noinspection RsUnnecessaryQualifications
async fn run_tauri_app() {
    let game_manager = GameManager::get().read().await;

    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .targets([
                    Target::new(TargetKind::Stdout),
                    Target::new(TargetKind::LogDir { file_name: None }),
                ])
                .level(LevelFilter::Debug)
                .format(|consumer, message, record| {
                    let time = time::format_description::parse("[hour]:[minute]:[second]").unwrap();
                    consumer.finish(format_args!(
                        "[{}] [{}] [{}]: {}",
                        TimezoneStrategy::UseLocal.get_now().format(&time).unwrap(),
                        record.level(),
                        record.target(),
                        message
                    ));
                })
                .build(),
        )
        .plugin(tauri_plugin_store::Builder::default().build())
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(generate_handler![
            translate,
            app::set_language,
            game::game__is_open,
            game::game__launch,
            game::game__locate,
            profile::profile__get_all,
            profile::profile__new_profile,
            profile::profile__set_profile,
            sniffer::sniffer__load,
            app::sniffer::sniffer__run,
            app::sniffer::sniffer__open,
            config::config__get,
            window::window__close,
            appearance::appearance__background,
            appearance::appearance__default_splash
        ])
        .setup(|app| {
            setup_tauri_app(app.handle(), game_manager)?;
            Ok(())
        })
        .run(utils::build_context())
        .expect("error while running tauri application");
}
