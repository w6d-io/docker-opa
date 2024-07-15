use std::sync::Arc;

use axum::async_trait;
use serde::Serialize;
use tokio::sync::RwLock;
use tracing::error;

use crate::{config::Opa, utils::kafka};

///this structure represent an error sent to kafka
#[derive(Serialize)]
pub struct Data<'a> {
    code: &'a str,
    message: String,
}

///This trait add the send method to the type implementing it,
///allowing it to be sent to kafka
#[async_trait]
pub trait Produce: std::error::Error + Sync {
    ///send error to the given kafka topic
    #[cfg(not(tarpaulin_include))]
    async fn send(&self, config: Arc<RwLock<Opa>>) {
        let read_config = config.read().await;
        let config = &read_config.kafka;
        let Some(topic) = config.topics.get("error") else {
            error!("the topic was not found.");
            return;
        };
        let error = Data {
            code: "opa_error",
            message: self.to_string(),
        };
        if let Err(e) = kafka::send(config, topic, error) {
            error!("failed to send error data to kafka: {e}");
        }
    }
}
