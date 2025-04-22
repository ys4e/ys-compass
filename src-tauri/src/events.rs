use log::warn;
use tauri::{AppHandle, Emitter};
use crate::capabilities::sniffer::VisualPacket;

pub enum Event {
    LanguageChanged(String),
    VisualizerPacket(VisualPacket)
}

impl Event {
    /// Converts the enum into an event string used on the frontend.
    fn to_string(&self) -> &'static str {
        match self {
            Event::LanguageChanged(_) => "ysc://language/changed",
            Event::VisualizerPacket(_) => "ysc://visualizer/packet"
        }
    }

    /// Emits this event to the global app handle.
    pub fn send(&self, app_handle: &AppHandle) {
        if let Err(error) = match self {
            Event::LanguageChanged(language) => app_handle.emit(self.to_string(), language.to_string()),
            Event::VisualizerPacket(packet) => app_handle.emit(self.to_string(), packet.clone())
        } {
            warn!("{} {}", t!("backend.tauri.emit.error"), error);
        }
    }
}

/// Emits a global event to the window.
///
/// Requires using the event enum.
pub fn emit_event(app_handle: &AppHandle, event: Event) {
    event.send(app_handle);
}
