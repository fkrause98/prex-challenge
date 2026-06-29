use std::path::PathBuf;

use crate::store::AccountStore;

/// The API Server global stat, shared across the actix endpoints.
pub struct AppState {
    pub accounts: AccountStore,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            accounts: AccountStore::default(),
        }
    }
}

impl AppState {
    /// Builds an AppState with the AccountStore initialized with an export directory
    pub fn with_export_dir(export_dir: impl Into<PathBuf>) -> Self {
        Self {
            accounts: AccountStore::with_output_dir(export_dir),
        }
    }
}
