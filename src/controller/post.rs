use std::sync::Arc;

use anyhow::{bail, Result};
use opa_wasm::Value;
use tokio::sync::RwLock;

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
    let data = serde_json::from_str::<Value>(&data_str)?;
    let input = serde_json::from_str::<Value>(&input_str)?;

    // instance opa wasm
    let mut write_config = config.write().await;
    let opa = match write_config.opa_policy {
        Some(ref mut policy) => policy,
        None => bail!("opa not initialized"),
    };

    // set data in opa wasm instance
    opa.set_data(&data)?;

    // evaluate input and get boolean result
    let opa_result = opa.evaluate(&input)?;

    let mut eval = Response { validate: false };

    if opa_result.to_string() != "{}" {
        eval.validate = true;
    }

    // Serialize it to a JSON string.
    Ok(eval)
}
