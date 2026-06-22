use anyhow::ensure;
use anyhow::Result;
use serde::{Deserialize, Serialize};

use super::error::ApiError;
use super::validated::Validate;

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

#[cfg(test)]
mod tests {
    use super::*;

    fn get_valid_request() -> ClientBalanceRequest {
        ClientBalanceRequest {
            client_id: "550e8400-e29b-41d4-a716-446655440000".to_string(),
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
    fn test_whitespace_only_client_id_fails() {
        let mut req = get_valid_request();
        req.client_id = "   ".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Client ID cannot be empty"));
    }
}
