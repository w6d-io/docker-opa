#[allow(unused_imports)]

#[macro_use]
extern crate rocket;

use anyhow::Result;
use log::warn;
use once_cell::sync::Lazy;
use rocket::{Build, Rocket};

mod config;
mod controller;
mod middleware;
use middleware::{id, logger, timer};
mod router;
use router::{handle_metrics, health_alive, health_ready, post};
mod utils;
use opa_wasm::{Policy, Value};
use tokio::sync::Mutex;
use utils::{error_catcher::default, logger::setup_logger};
use rs_utils::config::init_watcher;
use crate::config::CONFIG;

mod types;

/// This launch the rocket server.
fn setup_rocket() -> Rocket<Build> {
    rocket::build()
        .attach(id::CorrelationId)
        .attach(timer::RequestTimer)
        .attach(logger::RequestLogger)
        .register("/", catchers![default])
        .mount(
            "/",
            routes![post, handle_metrics, health_alive, health_ready],
        )
}

///this lauch ou grpc and rest server
#[rocket::main]
async fn main() -> Result<()> {
    let config = std::env::var("CONFIG").unwrap_or_else(|_| {
        warn!("config variable not found switching to fallback");
        "config.yaml".to_owned()
    });
    setup_logger(std::io::stdout()).expect("failled to initialize the logger");
    tokio::task::spawn(init_watcher(config, &CONFIG, None));
    //initialise config POLICY
    Lazy::force(&config::POLICY);
    //initialise config CONFIG
    Lazy::force(&config::CONFIG);
    //initialise config INPUT
    Lazy::force(&config::INPUT);
    //initialise config DATA
    Lazy::force(&config::DATA);
    //initialise the lazy static for open telemetry
    Lazy::force(&utils::telemetry::METER);
    //initialise the lazy static for open telemetry

    let rocket_handle = tokio::spawn(setup_rocket().launch());
    let (rocket_res,) = tokio::try_join!(rocket_handle)?;
    rocket_res?;
    Ok(())
}

pub static OPA2: Lazy<Mutex<Policy>> = Lazy::new(|| Mutex::new(init_opa()));

///initialise the config struct
fn init_opa() -> Policy {
    // compilation wasm
    let policy_path = "configs/acl.rego";
    let query = "data.app.rbac.main";
    let module = match opa_go::wasm::compile(query, &policy_path){
        Ok(module) => module,
        Err(err) => panic!("{}", err)
    };
    match Policy::from_wasm(&module) {
        Ok(opa) => opa,
        Err(err) => panic!("{:?}", err)
    }
}