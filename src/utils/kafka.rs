use std::collections::HashMap;

#[allow(unused_imports)]
use anyhow::{anyhow, bail, Result};
use rocket::serde::{json::serde_json, Serialize};

use crate::config::{OPAConfig, CONFIG};
use rslib::kafka::Producer;

///update the producer Producers if needed.

pub fn update_producer(mut config: OPAConfig) -> Result<OPAConfig> {
    let kafka = &config.kafka;
    let mut topics = match config.producers {
        Some(prod) => prod,
        None => HashMap::new(),
    };
    for topic in kafka.topics.iter() {
        topics.insert(topic.0.to_owned(), Producer::new(kafka, topic.0)?);
    }
    config.producers = Some(topics);
    Ok(config)
}

///Send data to kafka.
#[cfg(not(tarpaulin_include))]
pub async fn send_to_kafka<T: Serialize>(_topic: &str, data: T) -> Result<()> {
    let _message = serde_json::to_string(&data)?;
    let _conf = CONFIG.read().await;
    #[cfg(not(test))]
    match &_conf.producers {
        Some(prod) => prod
            .get(_topic)
            .ok_or_else(|| anyhow!("failed to get asked kafka topic!"))?
            .produce(&_message)?,
        None => bail!("topic not found"),
    }
    info!("data succesfully sent");
    Ok(())
}

#[cfg(test)]
mod test_kafka {

    use super::*;

    #[tokio::test]
    async fn test_send_to_kafka() {
        let map = HashMap::from([("examples".to_owned(), 42)]);
        assert!(send_to_kafka("examples", &map).await.is_ok());
    }
}
