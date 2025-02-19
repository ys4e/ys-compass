mod game;

use clap::ArgMatches;
use log::info;
use crate::app::game as app_game;
use crate::capabilities;

/// Command-line interface command handler.
pub async fn run(matches: Option<(&str, &ArgMatches)>) {
    match matches {
        Some(("sniff", _)) => {
            info!("Type 'help' for a list of commands.");
            capabilities::sniffer::run_cli().await;
        }
        Some(("game", sub_matches)) => {
            match sub_matches.subcommand().unwrap() {
                ("version", sub_matches) => game::version(sub_matches).await,
                ("profile", sub_matches) => game::profile(sub_matches).await,
                ("launch", sub_matches) => app_game::cli_game__launch(sub_matches).await,
                _ => unimplemented!()
            }
        }
        _ => panic!("unimplemented command")
    }
}
