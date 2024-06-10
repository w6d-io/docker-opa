use std::sync::Arc;

use anyhow::{anyhow, Result};
use regorus::Engine;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tracing::info;

use crate::{config::Opa, error::Router};

/// validate the input data and identity angaint the wasm policy module
pub async fn evaluate(
    input: Box<RawValue>,
    data: Box<RawValue>,
    config: Arc<RwLock<Opa>>,
) -> Result<String, Router> {
    // instance opa wasm
    let read_config = config.read().await;
    let policies = match read_config.policy {
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
    let tracing = true;
    let opa_results = rego.eval_query(policies.query.clone(), tracing)?;
    println!("result: {:?}", opa_results.result);
    let opa_result = opa_results
        .result
        .first()
        .ok_or_else(|| Router::EmptyResponse)?;
    let results = opa_result
        .expressions
        .first()
        .ok_or_else(|| Router::EmptyResponse)?;
    let value = results.value.as_bool()?;
    let json = serde_json::to_string(value)?;
    Ok(json)
}
