use std::time::SystemTime;

use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Data, Request, Response,
};

/// Fairing for timing requests.
pub struct RequestTimer;

/// Value stored in request-local state.
#[derive(Copy, Clone)]
struct TimerStart(Option<SystemTime>);

#[rocket::async_trait]
impl Fairing for RequestTimer {
    fn info(&self) -> Info {
        Info {
            name: "Request Timer",
            kind: Kind::Request | Kind::Response,
        }
    }

    /// Stores the start time of the request in request-local state.
    async fn on_request(&self, request: &mut Request<'_>, _: &mut Data<'_>) {
        request.local_cache(|| TimerStart(Some(SystemTime::now())));
    }

    /// Adds a header to the response indicating how long the server took to
    /// process the request.
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let start_time = req.local_cache(|| TimerStart(None));
        if let Some(Ok(duration)) = start_time.0.map(|st| st.elapsed()) {
            let ms = duration.as_secs_f32() * 1000.0;
            res.set_header(Header::new("duration", format!("{} ms", ms)));
        }
    }
}

#[cfg(test)]
mod test_timer_fairing {
    use std::sync::Arc;

    use rocket::local::blocking::Client;
    use rs_utils::config::Config;
    use tokio::sync::RwLock;

    use crate::{config::OPAConfig, setup_rocket};

    #[test]
    fn test_timer_attachment() {
        let config = Arc::new(RwLock::new(OPAConfig::new("CONFIG")));
        let client = Client::tracked(setup_rocket(config)).unwrap();
        let response = client
            .post("/webhook?providerId=github&providerType=github")
            .dispatch();
        assert!(response.headers().contains("duration"));
    }
}
