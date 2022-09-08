#[allow(unused_imports)]
#[macro_use]
extern crate rocket;
extern crate core;
use anyhow::Result;
use log::warn;
use once_cell::sync::Lazy;
use rocket::{Build, Rocket};

use tonic::transport::Server;

use middleware::{id, logger, timer};
use opa_wasm::{Policy, Value};

use rs_utils::config::init_watcher;

use std::time::Duration;

use tokio::sync::Mutex;
use utils::{error_catcher::default, logger::setup_logger};

pub mod open_policy_agency {
    tonic::include_proto!("openpolicyagency");
}
use open_policy_agency::opaproto_server::OpaprotoServer;

mod router;
use router::{handle_metrics, health_alive, health_ready, post, OpaRouter};
mod types;

mod middleware;
use middleware::{grpchttp2_guard::intercept, grpchttp2_guard::MyMiddlewareLayer};
mod controller;

mod utils;

mod config;
use config::CONFIG;

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

/// This launch the tonic server.
async fn setup_tonic() -> Result<()> {
    println!("setup_tonic 1");
    let (shutdown_trigger, shutdown_signal1) = triggered::trigger();

    info!("debug error");
    println!("setup_tonic 2");
    let addr = "127.0.0.1:3000".parse().unwrap();
    println!("setup_tonic 3");

    let opa = OpaRouter::default();
    println!("setup_tonic 4");

    let svc = OpaprotoServer::new(opa);
    // The stack of middleware that our service will be wrapped in
    let layer = tower::ServiceBuilder::new()
        // Interceptors can be also be applied as middleware
        .layer(tonic::service::interceptor(intercept))
        // Apply middleware from tower
        .timeout(Duration::from_secs(10))
        // Apply our own middleware
        .layer(MyMiddlewareLayer::default())
        .into_inner();

    println!("setup_tonic 5");

    ctrlc::set_handler(move || {
        shutdown_trigger.trigger();
    })
    .expect("Error setting Ctrl-C handler");

    println!("GreeterServer listening on {}", addr);

    Server::builder()
        // Wrap all services in the middleware stack
        .layer(layer)
        .add_service(svc)
        .serve_with_shutdown(addr, shutdown_signal1)
        .await?;

    println!("setup_tonic 6");

    Ok(())
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
    println!("main 1");
    let tonic_handle = tokio::spawn(setup_tonic());
    let (_rocket_res, _tonic_res) = tokio::try_join!(rocket_handle, tonic_handle)?;

    Ok(())
}

pub static OPA2: Lazy<Mutex<Policy>> = Lazy::new(|| Mutex::new(init_opa()));

///initialise the config struct
fn init_opa() -> Policy {
    // compilation wasm
    let policy_path = "configs/acl.rego";
    let query = "data.app.rbac.main";
    let module = match opa_go::wasm::compile(query, &policy_path) {
        Ok(module) => module,
        Err(err) => panic!("{}", err),
    };
    match Policy::from_wasm(&module) {
        Ok(opa) => opa,
        Err(err) => panic!("{:?}", err),
    }
}
