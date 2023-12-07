use std::{sync::Arc, fmt::Debug};

use anyhow::{anyhow, Result};
use figment::value;
use log::{info, debug};
use tokio::sync::RwLock;
use regorus::Engine;
use thiserror::Error;

use rs_utils::kratos::Identity;

use crate::{config::OPAConfig, types::opa::Response};

#[derive(Error, Debug)]
pub enum Error{
    #[error("the rego evaluation encountered an error: {0}")]
    Rego(String),
    #[error("while deserializing.")]
    Serialization(#[from] serde_json::Error),
    #[error("an error occurred.")]
    Other(#[from] anyhow::Error),
}


/// validate the input data and identity angaint the wasm policy module
pub async fn evaluate(
    input: serde_json::Value,
    data: Identity,
    config: &Arc<RwLock<OPAConfig>>,
) -> Result<Response, Error> {
    // instance opa wasm
    let read_config = config.read().await;
    let policies = match read_config.opa_policy {
        Some(ref policy) => policy,
        None => Err(anyhow!("opa not initialized"))?,
    };
    info!("input: {input:#?}");
    info!("data: {data:#?}");

    info!("Creating Opa Interpreter!");
    let data = serde_json::to_string(&data)?;
    let data = regorus::Value::from_json_str(&data)?;
    let input = serde_json::to_string(&input)?;
    let input = regorus::Value::from_json_str(&input)?;
    let mut rego = Engine::new();
    rego.set_input(input);
    rego.add_data(data)?;
    rego.add_policy_from_file(&policies.module)?;
    let tracing = cfg!(debug_assertions);
    let opa_results = rego.eval_query(policies.query.clone(), tracing)?;
    println!("result: {:?}",opa_results.result);
    let opa_result = opa_results.result.get(0).ok_or_else(|| anyhow!("should never be empty."))?;
    let results = opa_result.expressions.get(0).ok_or_else(|| anyhow!("should never be empty."))?;
    let value = results.value.as_bool()?;
    // evaluate input and get boolean result
    let eval = Response { validate: value.to_owned() };
    Ok(eval)
}
