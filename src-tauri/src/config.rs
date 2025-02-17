use std::fs::File;
use std::sync::{OnceLock, Mutex, MutexGuard};
use serde::{Deserialize, Serialize};
use anyhow::{anyhow, Result};
use crate::{system, utils, SYSTEM_LANGUAGE};

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

    /// Converts the language to a locale string.
    pub fn to_locale(&self) -> &'static str {
        match self {
            Language::English => "en-US",
            Language::Chinese => "zh-CN"
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

/// Creates a copy of the current config state.
#[tauri::command]
pub fn config__get() -> Config {
    Config::get().clone()
}

#[derive(Serialize, Deserialize, Default, PartialEq, Debug, Clone)]
pub struct Config {
    /// The application language.
    ///
    /// This is always used, regardless of the default system language.
    #[serde(default = "Config::default_language")]
    pub language: String,

    /// The configuration used for the launcher.
    #[serde(default)]
    pub launcher: Launcher,

    /// The configuration used for information about the game.
    #[serde(default)]
    pub game: Game,

    /// The configuration for the packet sniffer.
    #[serde(default)]
    pub sniffer: Sniffer
}

impl Config {
    /// Fetches a mutable configuration.
    ///
    /// Deserializes the configuration if it hasn't been done yet.
    pub fn get<'a>() -> MutexGuard<'a, Config> {
        static CONFIG: OnceLock<Mutex<Config>> = OnceLock::new();
        let mutex = CONFIG.get_or_init(|| {
            Mutex::new(deserialize(*SYSTEM_LANGUAGE).unwrap())
        });

        mutex.lock().unwrap()
    }

    /// Returns the default language.
    ///
    /// This is based on the system's language.
    fn default_language() -> String {
        SYSTEM_LANGUAGE.to_locale().to_string()
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Launcher {
    /// Whether to always elevate the launcher on start up or not.
    ///
    /// The launcher might sometimes ask for
    /// elevation regardless to open the game.
    pub always_elevate: bool
}

impl Default for Launcher {
    fn default() -> Self {
        Launcher {
            always_elevate: true
        }
    }
}

/// A game modification that can be injected into the game.
pub enum Modification {
    /// A modified version of 3DMigoto: a DX11 shader modding tool.
    ///
    /// The version used for this game is called [GIMI](https://github.com/SilentNightSound/GI-Model-Importer).
    ///
    /// This modification is managed by the application.
    Migoto,

    /// A general-purpose post-processing utility.
    ///
    /// ReShade is readily available at [reshade.me](https://reshade.me);
    ///
    /// This modification is managed by the application.
    ReShade,

    /// A modding utility developed by `ys4e` for interacting with `ys-compass`.
    ///
    /// This modification is always injected when playing on custom servers, and cannot be configured normally.
    ///
    /// This modification is managed by the application.
    YsHelper,

    /// This represents a DLL specified by the user.
    UnmanagedDll(String)
}

impl Modification {
    /// Parses the string into a modification.
    ///
    /// Returns `None` if the modification is unknown.
    pub fn from_raw(raw: &String) -> Option<Modification> {
        match raw.to_lowercase().as_str() {
            "3dmigoto" | "gimi" => Some(Modification::Migoto),
            "reshade" => Some(Modification::ReShade),
            "yshelper" => Some(Modification::YsHelper),
            _ => {
                if raw.starts_with("dll:") {
                    Some(Modification::UnmanagedDll(raw[4..].to_string()))
                } else {
                    None
                }
            }
        }
    }

    /// Returns the absolute path to the modification's DLL.
    pub fn to_path(&self) -> Result<String> {
        let path = match self {
            Modification::Migoto => system::resolve_path("$APPDATA/mods/3DMigoto/d3d11.dll")?,
            Modification::ReShade => system::resolve_path("$APPDATA/mods/ReShade/ReShade64.dll")?,
            Modification::YsHelper => system::resolve_path("$APPDATA/mods/YSHelper/yshelper.dll")?,
            Modification::UnmanagedDll(path) => system::resolve_path(path)?
        };

        Ok(path.to_string_lossy().to_string())
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
#[serde(rename_all = "kebab-case")]
pub struct Game {
    /// The absolute path to the game's executable.
    ///
    /// This is used to launch the game.
    pub path: String,

    /// A list of modifications to inject into the game.
    ///
    /// Valid, known examples:
    /// - 3DMigoto/GIMI
    /// - ReShade
    ///
    /// Absolute paths to DLLs can also be used when prefixed with `dll:`.
    modifications: Vec<String>,

    /// Whether to disable the anti-cheat.
    ///
    /// Sometimes, the anti-cheat can prevent DLLs from being injected.\
    /// If this is the case, set this to `true`.
    ///
    /// In most cases however, this should be set to `false`.
    pub disable_anti_cheat: bool
}

impl Game {
    /// Determines the name of the executable.
    ///
    /// This is used for checking if the game is running.
    pub fn get_executable_name(&self) -> String {
        self.path
            .replace("\\", "/")
            .split("/")
            .last()
            .unwrap_or(dotenv!("DEFAULT_EXECUTABLE_NAME"))
            .to_string()
    }

    /// Parses the modifications into a list of `Modification`s.
    pub fn modifications(&self) -> Vec<Modification> {
        let mut mods = Vec::new();

        for modification in &self.modifications {
            if let Some(modification) = Modification::from_raw(modification) {
                mods.push(modification);
            }
        }

        mods
    }
}

impl Default for Game {
    fn default() -> Self {
        Game {
            path: dotenv!("DEFAULT_EXECUTABLE_PATH").to_string(),
            modifications: vec!["3DMigoto".to_string()],
            disable_anti_cheat: false
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
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

impl Default for Sniffer {
    fn default() -> Self {
        Sniffer {
            device_name: String::new(),
            filter: "udp portrange 22101-22102".to_string(),
            server_ports: vec![22101, 22102],
            seeds_file: "$APPDATA/sniffer/known-seeds.txt".to_string()
        }
    }
}
