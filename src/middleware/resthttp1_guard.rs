use std::{result, sync::Arc};

use anyhow::anyhow;
use base64::decode;
use log::{error, info, warn};
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

// #[allow(unused_imports)]
use crate::{
    config::OPAConfig,
    types::{kratos::Opadata, opa::Input},
    utils::error::send_error,
};

#[derive(Debug, Error)]
#[allow(clippy::enum_variant_names)]
///enum containing all error type for the payload validation
pub enum PayloadValidationError {
    #[error("the body is missing")]
    ErrorMissingBody,
    #[error("a parameter is missing")]
    ErrorMissingQuery,
    // #[error("failed to get the project id")]
    // ErrorFailedToParseRepositoryID(#[from] anyhow::Error),
    #[error("failed to deserialize: {0}")]
    ErrorFailedToDeserialize(#[from] serde_json::Error),
    #[error("payload as broken utf8 encoding: {0}")]
    ErrorMalformedPayload(#[from] std::io::Error),
    #[error("wrong eval argument name in the input payload")]
    ErrorWrongEvalName,
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
#[derive(Debug)]
pub struct PayloadGuard {
    pub(crate) input: Input,
    pub(crate) data: Opadata,
}

pub async fn get_data_roles_from_kratos<'r>(
    input: Input,
    config: &Arc<RwLock<OPAConfig>>,
) -> data::Outcome<'r, PayloadGuard> {
    // get config of the opa api rust
    let config_read = config.read().await;

    // curl kratos
    let url = match config_read.http.get("kratos") {
        Some(url) => url.to_owned() + &input.kratos as &str,
        None => {
            error!("error : url kratos is missing");
            return Outcome::Failure((
                Status::BadRequest,
                PayloadValidationError::ErrorMissingQuery,
            ));
        }
    };

    // verify kratos response
    let resp = match reqwest::get(url).await {
        Ok(resp) => resp,
        Err(e) => {
            error!("error while sending request to kratos: {e}");
            if let Err(_e) = send_error(&config_read.kafka, "error", &e).await {
                warn!("Failed to send error to kafka");
            };

            return Outcome::Failure((
                Status::BadRequest,
                PayloadValidationError::ErrorBadRequest(e),
            ));
        }
    };

    match resp.error_for_status_ref(){
        Ok(resp) => resp,
        Err(e) => {
            error!("kratos returned an error code: {e}");
            if let Err(_e) = send_error(&config_read.kafka, "error", &e).await {
                warn!("Failed to send error to kafka");
            };

            return Outcome::Failure((
                Status::BadRequest,
                PayloadValidationError::ErrorBadRequest(e),
            ));
        }
    };
    // get body text response and unmarchall to struct
    let data = match resp.json::<Opadata>().await {
        Ok(data) => data,
        Err(e) => {
            error!("User ID Kratos do not exist. No text request body: {e}");
            if let Err(_e) = send_error(&config_read.kafka, "error", &e).await {
                warn!("Failed to send error to kafka");
            };
            return Outcome::Failure((
                Status::Unauthorized,
                PayloadValidationError::ErrorBadRequest(e),
            ));
        }
    };

    // return the data and input struct by the payloadGuard
    Outcome::Success(PayloadGuard { input, data })
}

///Deserialize a push event.
pub async fn payload_input_deserialize<'r>(
    header: String,
    config: &Arc<RwLock<OPAConfig>>,
) -> data::Outcome<'r, PayloadGuard> {
    let config_read = config.read().await;
    // Deserializing payload input
    let input = match serde_json::from_str::<Input>(&header) {
        Ok(input) => input,
        Err(e) => {
            error!("error while deserializing the request input:{e}");
            if let Err(_e) = send_error(&config_read.kafka, "error", &e).await {
                warn!("Failed to send error to kafka");
            };
            return Outcome::Failure((
                Status::BadRequest,
                PayloadValidationError::ErrorFailedToDeserialize(e),
            ));
        }
    };

    // verify the type of evaluation
    if input.eval == "private_projects"
        || input.eval == "organizations"
        || input.eval == "scopes"
        || input.eval == "affiliate_projects"
    {
        // get role of the user from kratos
        get_data_roles_from_kratos(input, config).await
    } else {
        Outcome::Failure((Status::NotFound, PayloadValidationError::ErrorWrongEvalName))
    }
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
            Outcome::Failure(e) => {
                let error = anyhow!(e.0.reason().unwrap());
                return Outcome::Failure((e.0, PayloadValidationError::from(error)));
            }
            Outcome::Forward(_) => return Outcome::Forward(data),
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
                return Outcome::Failure((
                    Status::BadRequest,
                    PayloadValidationError::ErrorWrongEvent,
                ));
            }
        }

        // get info request from body
        let input_body = match get_body(data).await {
            Ok(body) => body,
            Err(_e) => {
                return Outcome::Failure((
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

    #[test]
    fn test_get_eval() {
        let config = Arc::new(RwLock::new(OPAConfig::new("CONFIG")));
        let client = Client::untracked(setup_rocket(config)).unwrap();
        let req = client
            .post("/")
            .header(Header::new("Content-Type", "application/json"))
            .body(CLIENT_POAYLOAD);
        let res = req.dispatch();
        assert_eq!(res.status(), Status::Ok)
    }

    static CLIENT_POAYLOAD: &str = r#"{"id": "5cf289bd-4990-42ee-89dd-12e31df15028"}"#;

    #[test]
    fn test_post_kratos_identity() {
        let config = Arc::new(RwLock::new(OPAConfig::new("CONFIG")));
        let client = Client::untracked(setup_rocket(config)).unwrap();
        let req = client
            .post("/kratos?id=5cf289bd-4990-42ee-89dd-12e31df15028")
            .header(Header::new("Content-Type", "application/json"))
            .body(IDENTITY_POAYLOAD);
        let res = req.dispatch();
        assert_eq!(res.status(), Status::Ok)
    }

    static IDENTITY_POAYLOAD: &str = r#"{"id": "5cf289bd-4990-42ee-89dd-12e31df15028"}"#;
}
