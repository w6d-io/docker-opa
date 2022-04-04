use rocket::response::content;
use std::{fs, io::{stdout, Write}, env};
use std::borrow::BorrowMut;
use std::ops::Deref;
use clap::{App, Arg};
use opa_wasm::{Policy, Value};
use tracing::Level;
use tracing_subscriber::{fmt, EnvFilter};
use anyhow::Result;
use walkdir::WalkDir;
use curl::easy::Easy;
use rocket::form::validate::Contains;
use rocket::State;
use crate::types::{input, Input, Opadata};

extern crate curl;
extern crate serde;

use crate::config;
use crate::OPA2;

pub async fn post_eval(input: Input, data: Opadata) ->  Result<&'static str>  {
    // serde data and input struct to string
    let data_roles = serde_json::to_string(&data.traits.roles)?;
    let input_str = serde_json::to_string(&input)?;
    let data_str = "{\"roles\":".to_owned() + &data_roles + "}";
    println!("1");
    // serde data and input string to json
    let mut data = serde_json::from_str::<Value>(&data_str)?;
    let mut input = serde_json::from_str::<Value>(&input_str)?;
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
    let result = opa.evaluate(&input)?;
    println!("5");
    if result.to_string() == "{}".to_string() {
        Ok(r#"{"eval": false}"#)
    } else {
        Ok(r#"{"eval": true}"#)
    }
}