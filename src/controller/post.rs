use crate::types::{Input, Opadata, Result as OpaResult};
use anyhow::Result;

use opa_wasm::Value;

extern crate curl;
extern crate serde;

use crate::OPA2;

pub async fn post_eval(input: Input, data: Opadata) -> Result<OpaResult> {
    // serde data and input struct to string
    let data_roles = serde_json::to_string(&data.traits.roles)?;
    let input_str = serde_json::to_string(&input)?;
    let data_str = "{\"roles\":".to_owned() + &data_roles + "}";

    // serde data and input string to json
    let data = serde_json::from_str::<Value>(&data_str)?;
    let input = serde_json::from_str::<Value>(&input_str)?;

    // instance opa wasm
    let mut opa = OPA2.lock().await;

    // set data in opa wasm instance
    opa.set_data(&data)?;

    // evaluate input and get boolean result
    let opa_result = opa.evaluate(&input)?;

    let mut eval = OpaResult { result: false };

    if opa_result.to_string() != "{}".to_string() {
        eval.result = true;
    }

    // Serialize it to a JSON string.
    Ok(eval)
}
