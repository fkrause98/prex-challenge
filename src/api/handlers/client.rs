use actix_web::{get, post, web, Result};
use crate::api::error::ApiError;
use crate::api::models::client::{
    ClientBalanceRequest, ClientBalanceResponse, NewClientRequest, NewClientResponse,
    StoreBalancesResponse,
};
use crate::api::validated::Validated;
use crate::state::AppState;

#[post("/new_client")]
pub async fn new_client(
    state: web::Data<AppState>,
    payload: Validated<NewClientRequest>,
) -> Result<web::Json<NewClientResponse>> {
    let client_id = state.accounts.create_client(payload.0.document_number)?;
    Ok(web::Json(NewClientResponse { client_id: client_id.into() }))
}

#[get("/client_balance")]
pub async fn client_balance(
    state: web::Data<AppState>,
    query: web::Query<ClientBalanceRequest>,
) -> Result<web::Json<ClientBalanceResponse>> {
    let req = query.into_inner();

    if let Err(e) = crate::api::validated::Validate::validate(&req) {
        return Err(actix_web::error::ErrorBadRequest(e));
    }

    let ids = state.accounts.ids.lock().unwrap();
    let document_number = ids
        .get(&req.client_id)
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorNotFound(
            ApiError::bad_request("Client not found")
        ))?;

    let balances = state.accounts.balances.lock().unwrap();
    let balance = balances
        .get(&document_number)
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorNotFound(
            ApiError::bad_request("Balance not found for client")
        ))?;

    Ok(web::Json(ClientBalanceResponse {
        client_id: req.client_id,
        balance,
    }))
}

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
