use std::sync::Arc;

use anyhow::anyhow;
use log::error;
use rocket::{
    http::Status,
    request::{self, FromRequest, Outcome, Request},
    State,
};
use tokio::sync::RwLock;

use crate::{config::OPAConfig, utils::http};

pub struct Ready;

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Ready {
    type Error = anyhow::Error;

    async fn from_request(req: &'r Request<'_>) -> request::Outcome<Self, Self::Error> {
        let state = match req.guard::<&State<Arc<RwLock<OPAConfig>>>>().await {
            Outcome::Success(state) => state,
            Outcome::Failure(e) => {
                let error = anyhow!(e.0.reason().unwrap());
                return Outcome::Failure((e.0, error));
            }
            Outcome::Forward(f) => return Outcome::Forward(f),
        };
        let http = &state.read().await.http;

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
