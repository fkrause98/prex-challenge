use std::collections::HashMap;
use std::sync::Mutex;
use actix_web::Result;
use rust_decimal::Decimal;

use crate::api::error::ApiError;

#[derive(Default, Debug)]
pub struct AccountStore {
    pub balances: Mutex<HashMap<String, Decimal>>,
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
                entry.insert(Decimal::ZERO);
                
                Ok(client_id)
            }
        }
    }

    pub fn credit(&self, client_id: &str, amount: Decimal) -> Result<Decimal, actix_web::Error> {
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

    pub fn debit(&self, client_id: &str, amount: Decimal) -> Result<Decimal, actix_web::Error> {
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
