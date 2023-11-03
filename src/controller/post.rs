use std::sync::Arc;

use anyhow::{bail, Result};
use log::{debug, info};
use opa_wasm::Runtime;
use tokio::sync::RwLock;
use wasmtime::Store;

use rs_utils::kratos::Identity;

use crate::{config::OPAConfig, types::opa::Response};

/// validate the input data and identity angaint the wasm policy module
pub async fn post_eval(
    input: serde_json::Value,
    data: Identity,
    config: &Arc<RwLock<OPAConfig>>,
) -> Result<Response> {
    // instance opa wasm
    let mut write_config = config.write().await;
    let opa = match write_config.opa_policy {
        Some(ref mut policy) => policy,
        None => bail!("opa not initialized"),
    };
    info!("Creating Opa runtime!");
    let mut store = Store::new(&opa.engine, ());
    let runtime = Runtime::new(&mut store, &opa.module.clone()).await?;
    // set data in opa wasm instance
    let policy = runtime.with_data(&mut store, &data).await?;
    let entry_list = policy.entrypoints();
    info!("entry_list: {:?}", entry_list);
    // evaluate input and get boolean result
    let opa_result: Vec<serde_json::Value> = policy.evaluate(&mut store, "eval", &input).await?;
    info!("{opa_result:?}");
    debug!("opa_result: {opa_result:?}");
    let mut eval = Response { validate: false };
    if !opa_result.is_empty() {
        eval.validate = true;
    }
    // Serialize it to a JSON string.
    Ok(eval)
}
