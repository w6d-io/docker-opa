use anyhow::Result;
use opa_wasm::Policy;
use prometheus::proto;
use rocket::response::status;
use rocket::response::status::Accepted;
use rocket::State;
use tonic::{Request, Response, Status};

use crate::{
    open_policy_agency::{
        InputRequest,
        ResultReply,
        opaproto_server::{OpaprotoServer, Opaproto}
    },
    controller::post::post_eval,
    middleware::{
        ping_guard::Ready, resthttp1_guard::from_data_grpc, resthttp1_guard::PayloadGuard,
    },
    types::Result as BooleanResult,
    utils::{error::send_error, rocket_anyhow, telemetry::gather_telemetry},
};

//##
//##
//## GRPC (HTTP 1) Route
//##
//##
#[derive(Default)]
pub struct OpaRouter {}

#[tonic::async_trait]
impl Opaproto for OpaRouter {
    async fn get_decision(
        &self,
        request: Request<InputRequest>,
    ) -> Result<Response<ResultReply>, Status> {
        let data = from_data_grpc(request).await?;
        let resp = post_eval(data.input, data.data).await;
        let eval: BooleanResult = resp.unwrap();
        let reply = ResultReply {
            result: eval.result.to_string(),
        };

        Ok(Response::new(reply))
    }
}

//##
//##
//## HTTP 2 Route
//##
//##
///route to post
#[post("/", data = "<data>")]
pub async fn post(data: PayloadGuard) -> rocket_anyhow::Result<String> {
    // logger::log::router_error("wrong eval type")
    let eval = post_eval(data.input, data.data).await?;
    let resp = serde_json::to_string(&eval)?;
    Ok(resp)
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
