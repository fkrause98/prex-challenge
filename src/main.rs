use actix_web::{App, HttpResponse, HttpServer, middleware, web::{self, Data}};
use challenge_prex::{AppState, new_client, client_balance};
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init_from_env(
        env_logger::Env::default().default_filter_or("info")
    );
    HttpServer::new(|| {
        // General error for bad json requests
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

        // TODO: Maybe rebuild known accounts from file
        let state = Data::new(AppState { accounts: Default::default() });

        App::new()
            .app_data(state)
            .app_data(json_cfg)
            .service(new_client)
            .service(client_balance)
            .wrap(middleware::Logger::default())

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
