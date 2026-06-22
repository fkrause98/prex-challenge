pub mod api;

use std::{collections::HashMap, sync::Mutex};

use actix_web::{Result, get, post, web};
use api::{
    client_balance::{ClientBalanceRequest, ClientBalanceResponse},
    new_client::{NewClientRequest, NewClientResponse},
    new_credit_transaction::{NewCreditTransactionRequest, NewCreditTransactionResponse},
    new_debit_transaction::{NewDebitTransactionRequest, NewDebitTransactionResponse},
    validated::Validated,
};
use api::error::ApiError;

#[derive(Default, Debug)]
pub struct AccountStore {
    pub balances: Mutex<HashMap<String, rust_decimal::Decimal>>,
    pub ids: Mutex<HashMap<String, String>>,
}

impl AccountStore {
    pub fn create_client(&self, document_number: String) -> Result<String, actix_web::Error> {
        use std::collections::hash_map::Entry;

        let mut balances = self.balances.lock().unwrap();
        
        match balances.entry(document_number.clone()) {
            Entry::Occupied(_) => Err(actix_web::error::ErrorConflict(
                ApiError::bad_request("Client already exists")
            )),
            Entry::Vacant(entry) => {
                let client_id: String = uuid::Uuid::new_v4().into();
                let mut ids = self.ids.lock().unwrap();
                ids.insert(client_id.clone(), document_number);
                entry.insert(rust_decimal::Decimal::ZERO);
                
                Ok(client_id)
            }
        }
    }

    pub fn credit(&self, client_id: &str, amount: rust_decimal::Decimal) -> Result<rust_decimal::Decimal, actix_web::Error> {
        let document_number = {
            let ids = self.ids.lock().unwrap();
            ids.get(client_id).ok_or_else(|| {
                actix_web::error::ErrorNotFound(ApiError::bad_request("Client not found"))
            })?.clone()
        };
        
        let mut balances = self.balances.lock().unwrap();
        let balance = balances.get_mut(&document_number).ok_or_else(|| {
            actix_web::error::ErrorNotFound(ApiError::bad_request("Balance not found for client"))
        })?;
        
        *balance += amount;
        Ok(*balance)
    }

    pub fn debit(&self, client_id: &str, amount: rust_decimal::Decimal) -> Result<rust_decimal::Decimal, actix_web::Error> {
        let document_number = {
            let ids = self.ids.lock().unwrap();
            ids.get(client_id).ok_or_else(|| {
                actix_web::error::ErrorNotFound(ApiError::bad_request("Client not found"))
            })?.clone()
        };
        
        let mut balances = self.balances.lock().unwrap();
        let balance = balances.get_mut(&document_number).ok_or_else(|| {
            actix_web::error::ErrorNotFound(ApiError::bad_request("Balance not found for client"))
        })?;
        
        if *balance < amount {
            return Err(actix_web::error::ErrorBadRequest(ApiError::bad_request("Insufficient funds")));
        }
        
        *balance -= amount;
        Ok(*balance)
    }
}

pub struct AppState {
    pub accounts: AccountStore,
}

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

    if let Err(e) = api::validated::Validate::validate(&req) {
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

#[post("/new_credit_transaction")]
pub async fn new_credit_transaction(
    state: web::Data<AppState>,
    payload: Validated<NewCreditTransactionRequest>,
) -> Result<web::Json<NewCreditTransactionResponse>> {
    let new_balance = state.accounts.credit(&payload.0.client_id, payload.0.credit_amount)?;
    Ok(web::Json(NewCreditTransactionResponse {
        client_id: payload.0.client_id,
        new_balance,
    }))
}

#[post("/new_debit_transaction")]
pub async fn new_debit_transaction(
    state: web::Data<AppState>,
    payload: Validated<NewDebitTransactionRequest>,
) -> Result<web::Json<NewDebitTransactionResponse>> {
    let new_balance = state.accounts.debit(&payload.0.client_id, payload.0.debit_amount)?;
    Ok(web::Json(NewDebitTransactionResponse {
        client_id: payload.0.client_id,
        new_balance,
    }))
}
