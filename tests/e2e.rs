use actix_web::{test, web, App};
use challenge_prex::{
    api::{
        client_balance::ClientBalanceResponse,
        new_client::{NewClientRequest, NewClientResponse},
    },
    client_balance, new_client, AccountStore, AppState,
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
            .service(client_balance),
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
}
