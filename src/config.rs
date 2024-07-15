use std::{
    collections::HashMap,
    env::var,
    fmt,
    path::{Path, PathBuf},
    sync::Arc,
};

use anyhow::{bail, Result};
use axum::async_trait;
use figment::{
    providers::{Format, Toml},
    Figment,
};
use kafka::{
    producer::{default_config, BaseProducer},
    KafkaProducer,
};
use serde::Deserialize;
use tracing::info;

use rs_utils::config::Config;

pub const CONFIG_FALLBACK: &str = "tests/config.toml";

///Represntation of a kafka config it only contain producer
#[derive(Deserialize, Default, Clone)]
pub struct Kafka {
    pub service: String,
    pub topics: HashMap<String, String>,
    #[serde(skip)]
    pub producers: Option<HashMap<String, Arc<KafkaProducer<BaseProducer>>>>,
}

impl fmt::Debug for Kafka {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Kafka")
            .field("service", &self.service)
            .field("topics", &self.topics)
            .finish_non_exhaustive()
    }
}

impl Kafka {
    ///Update the producer if needed.
    pub fn update_producer(&mut self) -> Result<()> {
        let mut new_producer = HashMap::new();
        let producer = match self.producers {
            Some(ref mut prod) => prod,
            None => &mut new_producer,
        };
        for topic in &self.topics {
            producer.insert(
                topic.0.to_owned(),
                Arc::new(KafkaProducer::new(&default_config(&self.service), topic.0)?),
            );
        }
        Ok(())
    }
}

/// Represntation of the policy config,
/// this contain the data for rego.
#[derive(Deserialize, Clone, Default, Debug)]
pub struct Policy {
    pub query: String,
    pub module: PathBuf,
}

/// Represntation of the ports utilized by the service
#[derive(Deserialize, Clone, Default, Debug)]
pub struct Ports {
    pub main: String,
    pub health: String,
}

/// Represntation of the service config
#[derive(Deserialize, Clone, Default, Debug)]
pub struct Service {
    pub addr: String,
    pub ports: Ports,
}

/// Represntation of the app config
#[derive(Deserialize, Default, Clone, Debug)]
pub struct Opa {
    pub kafka: Kafka,
    pub service: Service,
    // pub grpc: HashMap<String, String>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub policy: Option<Policy>,
}

///static containing the config data. It is ititialised on first read then
///updated each time the file is writen.

#[async_trait]
impl Config for Opa {
    fn set_path<T: AsRef<Path>>(&mut self, path: T) -> &mut Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    ///Update the config in the static variable
    async fn update(&mut self) -> Result<()> {
        let path = match self.path {
            Some(ref path) => path as &Path,
            None => bail!("config file path not set"),
        };
        match path.try_exists() {
            Ok(exists) if !exists => bail!("config was not found"),
            Err(e) => bail!(e),
            _ => (),
        }
        let mut config: Opa = Figment::new().merge(Toml::file(path)).extract()?;
        config.path = Some(path.to_owned());
        config.kafka.update_producer()?;
        config.policy = Some(init_opa()?);
        *self = config;
        Ok(())
    }
}

///Initialize opa
fn init_opa() -> Result<Policy> {
    let module_path = var("OPA_POLICY").unwrap_or_else(|_| "configs/acl.rego".to_owned());
    info!("Using policy from: {}!", module_path);
    let module = PathBuf::from(module_path);
    if !module.try_exists()? {
        bail!("The rego file do not exist!")
    }
    let query = var("OPA_QUERY").unwrap_or_else(|_| "data.app.rbac.main".to_owned());
    Ok(Policy { query, module })
}
