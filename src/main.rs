use std::{sync::Arc, path::Path};

use anyhow::Result;
use log::warn;
use once_cell::sync::Lazy;
use rocket::{catchers, routes, Build, Rocket};
use tokio::sync::RwLock;

use rs_utils::config::{init_watcher, Config};

mod router;
use router::{handle_metrics, health_alive, health_ready, post};
mod types;

mod middleware;
use middleware::{logger, timer};
mod controller;

mod utils;
use utils::{error_catcher::default, logger::setup_logger};

mod config;
use config::OPAConfig;

/// This launch the rocket server.
fn setup_rocket(config: Arc<RwLock<OPAConfig>>) -> Rocket<Build> {
    rocket::build()
        .manage(config)
        .attach(timer::RequestTimer)
        .attach(logger::RequestLogger)
        .register("/", catchers![default])
        .mount(
            "/",
            routes![post, handle_metrics, health_alive, health_ready],
        )
}
/// ## OPA
/// OPA rust is an api that allows to manage authorizations. It is based on KRATOS Ory and integrates opa wasm.
///
/// To start the api follow the steps:
///
/// ## RUN KRATOS (Before units-test)
///
/// ```
/// ## For start kratos
/// #Step 1:
/// make
/// #Step 2:
/// make kratos
/// #Step 3:
/// make start
/// #Step 4 (grep the id of an identity for your test curl):
/// make fake
///
/// ## For stop kratos and clean repository
/// #Step 1:
/// make stop
/// #Step 2
/// make clean
/// ```
/// ## RUN OPA RUST AND CALL HIM
///
/// ```
/// ## For start opa rust api
/// #Step 1:
/// cargo build
/// #Step 2:
/// cargo run
///
/// ## For call to CURL
/// #Step 1:
/// Get a identity ID to KRATOS SERVICE
/// #Step 2
/// curl -X POST -L http://127.0.0.1:8000 -H "Content-Type: application/json" -d '{"kratos": "<Kratos Identity ID>", "eval": "private_projects","resource": 222,"role":"admin","method": "get", "uri": "api/projects" }'
///
///set var:
///CONFIG for config file path
///OPA_POLICY for opa path must be the same parent dir as CONFIG or the config 
///loader will not work correctly!!!!!!!
///OPA_QUERY query in rego
///
/// enjoy :)
#[rocket::main]
async fn main() -> Result<()> {
    let path = std::env::var("CONFIG_OPA").unwrap_or_else(|_| {
        warn!("config variable not found switching to fallback");
        "config/config.toml".to_owned()
    });
    let path = Path::new(&path);
    let path_dir = path.parent().unwrap().to_owned();
    setup_logger(std::io::stdout()).expect("failled to initialize the logger");
    let config = Arc::new(RwLock::new(OPAConfig::new(&path)));
    tokio::task::spawn(init_watcher(path_dir, config.clone(), None));
    Lazy::force(&utils::telemetry::METER);

    let rocket_handle = tokio::spawn(setup_rocket(config).launch());
    let _rocket_res = rocket_handle.await?;

    Ok(())
}
