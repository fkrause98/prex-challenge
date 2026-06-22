use anyhow::{ensure, Result};
use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::validated::Validate;

/// Entity representing a request for the endpoint 'new_client_request'.
#[derive(Serialize, Deserialize)]
pub struct NewClientRequest {
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub document_number: String,
    pub country: String,
}

/// Entity representing a response for the endpoint 'new_client_request'.
#[derive(Serialize, Deserialize)]
pub struct NewClientResponse {
    pub client_id: String,
}

impl Validate for NewClientRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.client_name.trim().is_empty(),
            ApiError::bad_request("Client name cannot be empty")
        );

        let today = Utc::now().date_naive();
        ensure!(
            self.birth_date < today,
            ApiError::bad_request("Birth date must be in the past")
        );

        ensure!(
            !self.document_number.trim().is_empty(),
            ApiError::bad_request("Document number cannot be empty")
        );

        ensure!(
            self.country.trim().len() == 2,
            ApiError::bad_request("Country must be a valid 2-letter ISO code")
        );

        Ok(())
    }
}

/// Entity representing a request for the endpoint 'client_balance'.
#[derive(Serialize, Deserialize)]
pub struct ClientBalanceRequest {
    pub client_id: String,
}

/// Entity representing a response for the endpoint 'client_balance'.
#[derive(Serialize, Deserialize)]
pub struct ClientBalanceResponse {
    pub client_id: String,
    pub balance: rust_decimal::Decimal,
}

impl Validate for ClientBalanceRequest {
    fn validate(&self) -> Result<()> {
        ensure!(
            !self.client_id.trim().is_empty(),
            ApiError::bad_request("Client ID cannot be empty")
        );

        Ok(())
    }
}

/// Entity representing a response for the endpoint 'store_balances'.
#[derive(Serialize, Deserialize)]
pub struct StoreBalancesResponse {
    pub filename: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn get_valid_new_client_request() -> NewClientRequest {
        NewClientRequest {
            client_name: "John Doe".to_string(),
            birth_date: Utc::now().date_naive() - Duration::days(365 * 20),
            document_number: "ABC-123456".to_string(),
            country: "US".to_string(),
        }
    }

    #[test]
    fn test_new_client_successful_validation() {
        let req = get_valid_new_client_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_empty_client_name_fails() {
        let mut req = get_valid_new_client_request();
        req.client_name = "   ".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Client name cannot be empty"));
    }

    #[test]
    fn test_future_birth_date_fails() {
        let mut req = get_valid_new_client_request();
        req.birth_date = Utc::now().date_naive() + Duration::days(1);

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Birth date must be in the past"));
    }

    #[test]
    fn test_today_birth_date_fails() {
        let mut req = get_valid_new_client_request();
        req.birth_date = Utc::now().date_naive();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Birth date must be in the past"));
    }

    #[test]
    fn test_empty_document_number_fails() {
        let mut req = get_valid_new_client_request();
        req.document_number = "".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Document number cannot be empty"));
    }

    #[test]
    fn test_invalid_country_code_length_fails() {
        let mut req = get_valid_new_client_request();

        req.country = "USA".to_string();
        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Country must be a valid 2-letter"));

        req.country = "U".to_string();
        let result2 = req.validate();
        assert!(result2.is_err());
    }

    fn get_valid_balance_request() -> ClientBalanceRequest {
        ClientBalanceRequest {
            client_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
        }
    }

    #[test]
    fn test_balance_successful_validation() {
        let req = get_valid_balance_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_empty_client_id_fails() {
        let mut req = get_valid_balance_request();
        req.client_id = "".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Client ID cannot be empty"));
    }

    #[test]
    fn test_whitespace_only_client_id_fails() {
        let mut req = get_valid_balance_request();
        req.client_id = "   ".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Client ID cannot be empty"));
    }
}
