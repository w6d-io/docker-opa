use std::sync::Arc;

use anyhow::{anyhow, Result};
use regorus::Engine;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tracing::{info, debug};

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

    info!("Creating Opa Interpreter!");
    let data = regorus::Value::from_json_str(data.get())?;
    let input = regorus::Value::from_json_str(input.get())?;
    info!("input: {input:#?}");
    info!("data: {data:#?}");
    let mut rego = Engine::new();
    rego.set_enable_coverage(true);
    rego.add_policy_from_file(&policies.module)?;
    rego.add_data(data)?;
    rego.set_input(input);
    // let tracing = true;
    let opa_results = rego.eval_rule(policies.query.clone())?;
    println!("result: {:?}", opa_results);
    let report = rego.get_coverage_report()?.to_string_pretty()?;
    debug!("{report:?}");
    let opa_result = opa_results
        .as_bool()?;
        /* .ok_or_else(|| Router::EmptyResponse)?;
    let results = opa_result
        .expressions
        .first()
        .ok_or_else(|| Router::EmptyResponse)?; */
    //let value = results.value.as_bool()?;
    let json = serde_json::to_string(opa_result)?;
    Ok(json)
}
