use tauri::Context;

/// Creates the context for the Tauri app.
/// 
/// This method is isolated as to not slow down IDE performance in `main.rs`.
pub(crate) fn build_context() -> Context {
    tauri::generate_context!()
}
