use std::sync::{LazyLock, Mutex};
use std::time::Instant;
use tauri::{AppHandle, WebviewUrl, WebviewWindowBuilder};
use crate::app::game;
use crate::capabilities::sniffer;
use crate::capabilities::sniffer::VisualPacket;
use crate::events;
use crate::events::Event;

/// This value holds whether the GUI-based sniffer is running or not.
static SNIFFER_RUNNING: LazyLock<Mutex<bool>> = LazyLock::new(|| Mutex::new(false));

/// Runs the packet sniffer asynchronously.
///
/// The sniffer will stop when the game is no longer detected.
#[tauri::command]
pub async fn sniffer__run(app_handle: AppHandle) -> Result<(), &'static str> {
    // If the sniffer is running, return early.
    if *SNIFFER_RUNNING.lock().unwrap() {
        return Ok(());
    }

    // Get the status listener.
    let mut listener = game::new_status_listener();

    // Run the sniffer itself.
    let (mut rx, shutdown_hook) = match sniffer::run_sniffer().await {
        Ok((rx, hook)) => (rx, hook),
        Err(_) => return Err("capability.sniffer.error")
    };

    // Create a thread for sending messages to the webview.
    let wv_listener = listener.clone();

    tokio::spawn(async move {
        let start_time = Instant::now();

        loop {
            // Check if the status has changed.
            if wv_listener.has_changed().unwrap() {
                // Read the status.
                let status = *wv_listener.borrow();
                // If the game is closed (the value is false)...
                if !status {
                    // ...stop the sniffer.
                    break;
                }
            }

            // Check if a packet is available.
            if let Ok(packet) = rx.try_recv() {
                // If so, push it to the webview through an event.
                let packet = VisualPacket::into_game(&packet, start_time);
                let event = Event::VisualizerPacket(packet);
                events::emit_event(&app_handle, event);
            }
        }
    });

    // Listen for the listener.
    tokio::spawn(async move {
        loop {
            let _ = listener.changed().await;

            // Check if the game is closed (the value is false)...
            if *listener.borrow() == false {
                // ...stop the sniffer.
                break;
            }
        }

        // Call the shutdown hook.
        if let Err(err) = shutdown_hook.send(()) {
            warn!("Failed to send shutdown signal: {}", err);
        }

        // Unset the sniffer value.
        *SNIFFER_RUNNING.lock().unwrap() = false;
    });

    // Set the sniffer value.
    *SNIFFER_RUNNING.lock().unwrap() = true;

    Ok(())
}

/// Opens the packet visualizer.
///
/// This opens a new webview window.
#[tauri::command]
pub async fn sniffer__open(app_handle: AppHandle) -> Result<(), &'static str> {
    // Create the webview window.
    let window = WebviewWindowBuilder::new(
        &app_handle, "visualizer",
        WebviewUrl::App("visualizer".into())
    )
        .title("Packet Visualizer")
        .build()
        .map_err(|err| {
            warn!("Failed to create visualizer window: {}", err);
            "Failed to open visualizer window."
        })?;

    // Show the window.
    window.show()
        .map_err(|_| "Failed to show visualizer window.")?;

    Ok(())
}
