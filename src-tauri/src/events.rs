use log::warn;
use tauri::{AppHandle, Emitter};

pub enum Event {
    LanguageChanged(String),
}

impl Event {
    /// Converts the enum into an event string used on the frontend.
    pub fn to_string(&self) -> &'static str {
        match self {
            Event::LanguageChanged(_) => "ysc://language/changed",
        }
    }

    /// Converts the enum into a payload string used on the frontend.
    ///
    /// If no payload was specified, a `()` will be returned.
    pub fn to_payload(&self) -> String {
        match self {
            Event::LanguageChanged(language) => language.to_string(),
        }
    }
}

/// Emits a global event to the window.
///
/// Requires using the event enum.
pub fn emit_event(app_handle: &AppHandle, event: Event) {
    if let Err(error) = app_handle.emit(event.to_string(), event.to_payload()) {
        warn!("{} {}", t!("backend.tauri.emit.error"), error);
    }
}
