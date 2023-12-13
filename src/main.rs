use std::{future::Future, sync::Arc};

use anyhow::Result;
use axum::{routing::{post, get}, Router, Server};

use stream_cancel::Tripwire;
use tokio::{sync::RwLock, task::JoinHandle};
use tower_http::request_id::{MakeRequestUuid, SetRequestIdLayer};
use tracing::{debug, info, warn};
use tracing_subscriber::{fmt, EnvFilter};

use rs_utils::config::{init_watcher, Config};

mod handler;
use handler::{fallback, shutdown_signal, shutdown_signal_trigger};
mod router;
use router::{alive, ready, eval_rego};
mod controller;
mod config;
use config::{OPAConfig, CONFIG_FALLBACK};
mod error;

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
type ConfigState = Arc<RwLock<OPAConfig>>;

///main router config
pub fn app(shared_state: ConfigState) -> Router {
    info!("configuring main router");

    Router::new()
        .route("/", post(eval_rego))
        .with_state(shared_state)
        .fallback(fallback)
        .layer(SetRequestIdLayer::x_request_id(MakeRequestUuid))
}

///heatlh router config
pub fn health(shared_state: ConfigState) -> Router {
    info!("configuring health router");
    let route = Router::new()
        .route("/alive", get(alive))
        .route("/ready", get(ready));
    Router::new()
        .nest("/health", route)
        .fallback(fallback)
        .with_state(shared_state)
}

///launch http router
fn make_http<T>(
    shared_state: ConfigState,
    f: fn(ConfigState) -> Router,
    addr: String,
    signal: T,
) -> JoinHandle<Result<(), hyper::Error>>
where
    T: Future<Output = ()> + std::marker::Send + 'static,
{
    info!("listening on {}", addr);
    //todo: add path for tlscertificate
    let handle = tokio::spawn(
        Server::bind(&addr.parse().unwrap())
            .serve(f(shared_state).into_make_service())
            .with_graceful_shutdown(signal),
    );
    info!("lauching http server on: {addr}");
    handle
}

#[cfg(not(tarpaulin_include))]
#[tokio::main]
async fn main() -> Result<()> {
    let logger = fmt()
        .with_target(false)
        .with_level(true)
        .with_env_filter(EnvFilter::from_default_env());
    match std::env::var("RUST_LOG") {
        Ok(env) if env == "debug" => logger.pretty().init(),
        _ => logger.init(),
    }

    let config_path = std::env::var("CONFIG").unwrap_or_else(|_| {
        warn!("Config variable not found switching to fallback");
        CONFIG_FALLBACK.to_owned()
    });
    debug!("launching from {:?}", std::env::current_exe());
    let config = OPAConfig::new(&config_path).await;
    let service = config.service.clone();
    let shared_state = Arc::new(RwLock::new(config));
    tokio::spawn(init_watcher(config_path, shared_state.clone(), None));
    let (trigger, shutdown) = Tripwire::new();
    let signal = shutdown_signal_trigger(trigger);
    info!("statrting http router");
    let http_addr = service.addr.clone() + ":" + &service.ports.main as &str;
    let http = make_http(shared_state.clone(), app, http_addr, signal);
    let signal = shutdown_signal(shutdown);
    let health_addr = service.addr.clone() + ":" + &service.ports.health as &str;
    let health = make_http(shared_state.clone(), health, health_addr, signal);
    let (http_critical, health_critical) = tokio::try_join!(http, health)?;
    http_critical?;
    health_critical?;
    Ok(())
}
