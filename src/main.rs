pub mod api;
use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, Result, middleware};
use api::{new_client::{NewClientRequest, NewClientResponse}, validated::Validated};
use serde_json::json;

#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/new_client")]
async fn new_client(client_info: Validated<NewClientRequest>) -> Result<web::Json<NewClientResponse>> {
    Ok(
        web::Json(NewClientResponse { client_id: "an_id".into() })
    )
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
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

        App::new()
            .app_data(json_cfg)
            .service(hello)
            .service(new_client)
            .route("/hey", web::get().to(manual_hello))
            .wrap(middleware::Logger::default())

    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
