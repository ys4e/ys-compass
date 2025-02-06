use std::fs::File;
use std::sync::{OnceLock, Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use crate::{utils, LANGUAGE};

#[derive(PartialEq, Copy, Clone)]
pub enum Language {
    English,
    Chinese
}

impl Language {
    /// Creates a language from a locale string.
    pub fn from_locale(locale: String) -> Self {
        match locale.to_lowercase().as_str() {
            "zh-cn" | "zh-hk" => Language::Chinese,
            _ => Language::English
        }
    }

    /// Returns the default configuration for the language.
    pub fn default_config(&self) -> &'static str {
        match self {
            Language::English => include_str!("../../resources/config/en-us.yml"),
            Language::Chinese => include_str!("../../resources/config/zh-cn.yml")
        }
    }
}

/// Attempts to deserialize the configuration file.
///
/// If it doesn't exist, the default configuration for the language will be used.
pub fn deserialize(language: Language) -> Result<Config> {
    // Resolve the app data directory.
    let Ok(app_data_dir) = utils::app_data_dir() else {
        return Err(anyhow!("Failed to resolve app data directory."));
    };
    let config_path = app_data_dir.join("config.yml");

    // If the config file doesn't exist, save the default one.
    if !config_path.exists() {
        let default_config = default_config(language)?;
        save_config(&default_config)?;

        return Ok(default_config);
    }

    // Otherwise, deserialize the configuration file.
    Ok(serde_yml::from_reader(File::open(config_path)?)?)
}

/// Saves the configuration to the file.
pub fn save_config(config: &Config) -> Result<()> {
    // Resolve the app data directory.
    let Ok(app_data_dir) = utils::app_config_dir() else {
        return Err(anyhow!("Failed to resolve app data directory."));
    };
    let config_path = app_data_dir.join("config.yml");

    // Save the configuration to the file.
    serde_yml::to_writer(File::create(config_path)?, config)?;

    Ok(())
}

/// Returns the default configuration for the specified language.
pub fn default_config(language: Language) -> Result<Config> {
    let default_config = language.default_config();

    // Deserialize the default configuration.
    Ok(serde_yml::from_str(default_config)?)
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
pub struct Config {
    pub sniffer: Sniffer
}

impl Config {
    /// Fetches a mutable configuration.
    ///
    /// Deserializes the configuration if it hasn't been done yet.
    pub fn get<'a>() -> MutexGuard<'a, Config> {
        static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
        let mutex = CONFIG.get_or_init(|| {
            Mutex::new(deserialize(*LANGUAGE).unwrap())
        });

        mutex.lock().unwrap()
    }
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct Sniffer {
    /// The name of the network interface to use.
    ///
    /// You will be asked to set this during the setup process.\
    /// Once it is set, you can change it here.
    pub device_name: String,

    /// The packet capturing filter to use.
    ///
    /// If you don't know how to write filters, leave this default.\
    /// In most scenarios, you won't need to change this.
    pub filter: String,

    /// A list of ports that the server listens on.
    ///
    /// If you change the capture filter, you will likely need to change this too.\
    /// This is used for determining which side sent a packet. (client/server)
    pub server_ports: Vec<u16>,

    /// The path to the 'known seeds' file.
    ///
    /// This file should be readable and writable.\
    /// It contains all encryption seeds used recently.
    pub seeds_file: String
}
