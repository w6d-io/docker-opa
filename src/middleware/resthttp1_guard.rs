use base64::decode;
use rocket::{
    data::{self, Data, FromData, Outcome, ToByteUnit},
    http::Status,
    request::Request,
    serde::json::serde_json,
};
use std::result;
use tonic::{Request as TonicRequest, Status as TonicStatus};

use thiserror::Error;

#[allow(unused_imports)]
use crate::{
    config::{CONFIG, DATA},
    open_policy_agency::InputRequest,
    types::Input,
    types::Opadata,
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
    #[error("failed to get the project id")]
    ErrorFailedToParseRepositoryID(#[from] anyhow::Error),
    #[error("failed to deserialize")]
    ErrorFailedToDeserialize(#[from] serde_json::Error),
    #[error("payload as broken utf8 encoding")]
    ErrorMalformedPayload(#[from] std::io::Error),
    #[error("wrong eval argument name in the input payload")]
    ErrorWrongEvalName,
    #[error("the body is to big")]
    ErrorBodyToBig,
    #[error("wrong field type in the input payload")]
    ErrorWrongEvent,
    #[error("bad request")]
    ErrorBadRequest(#[from] reqwest::Error),
}

type Result<T, E = PayloadValidationError> = result::Result<T, E>;

// the response struct
#[derive(Debug)]
pub struct PayloadGuard {
    pub(crate) input: Input,
    pub(crate) data: Opadata,
}

pub async fn get_data_roles_from_kratos<'r>(input: Input) -> data::Outcome<'r, PayloadGuard> {
    // get config of the opa api rust
    let config = &CONFIG.read().await;

    // curl kratos
    let url = match config.http.get("kratos") {
        Some(url) => url.to_owned() + &input.kratos,
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
            error!("error while run request to kratos: {e}");
            if let Err(_e) = send_error("error", &e).await {
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
            error!("User ID Kratos not exist. Not get text request body: {e}");
            if let Err(_e) = send_error("error", &e).await {
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
pub async fn payload_input_deserialize<'r>(header: String) -> data::Outcome<'r, PayloadGuard> {
    // Deserializing payload input
    let input = match serde_json::from_str::<Input>(&header) {
        Ok(input) => input,
        Err(e) => {
            error!("error while deserializing the request input:{e}");
            if let Err(_e) = send_error("error", &e).await {
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
        get_data_roles_from_kratos(input).await
    } else {
        return Outcome::Failure((Status::NotFound, PayloadValidationError::ErrorWrongEvalName));
    }
}

///Get body from GRPC request.
///will fail if:
/// - the body is not a valid utf8 string.
/// - the body is empty.
/// - the body is to big.
pub async fn from_data_grpc(
    body: TonicRequest<InputRequest>,
) -> Result<PayloadGuard, tonic::Status> {
    let grpc_params = body.into_inner();
    let input = Input {
        eval: grpc_params.eval,
        kratos: grpc_params.kratos,
        resource: grpc_params.resource,
        method: grpc_params.method,
        uri: grpc_params.uri,
        role: grpc_params.role,
    };

    let result = match get_data_roles_from_kratos(input).await {
        Outcome::Success(res) => res,
        Outcome::Failure((_, PayloadValidationError::ErrorMissingQuery)) => {
            return Err(TonicStatus::unavailable(
                PayloadValidationError::ErrorMissingQuery.to_string(),
            ))
        }
        Outcome::Failure((_, PayloadValidationError::ErrorBadRequest(e))) => {
            return Err(TonicStatus::unavailable(e.to_string()))
        }
        _ => return Err(TonicStatus::unknown("Unknown error")),
    };

    Ok(result)
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
        // get info request from header
        let header = match req.headers().get_one("input") {
            Some(header) => Some(header),
            None => req.headers().get_one("input"),
        };
        match header {
            None => {
                info!("{}", "Header is empty")
            }
            Some(header) if !header.is_empty() => {
                let header_str = header.to_string();
                let input_bite = decode(header_str).unwrap();
                let input_header = String::from_utf8(input_bite).unwrap();

                return payload_input_deserialize(input_header).await;
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

        payload_input_deserialize(input_body).await
    }
}

#[cfg(test)]
mod resthttp1_guard_test {

    use rocket::http::{Header, Status};
    use rocket::local::blocking::Client;

    use crate::setup_rocket;

    #[test]
    fn test_get_eval() {
        let client = Client::untracked(setup_rocket()).unwrap();
        let req = client
            .get("/eval")
            .header(Header::new("Content-Type", "application/json"))
            .body(CLIENT_POAYLOAD);
        let res = req.dispatch();
        assert_eq!(res.status(), Status::Ok)
    }

    static CLIENT_POAYLOAD: &str = r#"{"id": "5cf289bd-4990-42ee-89dd-12e31df15028"}"#;

    #[test]
    fn test_post_kratos_identity() {
        let client = Client::untracked(setup_rocket()).unwrap();
        let req = client
            .post("/kratos?id=5cf289bd-4990-42ee-89dd-12e31df15028")
            .header(Header::new("Content-Type", "application/json"))
            .body(IDENTITY_POAYLOAD);
        let res = req.dispatch();
        assert_eq!(res.status(), Status::Ok)
    }

    static IDENTITY_POAYLOAD: &str = r#"{"id": "5cf289bd-4990-42ee-89dd-12e31df15028"}"#;
}
