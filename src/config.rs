use std::{
    collections::HashMap,
    env::var,
    path::{Path, PathBuf},
};

use anyhow::{bail, Result};
use figment::{
    providers::{Format, Toml},
    Figment,
};
use kafka::{
    producer::{default_config, BaseProducer},
    KafkaProducer,
};
use opa_wasm::Policy;
use serde::Deserialize;

use rs_utils::config::Config;

#[derive(Deserialize, Default)]
pub struct Kafka {
    pub service: String,
    pub topics: HashMap<String, String>,
    #[serde(skip)]
    pub producers: Option<HashMap<String, KafkaProducer<BaseProducer>>>,
}

impl Kafka {
    ///update the producer Producers if needed.
    pub fn update_producer(&mut self) -> Result<()> {
        let mut new_producer = HashMap::new();
        let producer = match self.producers {
            Some(ref mut prod) => prod,
            None => &mut new_producer,
        };
        for topic in self.topics.iter() {
            producer.insert(
                topic.0.to_owned(),
                KafkaProducer::new(&default_config(&self.service), topic.0)?,
            );
        }
        Ok(())
    }
}

#[derive(Deserialize, Default)]
pub struct OPAConfig {
    pub kafka: Kafka,
    pub salt: String,
    pub salt_length: usize,
    pub http: HashMap<String, String>,
    // pub grpc: HashMap<String, String>,
    #[serde(skip)]
    pub path: Option<PathBuf>,
    #[serde(skip)]
    pub opa_policy: Option<Policy>,
}

///static containing the config data. It is ititialised on first read then
///updated each time the file is writen.

impl Config for OPAConfig {
    fn set_path<T: AsRef<Path>>(&mut self, path: T) -> &mut Self {
        self.path = Some(path.as_ref().to_path_buf());
        self
    }

    ///update the config in the static variable
    fn update(&mut self) -> Result<()> {
        let path = match self.path {
            Some(ref path) => path as &Path,
            None => bail!("config file path not set"),
        };
        match path.try_exists() {
            Ok(exists) if !exists => bail!("config was not found"),
            Err(e) => bail!(e),
            _ => (),
        }
        let mut config: OPAConfig = Figment::new().merge(Toml::file(path)).extract()?;
        config.kafka.update_producer()?;
        config.opa_policy = Some(init_opa()?);
        *self = config;
        Ok(())
    }
}

///initialise opa and compile policy to wasm
fn init_opa() -> Result<Policy> {
    // compilation wasm
    let policy_path = var("OPA_POLICY").unwrap_or_else(|_| "configs/acl.rego".to_owned());
    let query = var("OPA_QUERY").unwrap_or_else(|_| "data.app.rbac.main".to_owned());
    let module = opa_go::wasm::compile(&query, &policy_path)?;
    let policy = Policy::from_wasm(&module)?;
    Ok(policy)
}
