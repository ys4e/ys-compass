use std::sync::MutexGuard;
use log::warn;
use crate::config::Config;
use crate::system;
use crate::utils::MaybeError;

#[cfg(windows)]
use crate::{sys_str, system::AsCString};
#[cfg(windows)]
use windows::Win32::{Foundation::HANDLE, System::Threading::LPTHREAD_START_ROUTINE};

/// Utility method to check if the game is currently running.
///
/// In the event of any errors, this will return `false`.
#[tauri::command]
pub fn game__is_open() -> bool {
    let config = Config::get();
    system::find_process(&config.game.get_executable_name())
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
pub fn game__launch() -> MaybeError<()> {
    // Check if the game process is already running.
    if game__is_open() {
        return Err("game.error.already-open");
    }

    // Open the configuration.
    let config = Config::get();

    // Launch the game.
    launch_game(config)
}

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
fn launch_game(config: MutexGuard<'_, Config>) -> MaybeError<()> {
    use log::warn;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::Threading::ResumeThread;

    let game_config = &config.game;

    // 1. Launch the game and obtain handles.
    let (thread, process) = open_game(game_config.path.clone())?;

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
            Err(_) => return Err("game.error.launch.unknown")
        };

        let load_library = "LoadLibraryA".as_cstring();
        match GetProcAddress(kernel, sys_str!(load_library)) {
            Some(ptr) => std::mem::transmute::<_, LPTHREAD_START_ROUTINE>(ptr),
            None => return Err("game.error.launch.dll-fail")
        }
    };

    if !disable_ac {
        unsafe { suspend(&process)?; }
    }

    // Inject all DLLs in the configuration.
    for modification in config.game.modifications() {
        unsafe {
            match modification.to_path() {
                Ok(path) => inject_dll(&process, load_library, path)?,
                Err(_) => warn!("{}", t!("backend.path.error.modification"))
            }
        }
    }

    if !disable_ac {
        unsafe { resume(&process)?; }
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
        Err(_) => return Err("game.error.launch.unknown")
    };

    let suspend = "NtSuspendProcess".as_cstring();
    let ptr = match GetProcAddress(nt_module, sys_str!(suspend)) {
        Some(ptr) => ptr,
        None => return Err("game.error.launch.unknown")
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
        Err(_) => return Err("game.error.launch.unknown")
    };

    let resume = "NtResumeProcess".as_cstring();
    let ptr = match GetProcAddress(nt_module, sys_str!(resume)) {
        Some(ptr) => ptr,
        None => return Err("game.error.launch.unknown")
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
fn open_game(path: String) -> Result<(HANDLE, HANDLE), &'static str> {
    use sysinfo::System;
    use std::mem::size_of;
    use windows::Win32::Foundation::HANDLE;

    // Get token to open process.
    let token = unsafe {
        use windows::Win32::Security::TOKEN_ALL_ACCESS;
        use windows::Win32::System::Threading::{OpenProcessToken, GetCurrentProcess};

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
            Err(_) => return Err("game.error.launch.no-parent")
        }
    };

    if explorer.is_invalid() {
        return Err("game.error.launch.no-parent");
    }

    unsafe {
        use std::ffi::CString;
        use windows::core::PSTR;
        use windows::Win32::System::Threading::{
            PROCESS_INFORMATION,
            STARTUPINFOEXA,
            STARTUPINFOA,
            CreateProcessAsUserA,
            EXTENDED_STARTUPINFO_PRESENT
        };

        let mut process_info: PROCESS_INFORMATION = Default::default();
        let mut start_info = STARTUPINFOEXA {
            StartupInfo: STARTUPINFOA {
                cb: size_of::<STARTUPINFOEXA>() as u32,
                ..Default::default()
            },
            lpAttributeList: Default::default()
        };

        // Create the process.
        let Ok(path) = system::canonicalize(path) else {
            return Err("game.error.launch.bad-path");
        };

        let path = path.as_cstring();
        let arguments = CString::new("--insecure --verbose --console").unwrap();

        let result = CreateProcessAsUserA(
            Some(token),
            sys_str!(path),
            Some(PSTR(arguments.as_ptr() as *mut u8)),
            None, None,
            false,
            EXTENDED_STARTUPINFO_PRESENT,
            None, None,
            &mut start_info as *mut _ as *mut _,
            &mut process_info
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
    use std::{ffi::c_void, mem::{size_of, size_of_val}, ptr};
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
        for i in 0..driver_count {
            let mut name = [0u8; 256];
            let size = GetDeviceDriverBaseNameA(drivers[i], &mut name);

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
unsafe fn inject_dll(process: &HANDLE, load_library: LPTHREAD_START_ROUTINE, dll_path: String) -> MaybeError<()> {
    use windows::Win32::Foundation::{CloseHandle, WAIT_OBJECT_0};
    use windows::Win32::System::Diagnostics::Debug::WriteProcessMemory;
    use windows::Win32::System::Threading::{CreateRemoteThread, WaitForSingleObject};
    use windows::Win32::System::Memory::{VirtualAllocEx, VirtualFreeEx, MEM_RESERVE, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE};

    let path_length = dll_path.len() + 1;

    let dll_path = dll_path.as_cstring();
    let path = sys_str!(dll_path);

    // Allocate memory for the thread to access the DLL path.
    let dll_path = VirtualAllocEx(
        *process,
        None,
        path_length,
        MEM_RESERVE | MEM_COMMIT,
        PAGE_READWRITE
    );

    // Write the DLL path to the process.
    if let Err(_) = WriteProcessMemory(
        *process,
        dll_path,
        path.as_ptr() as *const _,
        path_length,
        None
    ) {
        return Err("game.error.launch.dll-fail");
    };

    // Invoke the LoadLibrary function.
    let Ok(thread) = CreateRemoteThread(
        *process,
        None,
        0,
        load_library,
        Some(dll_path),
        0, None
    ) else {
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
