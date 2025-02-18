use std::path::PathBuf;
use anyhow::Result;
use sysinfo::System;
use crate::utils;

#[cfg(windows)]
use std::ffi::CString;

/// Encodes a string as a wide string.
///
/// # Compatability
///
/// This only works for the Windows operating system, where this matters.
#[cfg(windows)]
macro_rules! wide_str {
    ($str:expr) => {
        std::ffi::OsString::from($str)
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<u16>>()
    };
}

/// Encodes a managed `String` as a Windows `PCSTR`.
///
/// # Compatability
///
/// This only works for the Windows operating system, where this matters.
#[cfg(windows)]
#[macro_export]
macro_rules! sys_str {
    ($str:expr) => {
        windows::core::PCSTR($str.as_ptr() as *const u8)
    };
}

/// Checks if the process is elevated.
///
/// # On Windows
///
/// This uses `is_elevated` to check if the process is running as Administrator.
///
/// # On Linux
///
/// This uses `sudo` to check if the process is running as root.
///
/// # On macOS
///
/// This uses `sudo` to check if the process is running as root.
pub fn is_elevated() -> bool {
    #[cfg(windows)]
    {
        is_elevated::is_elevated()
    }

    #[cfg(unix)]
    {
        sudo::check() == sudo::RunningAs::Root
    }
}

/// Reruns the current process as an elevated process.
///
/// # On Windows
///
/// This uses the Windows API to restart the application as administrator.
///
/// # On Linux
///
/// This uses `sudo` to restart the application as root.
///
/// # On macOS
///
/// This uses `sudo` to restart the application as root.
pub fn elevate() -> Result<()> {
    #[cfg(windows)]
    {
        use std::process::exit;
        use windows::core::PCWSTR;
        use std::env::{args, current_exe};
        use std::os::windows::ffi::OsStrExt;
        use windows::Win32::UI::{WindowsAndMessaging::SW_SHOW, Shell::ShellExecuteW};

        let exe_path = current_exe()?;
        let args: Vec<String> = args().skip(1)
            .collect(); // Skip the first argument, which is the path to the exe
        let args_string = args.join(" ");

        let operation = wide_str!("runas");
        let file = wide_str!(exe_path);
        let parameters = wide_str!(args_string);

        unsafe {
            ShellExecuteW(
                None,
                PCWSTR(operation.as_ptr()),
                PCWSTR(file.as_ptr()),
                PCWSTR(parameters.as_ptr()),
                None,
                SW_SHOW,
            );
        }

        exit(0);
    }

    #[cfg(unix)]
    {
        sudo::escalate_if_needed()?;
        Ok(())
    }
}

/// Resolves a path to an absolute path.
///
/// # Explanation
///
/// All paths resolve from the root, unless they are prefixed with a `$` character.
///
/// Aliases that are resolved properly:
/// - `$APPDATA` - The application's data directory.
pub fn resolve_path<S: AsRef<str>>(path: S) -> Result<PathBuf> {
    Ok(PathBuf::from(
        path.as_ref().to_string()
            .replace("\\", "/")
            .replace("$APPDATA", &utils::app_data_dir()?.to_string_lossy())
    ))
}

/// The result of opening the file.
///
/// This is used in `system::open_executable`.
#[derive(Debug)]
pub enum OpenResult {
    /// The file opened successfully.
    Success,

    /// This can occur for numerous reasons:
    /// 1. The file doesn't exist.
    /// 2. The user declined an interactive prompt to open the file.
    ///
    /// Interactive prompts exist when the file isn't trusted\
    /// or the file needs special privileges.
    Failed
}

/// Attempts to open the executable file.
///
/// This uses the `open` crate to make things easier.
pub fn open_executable<S: AsRef<str>>(path: S, args: Option<String>) -> Result<OpenResult> {
    // Store the current working directory.
    let cwd = std::env::current_dir()?;

    // Resolve the path to the executable.
    let executable = resolve_path(path)?;

    // Change the current working directory to the executable's directory.
    let mut folder = executable.clone();
    folder.pop();

    std::env::set_current_dir(folder)?;

    // Open the executable.
    if let Err(_) = open::that(format!(
        "{} {}",
        executable.to_string_lossy(),
        args.unwrap_or_default()
    )) {
        return Ok(OpenResult::Failed);
    }

    // Restore the original working directory.
    std::env::set_current_dir(cwd)?;

    Ok(OpenResult::Success)
}

/// Checks if the process is running.
pub fn find_process<S: AsRef<str>>(process_name: S) -> bool {
    let mut system = System::new();
    system.refresh_all();

    let process_name = process_name.as_ref().as_ref();
    let mut processes = system.processes_by_exact_name(process_name);
    processes.next().is_some()
}

/// Canonicalizes a path using the system's path rules.
///
/// In addition, this resolves any symbolic links or relative paths.
pub fn canonicalize<S: AsRef<str>>(path: S) -> Result<String> {
    let path = PathBuf::from(path.as_ref());
    let path = path.canonicalize()?
        .to_string_lossy()
        .trim()
        .to_string();

    #[cfg(windows)]
    {
        // This trims the '\\?\' prefix from the path.
        return Ok(path[4..].to_string());
    }

    #[cfg(unix)]
    {
        return Ok(path);
    }
}

#[cfg(windows)]
pub trait AsCString {
    /// Converts the current string into a `CString`.
    fn as_cstring(&self) -> CString;
}

#[cfg(windows)]
impl AsCString for str {
    fn as_cstring(&self) -> CString {
        CString::new(self).unwrap()
    }
}
