use serde::{Deserialize, Serialize};

/// Entity representing a response for the endpoint 'store_balances'.
#[derive(Serialize, Deserialize)]
pub struct StoreBalancesResponse {
    pub filename: String,
}
