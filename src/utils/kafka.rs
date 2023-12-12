#[allow(unused_imports)]
use anyhow::{anyhow, bail, Result};

use tracing::info;
use serde::Serialize;

use crate::config::Kafka;

///Send data to kafka.
#[cfg(not(tarpaulin_include))]
pub async fn send_to_kafka<T: Serialize>(_config: &Kafka, _topic: &str, data: T) -> Result<()> {
    let _message = kafka::KafkaMessage {
        headers: None,
        key: None,
        payload: serde_json::to_string(&data)?,
    };
    #[cfg(not(test))]
    match &_config.producers {
        Some(prod) => prod
            .get(_topic)
            .ok_or_else(|| anyhow!("failed to get asked kafka topic!"))?
            .produce(_message)?,
        None => bail!("topic not found"),
    }
    info!("data succesfully sent");
    Ok(())
}

#[cfg(test)]
mod test_kafka {
    use std::collections::HashMap;

    use rs_utils::config::Config;

    use super::*;
    use crate::config::OPAConfig;

    #[tokio::test]
    async fn test_send_to_kafka() {
        let map = HashMap::from([("examples".to_owned(), 42)]);
        let config = OPAConfig::new("CONFIG").await;
        assert!(send_to_kafka(&config.kafka, "examples", &map).await.is_ok());
    }
}
