use actix_web::web;

pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(crate::api::handlers::client::new_client)
        .service(crate::api::handlers::client::client_balance)
        .service(crate::api::handlers::client::store_balances)
        .service(crate::api::handlers::transaction::new_credit_transaction)
        .service(crate::api::handlers::transaction::new_debit_transaction);
}
