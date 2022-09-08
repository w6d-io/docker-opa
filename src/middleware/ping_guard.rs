use anyhow::anyhow;
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome, Request},
};

use crate::{config::CONFIG, utils::http};

pub struct Ready;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Ready {
    type Error = anyhow::Error;

    async fn from_request(_req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let http = CONFIG.read().await.http.clone();

        let url = match http.get("kratos") {
            Some(url) => url,
            None => {
                let e = anyhow!("address not found");
                error!("failed to get project address from config file: {}", e);
                return Outcome::Failure((Status::InternalServerError, e));
            }
        };

        match http::get(url).await {
            Err(e) => Outcome::Failure((Status::ServiceUnavailable, anyhow!(e))),
            Ok(_) => Outcome::Success(Ready),
        }
    }
}
