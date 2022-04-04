use log::info;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Method,
    Request, Response,
};

///Log the data about a a request/response.
pub struct RequestLogger;

///Struct containing data to be logged.
struct LogData<'a> {
    correlation_id: &'a str,
    ip: String,
    duration: &'a str,
    method: Method,
    uri: String,
    status: u16,
    referer: &'a str,
}

///Format the log to a json format.
fn log_formater(data: LogData) -> String {
    format!(
        "{{\"correlation_id\":\"{}\",\"ipadress\":\"{}\",\"duration\":\"{}\",\"method\":\"{}\",\"uri\":\"{}\",\"status\":\"{}\",\"referer\":\"{}\"}}",
        data.correlation_id,
        data.ip,
        data.duration,
        data.method,
        data.uri,
        data.status,
        data.referer
    )
}

#[rocket::async_trait]
impl Fairing for RequestLogger {
    fn info(&self) -> Info {
        Info {
            name: "Request Logger",
            kind: Kind::Response,
        }
    }
    async fn on_response<'r>(&self, req: &'r Request<'_>, res: &mut Response<'r>) {
        let data = LogData {
            ip: match req.client_ip() {
                Some(ip) => ip.to_string(),
                None => String::new(),
            },
            correlation_id: match req.headers().get_one("correlation_id") {
                Some(id) => id,
                None => "",
            },
            referer: match req.headers().get_one("Referer") {
                Some(referer) => referer,
                None => "",
            },
            duration: match res.headers().get_one("duration") {
                Some(duration) => duration,
                None => "",
            },
            method: req.method(),
            uri: req.uri().to_string(),
            status: res.status().code,
        };
        info!("{}", log_formater(data));
    }
}

#[cfg(test)]
mod fairing_logger_test {
    use super::*;

    #[test]
    fn test_logger_fainring_info() {
        let req_log = RequestLogger;
        let info = req_log.info();
        let expected = Info {
            name: "Request Logger",
            kind: Kind::Response,
        };
        assert_eq!(info.kind.to_string(), expected.kind.to_string());
        assert_eq!(info.name, expected.name);
    }

    #[test]
    fn test_formater() {
        let data = LogData {
            correlation_id: "examples",
            ip: "127.0.0.1".to_owned(),
            duration: "0.6000 ms",
            method: Method::Get,
            uri: "/".to_owned(),
            status: 404,
            referer: "",
        };
        let formated = log_formater(data);
        let expected = "{\"correlation_id\":\"examples\",\"ipadress\":\"127.0.0.1\",\"duration\":\"0.6000 ms\",\"method\":\"GET\",\"uri\":\"/\",\"status\":\"404\",\"referer\":\"\"}";
        assert_eq!(formated, expected)
    }
}
