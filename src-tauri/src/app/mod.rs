use crate::events;
use crate::events::Event;
use tauri::AppHandle;

pub mod appearance;
pub mod game;

/// Sets the application language.
#[tauri::command]
pub fn set_language(app_handle: AppHandle, language: String) {
    // Set the language in the backend.
    rust_i18n::set_locale(&language);

    // Emit the language change event.
    events::emit_event(&app_handle, Event::LanguageChanged(language));
}
