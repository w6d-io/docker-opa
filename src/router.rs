use rocket::response::status;
use rocket::response::status::Accepted;
use rocket::State;
use opa_wasm::{Policy};

use crate::{
    middleware::{payload_guard::PayloadGuard, ping_guard::Ready},
    utils::{error::send_error, rocket_anyhow, telemetry::gather_telemetry},
    controller::post::post_eval,
};

///route to post
#[post("/eval", data = "<data>")]
pub async fn post(data:PayloadGuard) -> rocket_anyhow::Result<&'static str> {
    // logger::log::router_error("wrong eval type")
    Ok(post_eval(data.input, data.data).await?)
}

///route for prometheus telemetry
#[get("/metrics")]
pub async fn handle_metrics() -> String {
    gather_telemetry().await
}

///route for prometheus telemetry
#[get("/health/alive")]
pub async fn health_alive() {}

///route for prometheus telemetry
#[get("/health/ready")]
pub async fn health_ready(_ready: Ready) {}

#[cfg(test)]
mod test_router {
    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;

    use crate::{setup_rocket, utils::secret::secret_test_utils::generate_hash};
}
