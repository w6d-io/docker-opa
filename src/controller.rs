use std::sync::Arc;

use anyhow::{anyhow, Result};
use regorus::Engine;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tracing::info;

use crate::{config::OPAConfig, error::RouterError};

/// validate the input data and identity angaint the wasm policy module
pub async fn evaluate(
    input: Box<RawValue>,
    data: Box<RawValue>,
    config: Arc<RwLock<OPAConfig>>,
) -> Result<String, RouterError> {
    // instance opa wasm
    let read_config = config.read().await;
    let policies = match read_config.opa_policy {
        Some(ref policy) => policy,
        None => Err(anyhow!("opa not initialized"))?,
    };
    info!("input: {input:#?}");
    info!("data: {data:#?}");

    info!("Creating Opa Interpreter!");
    let data = regorus::Value::from_json_str(data.get())?;
    let input = regorus::Value::from_json_str(input.get())?;
    let mut rego = Engine::new();
    rego.set_input(input);
    rego.add_data(data)?;
    rego.add_policy_from_file(&policies.module)?;
    let tracing = cfg!(debug_assertions);
    let opa_results = rego.eval_query(policies.query.clone(), tracing)?;
    println!("result: {:?}", opa_results.result);
    let opa_result = opa_results
        .result
        .get(0)
        .ok_or_else(|| RouterError::EmptyResponse)?;
    let results = opa_result
        .expressions
        .get(0)
        .ok_or_else(|| RouterError::EmptyResponse)?;
    let value = results.value.as_bool()?;
    let json = serde_json::to_string(value)?;
    Ok(json)
}
