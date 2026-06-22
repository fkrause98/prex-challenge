pub mod api;
use std::{collections::HashMap, sync::Mutex};

use actix_web::{App, HttpResponse, HttpServer, Responder, Result, get, middleware, post, web::{self, Data}};
use api::{new_client::{NewClientRequest, NewClientResponse}, validated::Validated};
use serde_json::json;


#[derive(Default, Debug)]
pub struct AccountStore {
    pub balances: Mutex<HashMap<String, String>>,
    pub ids: Mutex<HashMap<String, String>>
}

pub struct AppState {
    pub accounts: AccountStore
}

#[post("/new_client")]
async fn new_client(state: web::Data<AppState>, payload: Validated<NewClientRequest>) -> Result<web::Json<NewClientResponse>> {
    let client_id: String = uuid::Uuid::new_v4().into();
    {
        // TODO: Abstract this into a function + struct
        let mut balances = state.accounts.balances.lock().unwrap();
        let mut ids = state.accounts.balances.lock().unwrap();
        ids.insert(client_id.clone(), payload.0.document_number.clone());
        balances.insert(payload.0.document_number.clone().to_owned(), "0".to_owned());
    }
    Ok(
        web::Json(NewClientResponse { client_id: client_id.into() })
    )
}

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
            .wrap(middleware::Logger::default())

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
