use hyper::Body;
use serde::Deserialize;
use std::net::SocketAddr;
use std::task::{Context, Poll};
use std::time::Instant;
use tonic::{body::BoxBody, Request, Status};
use tower::{Layer, Service};
use uuid::Uuid;

// An interceptor  function.
pub(crate) fn intercept(req: Request<()>) -> Result<Request<()>, Status> {
    let addr = req.remote_addr();
    let mut req = req;

    // Set an extension that can be retrieved by `say_hello`
    req.extensions_mut().insert(MyExtension {
        correlation_id: Uuid::new_v4().to_string(),
        ip: addr,
        local_cache: Some(Instant::now()),
    });

    // corelation
    Ok(req)
}

#[derive(Default)]
struct MyExtension {
    correlation_id: String,
    ip: Option<SocketAddr>,
    local_cache: Option<Instant>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub(crate) struct MyMiddlewareLayer;

impl<S> Layer<S> for MyMiddlewareLayer {
    type Service = MyMiddleware<S>;

    fn layer(&self, service: S) -> Self::Service {
        MyMiddleware { inner: service }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct MyMiddleware<S> {
    inner: S,
}

///Struct containing data to be logged.
#[derive(Default)]
pub struct LogDatax<'a> {
    ip: String,
    correlation_id: String,
    duration: f32,
    method: String,
    uri: String,
    status: &'a str,
    referer: &'a str,
}
impl<S> Service<hyper::Request<Body>> for MyMiddleware<S>
where
    S: Service<hyper::Request<Body>, Response = hyper::Response<BoxBody>>
        + Clone
        + Send
        + 'static
        + std::fmt::Debug,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = futures::future::BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: hyper::Request<Body>) -> Self::Future {
        // This is necessary because tonic internally uses `tower::buffer::Buffer`.
        // See https://github.com/tower-rs/tower/issues/547#issuecomment-767629149
        // for details on why this is necessary
        let clone = self.inner.clone();
        let mut inner = std::mem::replace(&mut self.inner, clone);

        Box::pin(async move {
            /*
            let response= api_post_response(req).await;

            Ok(hyper::Response::builder()
                .status("200")
                .body(BoxBody::from(empty_body()))
                .unwrap())
            */

            let extension = match req.extensions_mut().remove() {
                Some(extension) => extension,
                None => MyExtension::default(),
            };
            let method = req.method().to_string();
            let uri = req.uri().to_string();

            // Do extra async work here...
            let response: hyper::Response<_> = inner.call(req).await?;

            let ip = match extension.ip {
                Some(ipx) => ipx.to_string(),
                None => String::new(),
            };

            let duration = match extension.local_cache {
                Some(duration) => duration.elapsed().as_secs_f32() * 1000.0,
                None => 0.0,
            };

            let mut data = LogDatax {
                ip,
                correlation_id: extension.correlation_id,
                duration,
                method,
                uri,
                status: "0",
                referer: "None",
            };

            /*let status_grpc = match response.headers().get("grpc-status") {
                Some(status) => status.to_str().unwrap(),
                None => "0",
            };*/

            if let Some(status) = response.headers().get("grpc-status") {
                data.status = status.to_str().unwrap();
            }

            info!("{}", log_formater(data));

            Ok(response)
        })
    }
}

///Format the log to a json format.
pub fn log_formater(data: LogDatax) -> String {
    format!(
        "{{\"correlation_id\":\"{}\",\"ipadress\":\"{}\",\"duration\":\"{} ms\",\"method\":\"{}\",\"uri\":\"{}\",\"status\":\"{}\",\"referer\":\"{}\"}}",
        data.correlation_id,
        data.ip,
        data.duration,
        data.method,
        data.uri,
        data.status,
        data.referer
    )
}
