use std::sync::Arc;

use anyhow::{bail, Result, anyhow};
use tokio::sync::RwLock;
use opa_wasm::Runtime;
use log::{info, debug};

use crate::{
    config::OPAConfig,
    types::{
        kratos::Opadata,
        opa::{Input, Response},
    },
};

pub async fn post_eval(
    input: Input,
    data: Opadata,
    config: &Arc<RwLock<OPAConfig>>,
) -> Result<Response> {
    // serde data and input struct to string
    let data_roles = serde_json::to_string(&data.traits.roles)?;
    let input_str = serde_json::to_string(&input)?;
    let data_str = "{\"roles\":".to_owned() + &data_roles as &str + "}";

    // serde data and input string to json
    let data = serde_json::from_str::<serde_json::Value>(&data_str)?;
    let input = serde_json::from_str::<serde_json::Value>(&input_str)?;

    // instance opa wasm
    let mut write_config = config.write().await;
    let opa = match write_config.opa_policy {
        Some(ref mut policy) => policy,
        None => bail!("opa not initialized"),
    };
    info!("Creating Opa runtime!");
    let runtime = Runtime::new(&mut opa.store, &opa.module).await?;
    // set data in opa wasm instance
    let policy = runtime.with_data(&mut opa.store, &data).await?;
    let entry = policy.default_entrypoint().ok_or_else(||anyhow!("no entry point found!"))?;
    let entry_list = policy.entrypoints();
    debug!("entry_list: {:?}", entry_list);
    // evaluate input and get boolean result
    let opa_result: Vec<serde_json::Value> = policy.evaluate(&mut opa.store, entry , &input).await?;
    debug!("opa_result: {opa_result:?}");
    let mut eval = Response { validate: false };
    if !opa_result.is_empty() {
        eval.validate = true;
    }

    // Serialize it to a JSON string.
    Ok(eval)
}
