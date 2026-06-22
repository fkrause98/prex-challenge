use actix_web::{test, web, App};
use challenge_prex::{
    api::{
        client_balance::ClientBalanceResponse,
        new_client::{NewClientRequest, NewClientResponse},
        new_credit_transaction::{NewCreditTransactionRequest, NewCreditTransactionResponse},
        new_debit_transaction::{NewDebitTransactionRequest, NewDebitTransactionResponse},
    },
    client_balance, new_client, new_credit_transaction, new_debit_transaction, AccountStore, AppState,
};
use chrono::Utc;

#[actix_web::test]
async fn test_create_account_and_fetch_balance() {
    let state = web::Data::new(AppState {
        accounts: AccountStore::default(),
    });

    let app = test::init_service(
        App::new()
            .app_data(state.clone())
            .service(new_client)
            .service(client_balance)
            .service(new_credit_transaction)
            .service(new_debit_transaction),
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
}
