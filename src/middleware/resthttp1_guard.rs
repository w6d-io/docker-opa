use std::{result, sync::Arc};

use anyhow::anyhow;
use base64::decode;
use log::{error, info, warn};
use serde::Deserialize;
use rocket::{
    data::{self, Data, FromData, ToByteUnit},
    http::Status,
    outcome::Outcome,
    request::Request,
    serde::json::serde_json,
    State,
};
use thiserror::Error;
use tokio::sync::RwLock;

use rs_utils::kratos::Identity;

use crate::{
    config::OPAConfig,
    utils::error::send_error,
};

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
///enum containing all error type for the payload validation
pub enum PayloadValidationError {
    #[error("the body is missing")]
    ErrorMissingBody,
    #[error("failed to deserialize: {0}")]
    ErrorFailedToDeserialize(#[from] serde_json::Error),
    #[error("payload as broken utf8 encoding: {0}")]
    ErrorMalformedPayload(#[from] std::io::Error),
    #[error("the body is to big")]
    ErrorBodyToBig,
    #[error("wrong field type in the input payload")]
    ErrorWrongEvent,
    #[error("bad request")]
    ErrorBadRequest(#[from] reqwest::Error),
    #[error("other: {0}")]
    Other(#[from] anyhow::Error),
}

type Result<T, E = PayloadValidationError> = result::Result<T, E>;


// the response struct
#[derive(Debug, Deserialize)]
pub struct PayloadGuard {
    pub(crate) input: serde_json::Value,
    pub(crate) data: Identity,
}

///Deserialize a push event.
pub async fn payload_input_deserialize<'r>(
    data: String,
    config: &Arc<RwLock<OPAConfig>>,
) -> data::Outcome<'r, PayloadGuard> {
    let config_read = config.read().await;
    // Deserializing payload input
    let payload = match serde_json::from_str::<PayloadGuard>(&data) {
        Ok(input) => input,
        Err(e) => {
            error!("error while deserializing the request input:{e}");
            if let Err(_e) = send_error(&config_read.kafka, "error", &e).await {
                warn!("Failed to send error to kafka");
            };
            return Outcome::Error((
                Status::BadRequest,
                PayloadValidationError::ErrorFailedToDeserialize(e),
            ));
        }
    };
    Outcome::Success(payload)
}

///Get body from the request.
///will fail if:
/// - the body is not a valid utf8 string.
/// - the body is empty.
/// - the body is to big.
async fn get_body(data: Data<'_>) -> Result<String, PayloadValidationError> {
    let body = match data.open(25.megabytes()).into_string().await {
        Ok(body) => body,
        Err(e) => {
            error!("error while opening the request body: {e}");
            return Err(PayloadValidationError::ErrorMalformedPayload(e));
        }
    };
    if body.is_empty() {
        error!("the request body is empty");
        return Err(PayloadValidationError::ErrorMissingBody);
    }
    if !body.is_complete() {
        error!("the request body is to big");
        return Err(PayloadValidationError::ErrorBodyToBig);
    }
    Ok(body.into_inner())
}

#[rocket::async_trait]
///this guard validatde the pay load with a its hash then deserialize it
impl<'r> FromData<'r> for PayloadGuard {
    type Error = PayloadValidationError;

    async fn from_data(req: &'r Request<'_>, data: Data<'r>) -> data::Outcome<'r, Self> {
        let config = match req.guard::<&State<Arc<RwLock<OPAConfig>>>>().await {
            Outcome::Success(state) => state,
            Outcome::Error(e) => {
                let error = anyhow!(e.0.reason().unwrap());
                return Outcome::Error((e.0, PayloadValidationError::from(error)));
            }
            Outcome::Forward(_) => return Outcome::Forward((data, Status::ImATeapot)),
        };
        // get info request from header
        match req.headers().get_one("input"){
            None => {
                info!("Header is empty")
            }
            Some(header) if !header.is_empty() => {
                let header_str = header.to_string();
                let input_bite = decode(header_str).unwrap();
                let input_header = String::from_utf8(input_bite).unwrap();

                return payload_input_deserialize(input_header, config).await;
            }
            Some(_) => {
                error!("wrong eval type");
                // logger::log::outcome_failure("wrong eval type")
                return Outcome::Error((
                    Status::BadRequest,
                    PayloadValidationError::ErrorWrongEvent,
                ));
            }
        }

        // get info request from body
        let input_body = match get_body(data).await {
            Ok(body) => body,
            Err(_e) => {
                return Outcome::Error((
                    Status::BadRequest,
                    PayloadValidationError::ErrorMissingBody,
                ))
            }
        };

        payload_input_deserialize(input_body, config).await
    }
}

#[cfg(test)]
mod resthttp1_guard_test {
    use std::sync::Arc;

    use rocket::{
        http::{Header, Status},
        local::blocking::Client,
    };
    use tokio::sync::RwLock;

    use rs_utils::config::Config;

    use crate::{config::OPAConfig, setup_rocket};

    static CLIENT_POAYLOAD: &str = r#"{
    "input": {
        "resource": "222",
        "role":"admin",
        "method": "get",
        "uri": "api/projects?projectId=dddd"
    },
    "data": {
        "id": "12a9733f-4e9c-4598-9f54-0db582223fce",
        "metadata_admin": {
            "organizations": [],
            "private_projects": [{
                "key": "222",
                "value": "admin"
            }],
            "scopes": [],
            "affiliate_projects": []
        },
        "schema_id": "person",
        "schema_url": "",
        "traits": {
            "email": "test2@wildcard.io",
            "name": {
                "first": "testy",
                "last": "testo"
            },
            "roles": {
                "organizations": [],
                "private_projects": [{
                    "key": "222",
                    "value": "admin"
                }],
                "scopes": [],
                "affiliate_projects": []
            } 
        }
    }
}"#;

    #[tokio::test]
    async fn test_get_eval() {
        let config = Arc::new(RwLock::new(OPAConfig::new("CONFIG").await));
        let client = Client::untracked(setup_rocket(config)).unwrap();
        let req = client
            .post("/")
            .header(Header::new("Content-Type", "application/json"))
            .body(CLIENT_POAYLOAD);
        let res = req.dispatch();
        assert_eq!(res.status(), Status::Ok)
    }


}
