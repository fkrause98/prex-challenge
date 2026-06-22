use serde::{Deserialize, Serialize};

/// Entity representing a response for the endpoint 'store_balances'.
#[derive(Serialize, Deserialize)]
pub struct StoreBalancesResponse {
    pub filename: String,
}

use actix_web::{post, web, Result};
use crate::state::AppState;

#[post("/store_balances")]
pub async fn store_balances(
    state: web::Data<AppState>,
) -> Result<web::Json<StoreBalancesResponse>> {
    let mut counter_guard = state.file_counter.lock().unwrap();
    let counter = *counter_guard;
    
    let now = chrono::Local::now();
    let filename = format!("{}_{}.DAT", now.format("%d%m%Y"), counter);

    let mut content = String::new();
    {
        let ids = state.accounts.ids.lock().unwrap();
        let balances = state.accounts.balances.lock().unwrap();
        
        for (client_id, doc_num) in ids.iter() {
            if let Some(balance) = balances.get(doc_num) {
                content.push_str(&format!("{} {}\n", client_id, balance));
            }
        }
    }

    std::fs::write(&filename, content).map_err(actix_web::error::ErrorInternalServerError)?;

    *counter_guard += 1;

    Ok(web::Json(StoreBalancesResponse { filename }))
}
