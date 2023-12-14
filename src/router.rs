use std::sync::Arc;

use axum::{extract::State, Extension, Json};
use serde::Deserialize;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tower_http::request_id::RequestId;

use crate::{
    config::OPAConfig,
    controller::evaluate,
    error::RouterError,
    utils::error::SendError, // utils::telemetry::gather_telemetry,
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
    State(config): State<Arc<RwLock<OPAConfig>>>,
    Json(data): Json<PayloadGuard>,
) -> Result<String, RouterError> {
    let eval = match evaluate(data.input, data.data, config.clone()).await {
        Ok(ev) => ev,
        Err(e) => {
            e.send_error(config).await;
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
pub async fn alive() -> Result<(), RouterError> {
    Ok(())
}

///route for prometheus telemetry
#[tracing::instrument]
#[axum_macros::debug_handler]
pub async fn ready() -> Result<(), RouterError> {
    Ok(())
}
