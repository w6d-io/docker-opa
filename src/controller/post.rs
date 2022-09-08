use crate::types::{input, Input, Opadata, Result as OpaResult};
use anyhow::Result;
use clap::{App, Arg};
use curl::easy::Easy;
use opa_wasm::{Policy, Value};
use rocket::form::validate::Contains;
use rocket::response::content;
use rocket::State;
use std::borrow::BorrowMut;
use std::ops::Deref;
use std::{
    env, fs,
    io::{stdout, Write},
};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};
use walkdir::WalkDir;

extern crate curl;
extern crate serde;

use crate::config;
use crate::OPA2;

pub async fn post_eval(input: Input, data: Opadata) -> Result<OpaResult> {
    // serde data and input struct to string
    let data_roles = serde_json::to_string(&data.traits.roles)?;
    let input_str = serde_json::to_string(&input)?;
    let data_str = "{\"roles\":".to_owned() + &data_roles + "}";
    println!("1");
    // serde data and input string to json
    let data = serde_json::from_str::<Value>(&data_str)?;
    let input = serde_json::from_str::<Value>(&input_str)?;
    println!("2");
    println!("{input:#?}");
    println!("2.1");
    // instance opa wasm
    let mut opa = OPA2.lock().await;
    println!("3");
    // set data in opa wasm instance
    opa.set_data(&data)?;
    println!("4");

    println!("{opa:#?}");
    println!("4.1");
    // evaluate input and get boolean result
    let opa_result = opa.evaluate(&input)?;
    println!("5");

    let mut eval = OpaResult { result: false };

    if opa_result.to_string() != "{}".to_string() {
        eval.result = true;
    }

    // Serialize it to a JSON string.
    Ok(eval)
}
