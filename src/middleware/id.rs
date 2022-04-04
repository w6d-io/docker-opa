use rocket::{
    fairing::{Fairing, Info, Kind},
    http::Header,
    Data, Request,
};
use uuid::Uuid;

///Fairing for setting the correlation id.
pub struct CorrelationId;

fn attach_id(req: &mut Request) {
    let correlation_id = Uuid::new_v4();
    req.add_header(Header::new("correlation_id", correlation_id.to_string()));
}

///Set a correlation_id in the header.
#[rocket::async_trait]
impl Fairing for CorrelationId {
    fn info(&self) -> Info {
        Info {
            name: "set correlation id",
            kind: Kind::Request,
        }
    }

    async fn on_request(&self, req: &mut Request<'_>, _data: &mut Data<'_>) {
        attach_id(req)
    }
}

#[cfg(test)]
mod fairing_corelation_id_test {
    use rocket::local::blocking::Client;

    use super::*;
    #[test]
    fn test_id_attach() {
        let rocket = rocket::build().attach(CorrelationId);
        let client = Client::tracked(rocket).expect("valid rocket");
        let mut req = client.post("/webhook/github");
        attach_id(&mut req);
        assert!(req.headers().contains("correlation_id"))
    }

    #[test]
    fn test_id_fairing() {
        let rocket = rocket::build().attach(CorrelationId);
        let client = Client::tracked(rocket).expect("valid rocket");
        let _res = client.get("/webhook/github").dispatch();
    }
}
