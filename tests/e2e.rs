use actix_web::{test, web, App};
use challenge_prex::state::AppState;
use challenge_prex::store::AccountStore;
use challenge_prex::api::{
    client_balance::{client_balance, ClientBalanceResponse},
    new_client::{new_client, NewClientRequest, NewClientResponse},
    new_credit_transaction::{new_credit_transaction, NewCreditTransactionRequest, NewCreditTransactionResponse},
    new_debit_transaction::{new_debit_transaction, NewDebitTransactionRequest, NewDebitTransactionResponse},
    store_balances::{store_balances, StoreBalancesResponse},
};
use chrono::Utc;

#[actix_web::test]
async fn test_create_account_and_fetch_balance() {
    let state = web::Data::new(AppState {
        accounts: AccountStore::default(),
        file_counter: std::sync::Mutex::new(1),
    });

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .service(new_client)
            .service(client_balance)
            .service(new_credit_transaction)
            .service(new_debit_transaction)
            .service(store_balances),
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
    
    assert!(!client_id.is_empty(), "Client ID should not be empty");

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?client_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.balance, rust_decimal::Decimal::ZERO);

    // First store: should be file #1, balance 0
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp1: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(resp1.filename.ends_with("_1.DAT"));
    let file_content1 = std::fs::read_to_string(&resp1.filename).expect("Failed to read file 1");
    assert_eq!(file_content1, format!("{} {}\n", client_id, rust_decimal::Decimal::ZERO));
    std::fs::remove_file(&resp1.filename).unwrap_or_default();

    let credit_amount = rust_decimal::Decimal::new(15050, 2); // 150.50
    let credit_req = NewCreditTransactionRequest {
        client_id: client_id.clone(),
        credit_amount,
    };

    let req = test::TestRequest::post()
        .uri("/new_credit_transaction")
        .set_json(&credit_req)
        .to_request();

    let resp: NewCreditTransactionResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.new_balance, credit_amount);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?client_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.balance, credit_amount);

    // Second store: should be file #2, balance 150.50
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp2: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(resp2.filename.ends_with("_2.DAT"));
    let file_content2 = std::fs::read_to_string(&resp2.filename).expect("Failed to read file 2");
    assert_eq!(file_content2, format!("{} {}\n", client_id, credit_amount));
    std::fs::remove_file(&resp2.filename).unwrap_or_default();

    // Debit transaction
    let debit_amount = rust_decimal::Decimal::new(5025, 2); // 50.25
    let expected_balance = credit_amount - debit_amount;
    
    let debit_req = NewDebitTransactionRequest {
        client_id: client_id.clone(),
        debit_amount,
    };

    let req = test::TestRequest::post()
        .uri("/new_debit_transaction")
        .set_json(&debit_req)
        .to_request();

    let resp: NewDebitTransactionResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.client_id, client_id);
    assert_eq!(resp.new_balance, expected_balance);

    let req = test::TestRequest::get()
        .uri(&format!("/client_balance?client_id={}", client_id))
        .to_request();

    let resp: ClientBalanceResponse = test::call_and_read_body_json(&app, req).await;
    assert_eq!(resp.balance, expected_balance);

    // Third store: should be file #3, expected_balance
    let req = test::TestRequest::post()
        .uri("/store_balances")
        .to_request();
    let resp3: StoreBalancesResponse = test::call_and_read_body_json(&app, req).await;
    assert!(resp3.filename.ends_with("_3.DAT"));
    
    let file_content3 = std::fs::read_to_string(&resp3.filename).expect("Failed to read file 3");
    assert_eq!(file_content3, format!("{} {}\n", client_id, expected_balance));

    std::fs::remove_file(&resp3.filename).unwrap_or_default();
}
