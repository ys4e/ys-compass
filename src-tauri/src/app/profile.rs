use tauri::State;
use crate::app::game::{GameManager, Profile};
use crate::GLOBAL_STATE;
use crate::state::SelectedProfile;
use crate::utils::MaybeError;

/// Fetches all available profiles.
#[tauri::command]
pub async fn profile__get_all() -> Vec<Profile> {
    let game_manager = GameManager::get().read().await;
    game_manager.profiles.clone()
}

/// Creates a new game profile.
#[tauri::command]
pub async fn profile__new_profile(
    state: State<'_, SelectedProfile>,
    profile: Profile,
) -> MaybeError<()> {
    // Save the profile.
    if let Err(error) = profile.save().await {
        warn!("Failed to save profile: {}", error);
        return Err("launcher.error.profile.unknown");
    };

    // Lock the selected profile.
    let mut selected_profile = state.0.lock().unwrap();

    // Check if an existing profile is set.
    let mut state = GLOBAL_STATE.write().unwrap();
    if state.selected_profile.is_none() || selected_profile.is_none() {
        // Set the persistent state.
        state.selected_profile = Some(profile.id.clone());
        state.save().ok();

        // Set the selected profile.
        *selected_profile = Some(profile);
    }

    Ok(())
}

/// Sets the selected profile by the given ID.
#[tauri::command]
pub async fn profile__set_profile(
    state: State<'_, SelectedProfile>,
    profile_id: String
) -> MaybeError<()> {
    // Get the game manager.
    let game_manager = GameManager::get().read().await;

    // Fetch the profile by its ID.
    let Some(profile) = game_manager.get_profile(&profile_id) else {
        // If the profile doesn't exist, return an error.
        return Err("launcher.error.profile.bad-id");
    };

    // Set the persisted state's selected profile.
    let mut persisted_state = GLOBAL_STATE.write().unwrap();
    persisted_state.selected_profile = Some(profile_id.clone());
    persisted_state.save().ok();

    // Set the app instance's selected profile.
    let mut selected_profile = state.0.lock().unwrap();
    *selected_profile = Some(profile);

    Ok(())
}
