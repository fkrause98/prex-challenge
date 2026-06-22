use actix_web::{App, HttpResponse, HttpServer, middleware, web::{self, Data}};
use challenge_prex::state::AppState;
use challenge_prex::api::{
    new_client::new_client,
    client_balance::client_balance,
    new_credit_transaction::new_credit_transaction,
    new_debit_transaction::new_debit_transaction,
    store_balances::store_balances
};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info")
    );
    HttpServer::new(|| {
        let json_cfg = web::JsonConfig::default()
            .error_handler(|err, _req| {
                let response = HttpResponse::BadRequest().json(json!({
                    "error": {
                        // TODO: Maybe improve this
                        "status": 400,
                        "message": err.to_string()
                    }
                }));
                actix_web::error::InternalError::from_response(err, response).into()
            });

        let state = Data::new(AppState { 
            accounts: Default::default(),
            file_counter: std::sync::Mutex::new(1),
        });

        App::new()
            .app_data(state)
            .app_data(json_cfg)
            .service(new_client)
            .service(client_balance)
            .service(new_credit_transaction)
            .service(new_debit_transaction)
            .service(store_balances)
            .wrap(middleware::Logger::default())

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
