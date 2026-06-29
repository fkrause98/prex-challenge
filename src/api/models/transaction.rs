use anyhow::{Result, ensure};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::validated::Validate;

/// Entity representing a request for the endpoint 'new_credit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewCreditTransactionRequest {
    pub client_id: u64,
    pub credit_amount: Decimal,
}

/// Entity representing a response for the endpoint 'new_credit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewCreditTransactionResponse {
    pub client_id: u64,
    pub new_balance: Decimal,
}

impl Validate for NewCreditTransactionRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.client_id > 0,
            ApiError::bad_request("Client ID must be greater than zero")
        );

        ensure!(
            self.credit_amount > Decimal::ZERO,
            ApiError::bad_request("Credit amount must be greater than zero")
        );

        Ok(())
    }
}

/// Entity representing a request for the endpoint 'new_debit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewDebitTransactionRequest {
    pub client_id: u64,
    pub debit_amount: Decimal,
}

/// Entity representing a response for the endpoint 'new_debit_transaction'.
#[derive(Serialize, Deserialize)]
pub struct NewDebitTransactionResponse {
    pub client_id: u64,
    pub new_balance: Decimal,
}

impl Validate for NewDebitTransactionRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            self.client_id > 0,
            ApiError::bad_request("Client ID must be greater than zero")
        );

        ensure!(
            self.debit_amount > Decimal::ZERO,
            ApiError::bad_request("Debit amount must be greater than zero")
        );

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_valid_credit_request() -> NewCreditTransactionRequest {
        NewCreditTransactionRequest {
            client_id: 1,
            credit_amount: Decimal::new(10050, 2),
        }
    }

    #[test]
    fn test_credit_successful_validation() {
        let req = get_valid_credit_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_credit_zero_client_id_fails() {
        let mut req = get_valid_credit_request();
        req.client_id = 0;

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Client ID must be greater than zero")
        );
    }

    #[test]
    fn test_credit_zero_amount_fails() {
        let mut req = get_valid_credit_request();
        req.credit_amount = Decimal::ZERO;

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Credit amount must be greater than zero")
        );
    }

    fn get_valid_debit_request() -> NewDebitTransactionRequest {
        NewDebitTransactionRequest {
            client_id: 1,
            debit_amount: Decimal::new(10050, 2),
        }
    }

    #[test]
    fn test_debit_successful_validation() {
        let req = get_valid_debit_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_debit_zero_client_id_fails() {
        let mut req = get_valid_debit_request();
        req.client_id = 0;

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Client ID must be greater than zero")
        );
    }

    #[test]
    fn test_debit_zero_amount_fails() {
        let mut req = get_valid_debit_request();
        req.debit_amount = Decimal::ZERO;

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Debit amount must be greater than zero")
        );
    }
}
