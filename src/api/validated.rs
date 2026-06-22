use actix_web::{dev::Payload, web, FromRequest, HttpRequest, Error};
use std::future::Future;
use std::pin::Pin;
use anyhow::Result;


pub trait Validate {
    fn validate(&self) -> Result<()>;
}

pub struct Validated<T>(pub T);

impl<T> FromRequest for Validated<T>
where
    T: serde::de::DeserializeOwned + Validate + 'static, 
{
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        let fut = web::Json::<T>::from_request(req, payload);

        Box::pin(async move {
            let json = fut.await?;
            let data = json.into_inner();

            if let Err(e) = data.validate() {
                return Err(actix_web::error::ErrorBadRequest(e));
            }

            Ok(Validated(data))
        })
    }
}
