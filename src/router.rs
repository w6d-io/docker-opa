use std::sync::Arc;

use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tower_http::request_id::RequestId;

use crate::{
    config::Opa,
    controller::evaluate,
    error::Router,
    utils::error::Produce, // utils::telemetry::gather_telemetry,
};

#[derive(Debug, Deserialize)]
pub struct PayloadGuard {
    input: Box<RawValue>,
    data: Box<RawValue>,
}

///this route Deserialize the data and evalute de data agaisnt the rego
#[tracing::instrument]
#[axum_macros::debug_handler]
pub async fn eval_rego(
    request_id: Extension<RequestId>,
    State(config): State<Arc<RwLock<Opa>>>,
    Json(data): Json<PayloadGuard>,
) -> Result<String, Router> {
    let eval = match evaluate(data.input, data.data, config.clone()).await {
        Ok(ev) => ev,
        Err(e) => {
            e.send(config).await;
            return Err(e);
        }
    };
    Ok(eval)
}

/* ///route for prometheus telemetry
#[get("/metrics")]
pub async fn handle_metrics() -> String {
    gather_telemetry().await
} */

///route for prometheus telemetry
#[tracing::instrument]
#[axum_macros::debug_handler]
pub async fn alive() -> Result<(), Router> {
    Ok(())
}

///route for prometheus telemetry
#[tracing::instrument]
#[axum_macros::debug_handler]
pub async fn ready() -> Result<(), Router> {
    Ok(())
}
