use anyhow::ensure;
use anyhow::Result;
use chrono::NaiveDate;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::error::ApiError;
use super::validated::Validate;

/// Entity representing a request for the endpoint 'new_client_request'.
#[derive(Serialize, Deserialize)]
pub struct NewClientRequest {
    client_name: String,
    birth_date: NaiveDate,
    // TODO: Make sure this is unique and maybe follow some format.
    document_number: String,
    // TODO: Make sure this is unique and maybe follow some format.
    country: String,
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
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    fn get_valid_request() -> NewClientRequest {
        NewClientRequest {
            client_name: "John Doe".to_string(),
            birth_date: Utc::now().date_naive() - Duration::days(365 * 20),
            document_number: "ABC-123456".to_string(),
            country: "US".to_string(),
        }
    }

    #[test]
    fn test_successful_validation() {
        let req = get_valid_request();
        assert!(req.validate().is_ok());
    }

    #[test]
    fn test_empty_client_name_fails() {
        let mut req = get_valid_request();
        req.client_name = "   ".to_string();

        let result = req.validate();
        assert!(result.is_err());

        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Client name cannot be empty"));
    }

    #[test]
    fn test_future_birth_date_fails() {
        let mut req = get_valid_request();
        req.birth_date = Utc::now().date_naive() + Duration::days(1);

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Birth date must be in the past"));
    }

    #[test]
    fn test_today_birth_date_fails() {
        let mut req = get_valid_request();
        req.birth_date = Utc::now().date_naive();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Birth date must be in the past"));
    }

    #[test]
    fn test_empty_document_number_fails() {
        let mut req = get_valid_request();
        req.document_number = "".to_string();

        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Document number cannot be empty"));
    }

    #[test]
    fn test_invalid_country_code_length_fails() {
        let mut req = get_valid_request();

        req.country = "USA".to_string();
        let result = req.validate();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Country must be a valid 2-letter"));

        req.country = "U".to_string();
        let result2 = req.validate();
        assert!(result2.is_err());
    }
}
