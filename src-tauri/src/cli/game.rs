use clap::ArgMatches;
use dialoguer::Input;
use dialoguer::{theme::ColorfulTheme, Select};
use log::warn;
use crate::app::game;
use crate::app::game::{GameManager, Profile};

/// Parses the command tree for `game version`.
pub async fn version(matches: &ArgMatches) {
    match matches.subcommand().unwrap() {
        ("locate", _) => locate_version().await,
        _ => unimplemented!()
    }
}

/// Prompts the user to input a path to the game version.
async fn locate_version() {
    // Ask the user to input the path.
    let bad_path = t!("game.error.launch.bad-path");
    let bad_path = bad_path.as_ref();

    let Ok(path) = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("cli.game.version.locate.prompt"))
        .validate_with(move |input: &String| -> Result<(), &str> {
            if input.is_empty() || !input.ends_with(".exe") {
                Err(bad_path)
            } else {
                Ok(())
            }
        })
        .interact_text() else {
        warn!("{}", t!("game.error.launch.bad-path"));
        return;
    };

    // Locate the game.
    if let Err(error) = game::locate_game(path).await {
        warn!("{}", t!(error));
    }
}

/// Parses the command tree for `game profile`.
pub async fn profile(matches: &ArgMatches) {
    match matches.subcommand().unwrap() {
        ("new", _) => new_profile().await,
        _ => unimplemented!()
    }
}

/// Prompts the user to create a new profile.
async fn new_profile() {
    // Ask the user to input a name for the profile.
    let Ok(name) = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("cli.game.profile.new.prompt.1"))
        .interact_text() else {
        warn!("{}", t!("launcher.error.profile.bad-name"));
        return;
    };

    // Resolve versions.
    let mut game_manager = GameManager::get().write().await;
    let versions = &game_manager.versions.iter()
        .map(|v| v.version.clone())
        .collect::<Vec<String>>();

    // Ask the user to input a path to the game version.
    let Ok(version) = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(t!("cli.game.profile.new.prompt.2"))
        .default(0)
        .items(&versions)
        .interact() else {
        warn!("{}", t!("launcher.error.profile.bad-version"));
        return;
    };

    // Create the profile.
    let profile = Profile {
        name,
        version: game_manager.versions[version].clone(),
        ..Default::default()
    };

    // Save the profile.
    if let Err(error) = game_manager.save_profile(profile).await {
        warn!("{} {}", t!("launcher.error.profile.unknown"), error);
    }
}
