use std::{collections::HashMap, path::Path};

use anyhow::{bail, Result};
use figment::{
    providers::{Format, Yaml},
    Figment,
};
use once_cell::sync::Lazy;
use serde::Deserialize;
use tokio::sync::RwLock;

use rs_utils::config::Config;
use rslib::{config::Kafka, kafka::Producer};

use crate::utils::kafka::update_producer;

#[derive(Deserialize)]
pub struct OPAConfig {
    pub kafka: Kafka,
    pub salt: String,
    pub salt_length: usize,
    pub http: HashMap<String, String>,
    pub grpc: HashMap<String, String>,

    #[serde(skip)]
    pub producers: Option<HashMap<String, Producer>>,
}

///static containing the config data. It is ititialised on first read then
///updated each time the file is writen.
pub static CONFIG: Lazy<RwLock<OPAConfig>> = Lazy::new(|| RwLock::new(OPAConfig::new("CONFIG")));
impl Config for OPAConfig {
    ///initialise the config struct
    fn new(var: &str) -> Self {
        let path = match std::env::var(var) {
            Ok(path) => path,
            Err(e) => {
                warn!("error while reading environment variable: {e}, switching to fallback.");
                "configs/config.yaml".to_owned()
            }
        };
        match Self::update(&path) {
            Ok(conf) => conf,
            Err(e) => panic!("failed to update config {:?}: {:?}", path, e),
        }
    }

    ///update the config in the static variable
    fn update<P: AsRef<Path>>(path: P) -> Result<Self> {
        if !path.as_ref().exists() {
            bail!("config was not found");
        }
        let mut config: OPAConfig = Figment::new().merge(Yaml::file(path)).extract()?;
        config = update_producer(config)?;
        //info!("{config:?}");
        Ok(config)
    }
}

pub static POLICY: Lazy<String> = Lazy::new(|| read_policy("POLICY"));
///initialise the config struct
fn read_policy(var: &str) -> String {
    let path = match std::env::var(var) {
        Ok(path) => path,
        Err(e) => {
            warn!("error while reading policy environment variable: {e}, switching to fallback.");
            "configs/acl.rego".to_owned()
        }
    };
    match std::fs::read_to_string(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Error while reading Policy file: {e}");
        }
    }
}

pub static INPUT: Lazy<String> = Lazy::new(|| read_input("INPUT"));
///initialise the config struct
fn read_input(var: &str) -> String {
    let path = match std::env::var(var) {
        Ok(path) => path,
        Err(e) => {
            warn!("error while reading policy environment variable: {e}, switching to fallback.");
            "examples/input.json".to_owned()
        }
    };
    match std::fs::read_to_string(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Error while reading Policy file: {e}");
        }
    }
}

pub static DATA: Lazy<String> = Lazy::new(|| read_data("DATA"));
///initialise the config struct
fn read_data(var: &str) -> String {
    let path = match std::env::var(var) {
        Ok(path) => path,
        Err(e) => {
            warn!("error while reading policy environment variable: {e}, switching to fallback.");
            "examples/kratos_payload.json".to_owned()
        }
    };

    match std::fs::read_to_string(path) {
        Ok(file) => file,
        Err(e) => {
            panic!("Error while reading Policy file: {e}");
        }
    }
}
