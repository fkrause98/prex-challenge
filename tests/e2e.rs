use actix_web::{App, test, web};
use challenge_prex::api::models::client::{
    ClientBalanceResponse, NewClientRequest, NewClientResponse, StoreBalancesResponse,
};
use challenge_prex::api::models::transaction::{
    NewCreditTransactionRequest, NewCreditTransactionResponse, NewDebitTransactionRequest,
    NewDebitTransactionResponse,
};
use challenge_prex::state::AppState;
use chrono::Utc;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Creates a fresh AppState that writes DAT files into an OS-provided temp
/// directory. Each test gets its own isolated directory so parallel test runs
/// don't interfere with each other.
fn create_test_state() -> web::Data<AppState> {
    web::Data::new(AppState::default())
}

/// Creates a fresh AppState with a caller-supplied export directory.
fn create_test_state_with_dir(dir: &std::path::Path) -> web::Data<AppState> {
    web::Data::new(AppState::with_export_dir(dir))
}

// ---------------------------------------------------------------------------
// Core flow tests
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_create_account_and_fetch_balance() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let new_client_req = NewClientRequest {
        client_name: "Test User".to_string(),
        birth_date: Utc::now().date_naive() - chrono::Duration::days(365 * 20),
        document_number: "TEST-1234".to_string(),
        country: "AR".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&new_client_req)
        .to_request();

    let resp: NewClientResponse = test::call_and_read_body_json(&app, req).await;
    let client_id = resp.client_id;

    assert!(client_id > 0, "Client ID should be a positive integer");

    // Fetch initial balance (should be 0)
    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?user_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.client_name, "Test User");
    assert_eq!(resp.document_number, "TEST-1234");
    assert_eq!(resp.country, "AR");
    assert_eq!(resp.balance, rust_decimal::Decimal::ZERO);

    // Credit
    let credit_amount = rust_decimal::Decimal::new(15050, 2); // 150.50
    let credit_req = NewCreditTransactionRequest {
        client_id,
        credit_amount,
    };

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(&credit_req)
        .to_request();

    let resp: NewCreditTransactionResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.new_balance, credit_amount);

    // Verify balance after credit
    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?user_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.balance, credit_amount);

    // Debit
    let debit_amount = rust_decimal::Decimal::new(5025, 2); // 50.25
    let expected_balance = credit_amount - debit_amount;

    let debit_req = NewDebitTransactionRequest {
        client_id,
        debit_amount,
    };

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(&debit_req)
        .to_request();

    let resp: NewDebitTransactionResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.new_balance, expected_balance);

    // Verify balance after debit
    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?user_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.balance, expected_balance);
}

#[actix_web::test]
async fn test_store_balances_resets_balance() {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let state = create_test_state_with_dir(tmp.path());

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let new_client_req = NewClientRequest {
        client_name: "Store Test User".to_string(),
        birth_date: Utc::now().date_naive() - chrono::Duration::days(365 * 25),
        document_number: "STORE-1234".to_string(),
        country: "AR".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&new_client_req)
        .to_request();

    let resp: NewClientResponse = test::call_and_read_body_json(&app, req).await;
    let client_id = resp.client_id;

    let credit_amount = rust_decimal::Decimal::new(20000, 2); // 200.00
    let credit_req = NewCreditTransactionRequest {
        client_id,
        credit_amount,
    };

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(&credit_req)
        .to_request();
    let _: NewCreditTransactionResponse = test::call_and_read_body_json(&app, req).await;

    // Trigger store_balances
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;

    assert!(resp.filename.ends_with("_1.DAT"));

    // Verify file contents
    let file_path = tmp.path().join(&resp.filename);
    let file_content = std::fs::read_to_string(&file_path).expect("Failed to read DAT file");
    assert_eq!(file_content, format!("{} {}\n", client_id, credit_amount));

    // Verify balance was reset to zero
    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?user_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.balance, rust_decimal::Decimal::ZERO);
}

// ---------------------------------------------------------------------------
// store_balances: counter increments across multiple calls
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_store_balances_counter_increments() {
    let tmp = tempfile::tempdir().expect("Failed to create temp dir");
    let state = create_test_state_with_dir(tmp.path());

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    // First call → counter = 1
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(
        resp.filename.ends_with("_1.DAT"),
        "Expected first file to end with _1.DAT, got: {}",
        resp.filename
    );

    // Second call → counter = 2
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(
        resp.filename.ends_with("_2.DAT"),
        "Expected second file to end with _2.DAT, got: {}",
        resp.filename
    );

    // Third call → counter = 3
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(
        resp.filename.ends_with("_3.DAT"),
        "Expected third file to end with _3.DAT, got: {}",
        resp.filename
    );
}

// ---------------------------------------------------------------------------
// Duplicate document number
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_duplicate_document_number_rejected() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let new_client_req = NewClientRequest {
        client_name: "First User".to_string(),
        birth_date: Utc::now().date_naive() - chrono::Duration::days(365 * 20),
        document_number: "UNIQUE-DOC".to_string(),
        country: "AR".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&new_client_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let duplicate_req = NewClientRequest {
        client_name: "Second User".to_string(),
        birth_date: Utc::now().date_naive() - chrono::Duration::days(365 * 30),
        document_number: "UNIQUE-DOC".to_string(),
        country: "UY".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&duplicate_req)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);
}

// ---------------------------------------------------------------------------
// new_client: HTTP-level validation failures
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_new_client_empty_name_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "   ",
            "birth_date": "1990-01-01",
            "document_number": "DOC-001",
            "country": "AR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_new_client_future_birth_date_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let future_date = Utc::now().date_naive() + chrono::Duration::days(1);

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "Future Person",
            "birth_date": future_date.to_string(),
            "document_number": "DOC-002",
            "country": "AR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_new_client_today_birth_date_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let today = Utc::now().date_naive();

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "Born Today",
            "birth_date": today.to_string(),
            "document_number": "DOC-003",
            "country": "AR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_new_client_empty_document_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "Valid Name",
            "birth_date": "1990-01-01",
            "document_number": "",
            "country": "AR"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_new_client_invalid_country_code_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    // 3-letter code should fail
    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "Valid Name",
            "birth_date": "1990-01-01",
            "document_number": "DOC-004",
            "country": "ARG"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // 1-letter code should also fail
    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({
            "client_name": "Valid Name",
            "birth_date": "1990-01-01",
            "document_number": "DOC-005",
            "country": "A"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_new_client_missing_fields_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    // Empty body
    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(serde_json::json!({}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ---------------------------------------------------------------------------
// new_credit_transaction: HTTP-level validation failures
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_credit_zero_amount_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(serde_json::json!({
            "client_id": 1,
            "credit_amount": "0"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_credit_zero_client_id_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(serde_json::json!({
            "client_id": 0,
            "credit_amount": "100.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_credit_nonexistent_client() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let credit_req = NewCreditTransactionRequest {
        client_id: 999,
        credit_amount: rust_decimal::Decimal::new(10000, 2),
    };

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(&credit_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

// ---------------------------------------------------------------------------
// new_debit_transaction: HTTP-level validation failures
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_debit_zero_amount_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(serde_json::json!({
            "client_id": 1,
            "debit_amount": "0"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_debit_zero_client_id_returns_400() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(serde_json::json!({
            "client_id": 0,
            "debit_amount": "50.00"
        }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

#[actix_web::test]
async fn test_debit_nonexistent_client() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let debit_req = NewDebitTransactionRequest {
        client_id: 999,
        debit_amount: rust_decimal::Decimal::new(10000, 2),
    };

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(&debit_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_debit_insufficient_funds() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let new_client_req = NewClientRequest {
        client_name: "Debit Test User".to_string(),
        birth_date: Utc::now().date_naive() - chrono::Duration::days(365 * 20),
        document_number: "DEBIT-FAIL".to_string(),
        country: "AR".to_string(),
    };

    let req = test::TestRequest::post()
        .uri("/new_client")
        .set_json(&new_client_req)
        .to_request();
    let resp: NewClientResponse = test::call_and_read_body_json(&app, req).await;
    let client_id = resp.client_id;

    let credit_req = NewCreditTransactionRequest {
        client_id,
        credit_amount: rust_decimal::Decimal::new(5000, 2), // 50.00
    };
    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(&credit_req)
        .to_request();
    let _: NewCreditTransactionResponse = test::call_and_read_body_json(&app, req).await;

    let debit_req = NewDebitTransactionRequest {
        client_id,
        debit_amount: rust_decimal::Decimal::new(10000, 2), // 100.00 > 50.00
    };

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(&debit_req)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);
}

// ---------------------------------------------------------------------------
// client_balance: error cases
// ---------------------------------------------------------------------------

#[actix_web::test]
async fn test_balance_nonexistent_client() {
    let state = create_test_state();

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .configure(challenge_prex::api::routes::configure),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/client_balance?user_id=999")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
