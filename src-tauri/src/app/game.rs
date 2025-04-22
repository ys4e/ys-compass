use crate::config::Config;
use crate::utils::MaybeError;
use crate::{database, system, utils, GLOBAL_STATE};
use anyhow::{anyhow, Result};
use clap::ArgMatches;
use lazy_static::lazy_static;
use log::warn;
use regex::Regex;
use serde::{Deserialize, Serialize};
use sqlx::Error;
use std::path::PathBuf;
use std::sync::MutexGuard;
use std::thread::sleep;
use std::time::Duration;
use tauri::State;
use tokio::sync::{watch, watch::{Sender as WatchSender, Receiver as WatchReceiver}, RwLock};

use crate::state::SelectedProfile;
#[cfg(windows)]
use crate::{sys_str, system::AsCString};
#[cfg(windows)]
use windows::Win32::{Foundation::HANDLE, System::Threading::LPTHREAD_START_ROUTINE};

lazy_static! {
    static ref GAME_MANAGER: RwLock<GameManager> = RwLock::new(GameManager::default());
    static ref VERSION_STRING_REGEX: Regex =
        Regex::new(r"(OS|CN)(REL|CB)Win([1-9])\.([0-9])\.([0-9]*)").unwrap();
    static ref GAME_STATUS: (WatchSender<bool>, WatchReceiver<bool>) = watch::channel(false);
}

/// A game launch profile.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub version: Version,
    pub tools: Vec<Tool>,
    pub mods: Vec<Mod>,
    pub launch_args: String,
}

impl Profile {
    /// Saves the profile to the database.
    ///
    /// If it already exists, it updates the values.
    pub async fn save(&self) -> Result<()> {
        let pool = database::get_pool();

        // Convert tools and mods to a string.
        let tools = self
            .tools
            .iter()
            .map(|tool| tool.id.clone())
            .collect::<Vec<String>>()
            .join(",");

        let mods = self
            .mods
            .iter()
            .map(|r#mod| r#mod.id.clone())
            .collect::<Vec<String>>()
            .join(",");

        sqlx::query!(
            r#"INSERT INTO `profiles` (`id`, `name`, `icon`, `version`, `tools`, `mods`, `launch_args`) VALUES
            ($1, $2, $3, $4, $5, $6, $7) ON CONFLICT(`id`) DO UPDATE SET
            `name` = $2, `icon` = $3, `version` = $4, `tools` = $5, `mods` = $6, `launch_args` = $7"#,
            self.id, self.name, self.icon, self.version.version, tools, mods, self.launch_args
        ).execute(&pool).await?;

        Ok(())
    }
}

/// A game 'modification'.
///
/// This links to `Modification`.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Tool {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub path: String,
}

/// A game modification.
///
/// This represents things such as game plugins or visual mods.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Mod {
    pub id: String,
    pub name: String,
    pub icon: String,
    pub path: String,
    pub version: String,
    pub tool: Tool,
}

/// A game version.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct Version {
    pub version: String,
    pub path: String,
}

impl Version {
    /// Saves the profile to the database.
    ///
    /// If it already exists, it updates the values.
    pub async fn save(&self) -> Result<()> {
        let pool = database::get_pool();

        sqlx::query!(
            r#"INSERT INTO `versions` (`version`, `path`) VALUES
            ($1, $2) ON CONFLICT(`version`) DO UPDATE SET
            `path` = $2"#,
            self.version,
            self.path
        )
        .execute(&pool)
        .await?;

        Ok(())
    }
}

/// A manager for parts of the game.
///
/// Includes managing:
/// - profiles
/// - versions
/// - tools
/// - mods
#[derive(Default)]
pub struct GameManager {
    pub profiles: Vec<Profile>,
    pub versions: Vec<Version>,
    pub tools: Vec<Tool>,
    pub mods: Vec<Mod>,
}

impl GameManager {
    /// Gets the game manager lock.
    pub fn get<'a>() -> &'a RwLock<GameManager> {
        &GAME_MANAGER
    }

    /// Fetches a profile by its ID.
    ///
    /// This returns a clone of the profile data.
    pub fn get_profile<S: AsRef<str>>(&self, profile_id: S) -> Option<Profile> {
        let profile_id = profile_id.as_ref();

        self.profiles.iter().find(|p| p.id.eq(profile_id)).cloned()
    }

    /// Saves the given profile to the database.
    pub async fn save_profile(&mut self, mut profile: Profile) -> Result<()> {
        // Set the profile ID.
        profile.id = utils::random_id();

        // Write the profile to the database.
        profile.save().await
    }

    /// Loads all attributes from the database.
    pub async fn load_all(&mut self) -> Result<()> {
        // Load all data.
        self.load_tools().await?;
        self.load_mods().await?;
        self.load_versions().await?;
        self.load_profiles().await?;

        Ok(())
    }

    /// Loads all tools from the database.
    pub async fn load_tools(&mut self) -> Result<()> {
        let pool = database::get_pool();

        // Get tools from the database.
        let Ok(results) = sqlx::query!("SELECT * FROM `tools`").fetch_all(&pool).await else {
            return Err(anyhow!("Unable to query database for tools."));
        };

        // Parse tools.
        self.tools.clear();
        for result in results {
            self.tools.push(Tool {
                id: result.id,
                name: result.name,
                icon: result.icon,
                path: result.path,
            });
        }

        Ok(())
    }

    /// Loads all mods from the database.
    ///
    /// This might fail if tools are not loaded first.
    pub async fn load_mods(&mut self) -> Result<()> {
        let pool = database::get_pool();

        // Get mods from the database.
        let Ok(results) = sqlx::query!("SELECT * FROM `mods`").fetch_all(&pool).await else {
            return Err(anyhow!("Unable to query database for mods."));
        };

        // Parse mods.
        self.mods.clear();
        for result in results {
            let Some(tool) = self.tools.iter().find(|tool| tool.id == result.tool) else {
                warn!("Tool {} not found for mod: {}.", result.tool, result.id);
                continue;
            };

            self.mods.push(Mod {
                id: result.id,
                name: result.name,
                icon: result.icon,
                path: result.path,
                version: result.version,
                tool: tool.clone(),
            });
        }

        Ok(())
    }

    /// Loads all known versions from the database.
    pub async fn load_versions(&mut self) -> Result<()> {
        let pool = database::get_pool();

        // Get versions from the database.
        let Ok(results) = sqlx::query!("SELECT * FROM `versions`")
            .fetch_all(&pool)
            .await
        else {
            return Err(anyhow!("Unable to query database for versions."));
        };

        // Parse versions.
        self.versions.clear();
        for result in results {
            self.versions.push(Version {
                version: result.version,
                path: result.path,
            });
        }

        Ok(())
    }

    /// Loads all profiles from the database.
    ///
    /// This requires you to load the following:
    /// - tools
    /// - mods
    /// - versions
    pub async fn load_profiles(&mut self) -> Result<()> {
        let pool = database::get_pool();

        // Get profiles from the database.
        let Ok(results) = sqlx::query!("SELECT * FROM `profiles`")
            .fetch_all(&pool)
            .await
        else {
            return Err(anyhow!("Unable to query database for profiles."));
        };

        // Parse profiles.
        self.profiles.clear();
        for result in results {
            let Some(version) = self.versions.iter().find(|v| v.version.eq(&result.version)) else {
                warn!(
                    "Version {} not found for profile: {}.",
                    result.version, result.id
                );
                continue;
            };

            let profile = Profile {
                id: result.id,
                name: result.name,
                icon: result.icon,
                version: version.clone(),
                tools: match result.tools {
                    Some(tools) => {
                        let ids = tools.split(',');
                        let mut tools = Vec::new();

                        for id in ids {
                            if let Some(tool) = self.tools.iter().find(|tool| tool.id == id) {
                                tools.push(tool.clone());
                            }
                        }

                        tools
                    }
                    None => Vec::new(),
                },
                mods: match result.mods {
                    Some(mods) => {
                        let ids = mods.split(',');
                        let mut mods = Vec::new();

                        for id in ids {
                            if let Some(r#mod) = self.mods.iter().find(|m| m.id == id) {
                                mods.push(r#mod.clone());
                            }
                        }

                        mods
                    }
                    None => Vec::new(),
                },
                launch_args: result.launch_args,
            };

            self.profiles.push(profile);
        }

        Ok(())
    }
}

/// Returns a new channel reference to listen for the game status.
pub fn new_status_listener() -> WatchReceiver<bool> {
    GAME_STATUS.1.clone()
}

/// Utility method to check if the game is currently running.
///
/// In the event of any errors, this will return `false`.
#[tauri::command]
pub fn game__is_open(profile: State<SelectedProfile>) -> bool {
    // Lock the selected profile.
    let Some(ref profile) = *profile.0.lock().unwrap() else {
        return false;
    };

    let path = &profile.version.path;
    system::find_process(utils::get_executable_name(path))
}

/// Enables the 'process watcher'.
///
/// This will look for the game process.
///
/// Once the game is closed, this will need to be re-run.
pub fn watch_game(profile: Profile) {
    // Get the game path.
    let path = profile.version.path.clone();

    // Get the status channel.
    let sender = GAME_STATUS.0.clone();

    std::thread::spawn(move || {
        // If the game is not open yet, wait for it to open.
        while !system::find_process(utils::get_executable_name(&path)) {
            trace!("Waiting for game process to open...");
            sleep(Duration::from_secs(2));
        }

        // Once the game is open, notify listeners.
        sender.send(true).unwrap();

        // Wait for the game to close.
        while system::find_process(utils::get_executable_name(&path)) {
            sleep(Duration::from_secs(2));
        }

        // Once the game is closed, notify listeners.
        sender.send(false).unwrap();
    });
}

/// Launches the game.
///
/// If the game is already open, this fails with a helpful error message.
///
/// # Errors
///
/// Errors are not localized and need to be looked up by the
/// caller before displaying to the user.
#[tauri::command]
pub fn game__launch(profile: State<SelectedProfile>) -> MaybeError<()> {
    // Check if the game process is already running.
    if game__is_open(profile.clone()) {
        return Err("game.error.already-open");
    }

    // Get the configuration.
    let config = Config::get();

    // Lock the selected profile.
    let Some(ref profile) = *profile.0.lock().unwrap() else {
        return Err("game.error.launch.no-profile");
    };

    // Run the game watcher.
    watch_game(profile.clone());

    // Launch the game.
    launch_game(profile, config)
}

/// Launches the game.
///
/// This is invoked from the CLI.
pub async fn cli_game__launch(matches: &ArgMatches) {
    let game_manager = GameManager::get().read().await;

    // Get the default/selected profile, or the one specified.
    let Some(profile) = (match matches.get_one::<String>("profile") {
        Some(profile) => game_manager.get_profile(profile),
        None => {
            let state = GLOBAL_STATE.read().unwrap();
            let Some(profile) = &state.selected_profile else {
                warn!("{}", t!("game.error.launch.no-profile"));
                return;
            };

            game_manager.get_profile(profile)
        }
    }) else {
        warn!("{}", t!("game.error.launch.invalid-profile"));
        return;
    };

    // Lock the configuration.
    let config = Config::get();

    // Launch the game.
    if let Err(error) = launch_game(&profile, config) {
        warn!("{} {}", t!("launcher.error.profile.unknown"), error);
    }
}

/// Locates a game installation, then adds it to the version database.
///
/// # Errors
///
/// Errors are not localized and need to be looked up by the
/// caller before displaying to the user.
#[tauri::command]
pub async fn game__locate(path: String) -> MaybeError<()> {
    locate_game(path).await
}

/// Locates an existing game installation.
pub async fn locate_game(path: String) -> MaybeError<()> {
    // Load the executable data into memory.
    // "is there a better way to do this? probably not."
    let executable_path = PathBuf::from(&path);
    let Some(parent) = executable_path.parent() else {
        return Err("backend.version.resolve.error");
    };

    // If a `UnityPlayer.dll` is found, use it for the version string lookup.
    let unity_player = parent.join("UnityPlayer.dll");
    let game_data = match std::fs::read(if unity_player.exists() {
        unity_player
    } else {
        executable_path
    }) {
        Ok(data) => data,
        Err(_) => return Err("backend.version.resolve.error"),
    };
    let game_data = String::from_utf8_lossy(&game_data);

    // Match the version string.
    let Some(captures) = VERSION_STRING_REGEX.captures(&game_data) else {
        return Err("backend.version.resolve.error");
    };
    let Some(version_string) = captures.get(0) else {
        return Err("backend.version.resolve.error");
    };
    let version_string = version_string.as_str();

    // Insert the game into the database.
    let pool = database::get_pool();

    // Check if the version already exists.
    match sqlx::query!(
        "SELECT * FROM `versions` WHERE `version` = $1",
        version_string
    )
    .fetch_one(&pool)
    .await
    {
        Err(Error::RowNotFound) => (),
        Ok(_) => return Err("backend.version.resolve.exists"),
        _ => return Err("database.query-failed"),
    }

    // Otherwise, insert the version.
    let version = Version {
        version: version_string.to_string(),
        path,
    };

    if let Err(error) = version.save().await {
        warn!("Failed to insert version: {}", error);
        return Err("database.query-failed");
    };

    Ok(())
}

// ------------------------------ BEWARE: Below is all platform-dependent code! ------------------------------ \\

/// Internal method used to launch the game.
///
/// # On Linux/macOS
///
/// This uses a combination (or user preference) of Wine and Proton to run the game.
///
/// The game executable is run without privilege, then the
/// game modifications specified in the configuration are loaded afterward.
#[cfg(unix)]
fn launch_game(_: MutexGuard<'_, Config>) -> MaybeError<()> {
    Err("game.error.launch.unsupported")
}

// ------------------------------ BEWARE: Below is all Windows API code! ------------------------------ \\

/// Internal method used to launch the game.
///
/// # On Windows
///
/// This uses the Windows API to launch the game in various steps:
/// 1. Opening the game and obtaining a handle.
/// 2. Disabling the anti-cheat if specified.
/// 3. Injecting any DLLs specified by the user.
#[cfg(windows)]
fn launch_game(profile: &Profile, config: MutexGuard<'_, Config>) -> MaybeError<()> {
    use log::warn;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::ResumeThread;

    let game_config = &config.game;
    let version = &profile.version;

    // 1. Launch the game and obtain handles.
    let (thread, process) = open_game(&version.path, &profile.launch_args)?;

    // 2. Disable the anti-cheat if specified.
    let disable_ac = game_config.disable_anti_cheat;
    if disable_ac {
        unsafe {
            wait_for_driver(&process)?;
        }
    }

    // 3. Inject any DLLs specified by the user.
    let load_library = unsafe {
        use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

        let kernel = "kernel32.dll".as_cstring();
        let kernel = match GetModuleHandleA(sys_str!(kernel)) {
            Ok(handle) => handle,
            Err(_) => return Err("game.error.launch.unknown"),
        };

        let load_library = "LoadLibraryA".as_cstring();
        match GetProcAddress(kernel, sys_str!(load_library)) {
            Some(ptr) => std::mem::transmute::<_, LPTHREAD_START_ROUTINE>(ptr),
            None => return Err("game.error.launch.dll-fail"),
        }
    };

    if !disable_ac {
        unsafe {
            suspend(&process)?;
        }
    }

    // Inject all DLLs in the configuration.
    for tool in &profile.tools {
        // Resolve the tool's path.
        let Ok(path) = system::resolve_path(&tool.path) else {
            warn!("{}", t!("backend.path.error.modification"));
            continue;
        };

        if !path.exists() {
            warn!("{}", t!("backend.path.error.modification"));
            continue;
        }

        // Check the tool type.
        let Some(extension) = path.extension() else {
            warn!("{}", t!("backend.path.error.modification"));
            continue;
        };

        let path = path.to_string_lossy().to_string();
        match extension.to_string_lossy().as_ref() {
            "dll" => unsafe {
                inject_dll(&process, load_library, path)?;
            },
            "exe" => {
                if let Err(error) = system::open_executable(&path, None) {
                    warn!("{} {:?}", t!("game.error.launch.exe-fail"), error)
                }
            }
            _ => warn!("{}: '{}'", t!("game.error.launch.unknown-tool"), tool.name),
        }
    }

    if !disable_ac {
        unsafe {
            resume(&process)?;
        }
    }

    // Finally, clean up any left-over handles.
    unsafe {
        _ = ResumeThread(thread);
        _ = CloseHandle(process);
    }

    Ok(())
}

/// This type is used by both 'suspend' and 'resume' methods.
#[cfg(windows)]
type NtSuspendProcess = unsafe extern "system" fn(HANDLE) -> i32;

/// Internal method used on Windows systems to suspend the given process handle.
///
/// This uses the NT API to suspend the process.
#[cfg(windows)]
unsafe fn suspend(process: &HANDLE) -> MaybeError<()> {
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    let nt = "ntdll".as_cstring();
    let nt_module = match GetModuleHandleA(sys_str!(nt)) {
        Ok(handle) => handle,
        Err(_) => return Err("game.error.launch.unknown"),
    };

    let suspend = "NtSuspendProcess".as_cstring();
    let ptr = match GetProcAddress(nt_module, sys_str!(suspend)) {
        Some(ptr) => ptr,
        None => return Err("game.error.launch.unknown"),
    };

    // Call the function.
    let func = std::mem::transmute::<_, NtSuspendProcess>(ptr);
    func(*process);

    Ok(())
}

/// Internal method used on Windows systems to resume the given process handle.
///
/// This uses the NT API to resume the process.
#[cfg(windows)]
unsafe fn resume(process: &HANDLE) -> MaybeError<()> {
    use windows::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    let nt = "ntdll".as_cstring();
    let nt_module = match GetModuleHandleA(sys_str!(nt)) {
        Ok(handle) => handle,
        Err(_) => return Err("game.error.launch.unknown"),
    };

    let resume = "NtResumeProcess".as_cstring();
    let ptr = match GetProcAddress(nt_module, sys_str!(resume)) {
        Some(ptr) => ptr,
        None => return Err("game.error.launch.unknown"),
    };

    // Call the function.
    let func = std::mem::transmute::<_, NtSuspendProcess>(ptr);
    func(*process);

    Ok(())
}

/// Internal method used on Windows systems to open the executable.
///
/// This returns the handles of the thread and process.
#[cfg(windows)]
fn open_game(path: &String, launch_args: &str) -> Result<(HANDLE, HANDLE), &'static str> {
    use std::mem::size_of;
    use sysinfo::System;
    use windows::Win32::Foundation::HANDLE;

    // Get token to open process.
    let token = unsafe {
        use windows::Win32::Security::TOKEN_ALL_ACCESS;
        use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

        // Check for proper elevation & to gain a token.
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &mut token).is_err() {
            return Err("game.error.launch.not-elevated");
        }

        token
    };

    // Find the Windows Explorer process.
    let explorer = {
        let mut system = System::new();
        system.refresh_all();

        let mut process = system.processes_by_exact_name("explorer.exe".as_ref());
        let Some(process) = process.next() else {
            return Err("game.error.launch.no-parent");
        };

        process.pid().as_u32()
    };

    // Open the Windows Explorer process.
    let explorer = unsafe {
        use windows::Win32::System::Threading::{OpenProcess, PROCESS_ALL_ACCESS};
        match OpenProcess(PROCESS_ALL_ACCESS, false, explorer) {
            Ok(handle) => handle,
            Err(_) => return Err("game.error.launch.no-parent"),
        }
    };

    if explorer.is_invalid() {
        return Err("game.error.launch.no-parent");
    }

    unsafe {
        use windows::core::PSTR;
        use windows::Win32::System::Threading::{
            CreateProcessAsUserA, EXTENDED_STARTUPINFO_PRESENT, PROCESS_INFORMATION, STARTUPINFOA,
            STARTUPINFOEXA,
        };

        let mut process_info: PROCESS_INFORMATION = Default::default();
        let mut start_info = STARTUPINFOEXA {
            StartupInfo: STARTUPINFOA {
                cb: size_of::<STARTUPINFOEXA>() as u32,
                ..Default::default()
            },
            lpAttributeList: Default::default(),
        };

        // Create the process.
        let Ok(path) = system::canonicalize(path) else {
            return Err("game.error.launch.bad-path");
        };

        let path = path.as_cstring();
        let arguments = launch_args.as_cstring();

        let result = CreateProcessAsUserA(
            Some(token),
            sys_str!(path),
            Some(PSTR(arguments.as_ptr() as *mut u8)),
            None,
            None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None,
            None,
            &mut start_info as *mut _ as *mut _,
            &mut process_info,
        );

        // Check if the process was created.
        if let Err(error) = result {
            warn!("Failed to launch game: {}", error);
            return Err("game.error.launch.unknown");
        };

        Ok((process_info.hThread, process_info.hProcess))
    }
}

/// Internal method used on Windows systems to disable the anti-cheat.
///
/// This works by suspending the process until the anti-cheat driver is unloaded.
#[cfg(windows)]
unsafe fn wait_for_driver(process: &HANDLE) -> MaybeError<()> {
    use std::{
        ffi::c_void,
        mem::{size_of, size_of_val},
        ptr,
    };
    use windows::Win32::System::ProcessStatus::{EnumDeviceDrivers, GetDeviceDriverBaseNameA};

    unsafe fn driver_loaded() -> MaybeError<bool> {
        let mut needed = 0;
        let mut drivers: [*mut c_void; 1024] = [ptr::null_mut(); 1024];

        // Get the list of drivers.
        if EnumDeviceDrivers(drivers.as_mut_ptr(), size_of::<u32>() as u32, &mut needed).is_err() {
            return Err("game.error.launch.unknown");
        }

        // Check if there is an under-allocation of the list.
        if needed > size_of_val(&drivers) as u32 {
            return Err("game.error.launch.unknown");
        }

        // Enumerate over all drivers.
        let driver_count = needed as usize / size_of_val(&drivers[0]);
        for driver in drivers.iter().take(driver_count) {
            let mut name = [0u8; 256];
            let size = GetDeviceDriverBaseNameA(*driver, &mut name);

            if size == 0 {
                continue;
            }

            // Convert the name into a string.
            if let Ok(name) = String::from_utf8(name[..size as usize].to_vec()) {
                if name.starts_with(dotenv!("GAME_DRIVER_NAME")) {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    let mut found = false;

    // Wait until the driver is first found.
    while !found {
        found = driver_loaded()?;
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    // Wait for the driver to unload.
    suspend(process)?;

    while driver_loaded()? {
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    resume(process)?;

    Ok(())
}

/// Internal method used on Windows systems to remotely inject a DLL.
///
/// This uses `LoadLibrary` provided by the Windows API.
#[cfg(windows)]
unsafe fn inject_dll(
    process: &HANDLE,
    load_library: LPTHREAD_START_ROUTINE,
    dll_path: String,
) -> MaybeError<()> {
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
    use windows::Win32::System::Memory::{
        VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
    };
    use windows::Win32::System::Threading::{CreateRemoteThread, WaitForSingleObject};

    let path_length = dll_path.len() + 1;

    let dll_path = dll_path.as_cstring();
    let path = sys_str!(dll_path);

    // Allocate memory for the thread to access the DLL path.
    let dll_path = VirtualAllocEx(
        *process,
        None,
        path_length,
        MEM_RESERVE | MEM_COMMIT,
        PAGE_READWRITE,
    );

    // Write the DLL path to the process.
    if WriteProcessMemory(
        *process,
        dll_path,
        path.as_ptr() as *const _,
        path_length,
        None,
    )
    .is_err()
    {
        return Err("game.error.launch.dll-fail");
    };

    // Invoke the LoadLibrary function.
    let Ok(thread) = CreateRemoteThread(*process, None, 0, load_library, Some(dll_path), 0, None)
    else {
        // Free the memory.
        _ = VirtualFreeEx(*process, dll_path, 0, MEM_RELEASE);

        return Err("game.error.launch.dll-fail");
    };

    // Wait for the thread to exit.
    if WaitForSingleObject(thread, 2000) == WAIT_OBJECT_0 {
        _ = VirtualFreeEx(*process, dll_path, 0, MEM_RELEASE);
    }

    // Close the thread handle.
    _ = CloseHandle(thread);

    Ok(())
}
