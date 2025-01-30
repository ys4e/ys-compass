use tauri::AppHandle;

/// Closes the application.
#[tauri::command]
pub fn window__close(app_handle: AppHandle) {
    // TODO: Check configuration state to see if the launcher should minimize to the tray.
    app_handle.exit(0);
}
