use anyhow::ensure;
use anyhow::Result;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use super::error::ApiError;
use super::validated::Validate;

/// Entity representing a request for the endpoint 'new_credit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewCreditTransactionRequest {
    pub client_id: String,
    pub credit_amount: Decimal,
}

/// Entity representing a response for the endpoint 'new_credit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewCreditTransactionResponse {
    pub client_id: String,
    pub new_balance: Decimal,
}

impl Validate for NewCreditTransactionRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.client_id.trim().is_empty(),
            ApiError::bad_request("Client ID cannot be empty")
        );

        ensure!(
            self.credit_amount > Decimal::ZERO,
            ApiError::bad_request("Credit amount must be greater than zero")
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_valid_request() -> NewCreditTransactionRequest {
        NewCreditTransactionRequest {
            client_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            credit_amount: Decimal::new(10050, 2),
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
        req.credit_amount = Decimal::ZERO;

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Credit amount must be greater than zero"));
    }
}
