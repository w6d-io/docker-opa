use std::sync::Arc;

use axum::async_trait;
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::error;

use crate::{config::OPAConfig, utils::kafka::send_to_kafka};

#[derive(Serialize)]
pub struct ErrorData<'a> {
    code: &'a str,
    message: String,
}

#[async_trait]
pub trait SendError: std::error::Error + Sync {
    ///send error to the given kafka topic
    #[cfg(not(tarpaulin_include))]
    async fn send_error(&self, config: Arc<RwLock<OPAConfig>>) {
        let read_config = config.read().await;
        let config = &read_config.kafka;
        let error = ErrorData {
            code: "opa_error",
            message: self.to_string(),
        };
        if let Err(e) = send_to_kafka(config, "error", &error).await {
            error!("failed to send error data to kafka: {e}")
        }
    }
}
