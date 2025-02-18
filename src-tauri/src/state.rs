use std::sync::Mutex;
use serde::{Deserialize, Serialize};
use anyhow::Result;
use tokio::sync::RwLockReadGuard;
use crate::app::game::{GameManager, Profile};
use crate::{utils, GLOBAL_STATE};

/// This state can be saved to the disk.
#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct PersistentState {
    /// This is the ID of the user's selected profile.
    pub selected_profile: Option<String>
}

impl PersistentState {
    /// Creates a new instance of the persistent state.
    pub fn new() -> Self {
        // Check if the state was saved to the disk.
        if let Ok(app_data_dir) = utils::app_data_dir() {
            let state_path = app_data_dir.join("state.json");
            if let Ok(state_file) = std::fs::read_to_string(&state_path) {
                return serde_json::from_str(&state_file).unwrap_or_default();
            }
        }

        // Otherwise, return the default state.
        PersistentState::default()
    }

    /// Saves the state to the disk.
    pub fn save(&self) -> Result<()> {
        // Get the application data directory.
        let app_data_dir = utils::app_data_dir()?;
        let serialized = serde_json::to_string(self)?;

        // Save the state to the disk.
        let state_path = app_data_dir.join("state.json");
        std::fs::write(&state_path, serialized)?;

        Ok(())
    }
}

/// A state used by Tauri.
pub struct SelectedProfile(pub Mutex<Option<Profile>>);

impl SelectedProfile {
    /// Creates a new instance of the selected profile state.
    pub fn new(game_manager: RwLockReadGuard<'_, GameManager>) -> Self {
        // Try getting the profile from the global state.
        let state = GLOBAL_STATE.read().unwrap();
        if let Some(state) = &state.selected_profile {
            if let Some(profile) = game_manager.get_profile(state) {
                return Self(Mutex::new(Some(profile)));
            }
        }

        Self(Mutex::new(None))
    }
}
