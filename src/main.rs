use actix_web::{
    App, HttpResponse, HttpServer, middleware,
    web::{self, Data},
};
use challenge_prex::api::routes::configure;
use challenge_prex::state::AppState;
use serde_json::json;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup the logger, it will use 'info', by default.
    // Can be configured with the RUST_LOG env var.
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    // The 'state' of the API Server. It will be shared across all
    // the workers (threads) that run the HTTP server.
    let state = Data::new(AppState::default());

    HttpServer::new(move || {
        let json_cfg = web::JsonConfig::default().error_handler(|err, _req| {
            let response = HttpResponse::BadRequest().json(json!({
                "error": {
                    "message": err.to_string()
                }
            }));
            actix_web::error::InternalError::from_response(err, response).into()
        });

        App::new()
            .app_data(state.clone())
            .app_data(json_cfg)
            .configure(configure)
            .wrap(middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
