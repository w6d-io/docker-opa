use std::sync::Arc;

use anyhow::{anyhow, Result};
use regorus::Engine;
use serde_json::value::RawValue;
use tokio::sync::RwLock;
use tracing::{debug, info};

use crate::{config::Opa, error::Router};

///Validate the input data and identity against the OPA policy module.
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
    debug!("input: {input:#?}");
    debug!("data: {data:#?}");
    let mut rego = Engine::new();
    rego.set_enable_coverage(true);
    rego.add_policy_from_file(&policies.module)?;
    rego.add_data(data)?;
    rego.set_input(input);
    info!("evaluating rule: {}", policies.query);
    let opa_results = rego.eval_rule(policies.query.clone())?;
    println!("result: {:?}", opa_results);
    let report = rego.get_coverage_report()?.to_string_pretty()?;
    debug!("{report}");
    let opa_result = opa_results.as_bool()?;
    let json = serde_json::to_string(opa_result)?;
    Ok(json)
}
