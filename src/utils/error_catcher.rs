use opentelemetry::global;
use rocket::{catch, http::Status, Request};

///Default error handler it is called if the error status code
///is no treated in a specific handler.
#[catch(default)]
pub fn default(status: Status, req: &Request) -> String {
    let meter = global::meter("webhook");
    let counter = meter
        .u64_counter("bad request")
        .with_description("number of failed request")
        .init();
    counter.add(1, &[]);
    format!(
        "{{\"status\":\"{}\",\"resp\":\"{} KO\"}}",
        status.code,
        req.method().to_string().to_lowercase()
    )
}
