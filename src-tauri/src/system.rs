use anyhow::Result;

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
