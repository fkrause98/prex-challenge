use actix_web::{Result, get, post, web};

use crate::api::models::client::{
    ClientBalanceRequest, ClientBalanceResponse, NewClientRequest, NewClientResponse,
    StoreBalancesResponse,
};
use crate::api::validated::Validated;
use crate::state::AppState;

/// Handler for 'new_client' endpoint.
#[post("/new_client")]
pub async fn new_client(
    state: web::Data<AppState>,
    request_data: Validated<NewClientRequest>,
) -> Result<web::Json<NewClientResponse>> {
    let payload = request_data.payload();
    let client_id = state.accounts.create_client(
        payload.client_name,
        payload.birth_date,
        payload.document_number,
        payload.country,
    )?;

    Ok(web::Json(NewClientResponse { client_id }))
}

/// Handler for 'client_balance' endpoint.
#[get("/client_balance")]
pub async fn client_balance(
    state: web::Data<AppState>,
    query: web::Query<ClientBalanceRequest>,
) -> Result<web::Json<ClientBalanceResponse>> {
    let client = state
        .accounts
        .get_client(query.user_id)?
        .ok_or_else(|| actix_web::error::ErrorNotFound(crate::api::error::ApiError::not_found("Client not found")))?;
    Ok(web::Json(client.into()))
}

/// Handler for 'store_balances' endpoint.
#[post("/store_balances")]
pub async fn store_balances(
    state: web::Data<AppState>,
) -> Result<web::Json<StoreBalancesResponse>> {
    let filename = web::block(move || state.accounts.export())
        .await
        .map_err(actix_web::error::ErrorInternalServerError)??;

    Ok(web::Json(StoreBalancesResponse { filename }))
}
