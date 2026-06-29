use anyhow::{Result, ensure};
use chrono::{NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use crate::api::error::ApiError;
use crate::api::validated::Validate;
use crate::store::Client;

/// Entity representing a request for the endpoint 'new_client'.
#[derive(Serialize, Deserialize)]
pub struct NewClientRequest {
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub document_number: String,
    pub country: String,
}

/// Entity representing a response for the endpoint 'new_client'.
#[derive(Serialize, Deserialize)]
pub struct NewClientResponse {
    pub client_id: u64,
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
    pub user_id: u64,
}

/// Entity representing a response for the endpoint 'client_balance'.
#[derive(Serialize, Deserialize)]
pub struct ClientBalanceResponse {
    pub client_id: u64,
    pub client_name: String,
    pub birth_date: NaiveDate,
    pub document_number: String,
    pub country: String,
    pub balance: Decimal,
}

impl From<Client> for ClientBalanceResponse {
    fn from(client: Client) -> Self {
        Self {
            client_id: client.id,
            client_name: client.client_name,
            birth_date: client.birth_date,
            document_number: client.document_number,
            country: client.country,
            balance: client.balance,
        }
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Client name cannot be empty")
        );
    }

    #[test]
    fn test_future_birth_date_fails() {
        let mut req = get_valid_new_client_request();
        req.birth_date = Utc::now().date_naive() + Duration::days(1);

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Birth date must be in the past")
        );
    }

    #[test]
    fn test_today_birth_date_fails() {
        let mut req = get_valid_new_client_request();
        req.birth_date = Utc::now().date_naive();

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Birth date must be in the past")
        );
    }

    #[test]
    fn test_empty_document_number_fails() {
        let mut req = get_valid_new_client_request();
        req.document_number = "".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Document number cannot be empty")
        );
    }

    #[test]
    fn test_invalid_country_code_length_fails() {
        let mut req = get_valid_new_client_request();

        req.country = "USA".to_string();
        let result = req.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Country must be a valid 2-letter")
        );

        req.country = "U".to_string();
        let result2 = req.validate();
        assert!(result2.is_err());
    }
}
