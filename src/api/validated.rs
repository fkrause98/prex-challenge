use actix_web::{Error, FromRequest, HttpRequest, dev::Payload, web};
use anyhow::Result;
use std::future::Future;
use std::pin::Pin;

/// Trait to validate API payloads.
/// Each entity representing an API request should implement this trait.
pub trait Validate {
    fn validate(&self) -> Result<()>;
}

/// Actix-web extractor to automatically validate JSON payloads through
/// the 'validated' trait.
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

impl<T> Validated<T> {
    pub fn payload(self) -> T {
        return self.0;
    }
}
