use std::sync::Arc;

use log::info;
use rocket::{get, post, State};
use rs_utils::anyhow_rocket::Result;
use tokio::sync::RwLock;

use crate::{
    config::OPAConfig, controller::post::post_eval, middleware::resthttp1_guard::PayloadGuard,
    utils::telemetry::gather_telemetry,
};

/* //##
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
        let data = from_data_grpc(request, config).await?;
        let resp = post_eval(data.input, data.data).await;
        let eval: BooleanResult = resp.unwrap();
        let reply = ResultReply {
            result: eval.result.to_string(),
        };

        Ok(Response::new(reply))
    }
} */

//##
//##
//## HTTP 2 Route
//##
//##
///route to post
#[post("/", data = "<data>")]
pub async fn post(data: PayloadGuard, config: &State<Arc<RwLock<OPAConfig>>>) -> Result<String> {
    // logger::log::router_error("wrong eval type")
    let eval = post_eval(data.input, data.data, config).await?;
    let resp = serde_json::to_string(&eval.validate)?;
    Ok(resp)
}

///route for prometheus telemetry
#[get("/metrics")]
pub async fn handle_metrics() -> String {
    gather_telemetry().await
}

///route for prometheus telemetry
#[get("/health/alive")]
pub async fn health_alive() -> Result<()> {
    Ok(())
}

///route for prometheus telemetry
#[get("/health/ready")]
pub async fn health_ready() -> Result<()> {
    Ok(())
}
