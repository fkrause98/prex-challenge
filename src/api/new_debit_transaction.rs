use anyhow::ensure;
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::error::ApiError;
use super::validated::{Validate, Validated};

use actix_web::{post, web};
use crate::state::AppState;

/// Entity representing a request for the endpoint 'new_debit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewDebitTransactionRequest {
    pub client_id: String,
    pub debit_amount: Decimal,
}

/// Entity representing a response for the endpoint 'new_debit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewDebitTransactionResponse {
    pub client_id: String,
    pub new_balance: Decimal,
}

impl Validate for NewDebitTransactionRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.client_id.trim().is_empty(),
            ApiError::bad_request("Client ID cannot be empty")
        );

        ensure!(
            self.debit_amount > Decimal::ZERO,
            ApiError::bad_request("Debit amount must be greater than zero")
        );

        Ok(())
    }
}

#[post("/new_debit_transaction")]
pub async fn new_debit_transaction(
    state: web::Data<AppState>,
    payload: Validated<NewDebitTransactionRequest>,
) -> actix_web::Result<web::Json<NewDebitTransactionResponse>> {
    let new_balance = state.accounts.debit(&payload.0.client_id, payload.0.debit_amount)?;
    Ok(web::Json(NewDebitTransactionResponse {
        client_id: payload.0.client_id,
        new_balance,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_valid_request() -> NewDebitTransactionRequest {
        NewDebitTransactionRequest {
            client_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            debit_amount: Decimal::new(10050, 2),
        }
    }

    #[test]
    fn test_successful_validation() {
        let req = get_valid_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_empty_client_id_fails() {
        let mut req = get_valid_request();
        req.client_id = "".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Client ID cannot be empty"));
    }

    #[test]
    fn test_zero_amount_fails() {
        let mut req = get_valid_request();
        req.debit_amount = Decimal::ZERO;

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Debit amount must be greater than zero"));
    }
}
