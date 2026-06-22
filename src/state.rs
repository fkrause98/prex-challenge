use std::sync::Mutex;
use crate::store::AccountStore;

pub struct AppState {
    pub accounts: AccountStore,
    pub file_counter: Mutex<u64>,
}
