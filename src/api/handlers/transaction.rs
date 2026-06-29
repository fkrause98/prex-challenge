use crate::api::models::transaction::{
    NewCreditTransactionRequest, NewCreditTransactionResponse, NewDebitTransactionRequest,
    NewDebitTransactionResponse,
};
use crate::api::validated::Validated;
use crate::state::AppState;
use actix_web::{Result, post, web};

/// Handler for 'new_credit_transaction' endpoint
#[post("/new_credit_transaction")]
pub async fn new_credit_transaction(
    state: web::Data<AppState>,
    payload: Validated<NewCreditTransactionRequest>,
) -> Result<web::Json<NewCreditTransactionResponse>> {
    let new_balance = state
        .accounts
        .credit(payload.0.client_id, payload.0.credit_amount)?
        .ok_or_else(|| actix_web::error::ErrorNotFound(crate::api::error::ApiError::not_found("Client not found")))?;
    Ok(web::Json(NewCreditTransactionResponse {
        client_id: payload.0.client_id,
        new_balance,
    }))
}

/// Handler for 'new_debit_transaction' endpoint
#[post("/new_debit_transaction")]
pub async fn new_debit_transaction(
    state: web::Data<AppState>,
    payload: Validated<NewDebitTransactionRequest>,
) -> Result<web::Json<NewDebitTransactionResponse>> {
    let new_balance = state
        .accounts
        .debit(payload.0.client_id, payload.0.debit_amount)?
        .ok_or_else(|| actix_web::error::ErrorNotFound(crate::api::error::ApiError::not_found("Client not found")))?;
    Ok(web::Json(NewDebitTransactionResponse {
        client_id: payload.0.client_id,
        new_balance,
    }))
}
