use axum::{
    http::{header::ToStrError, StatusCode},
    response::{IntoResponse, Response},
};
use thiserror::Error;
use tracing::error;
// use vaultrs::sys::ServerStatus;

use crate::utils::error::SendError;

///handler for error in the http service
///it convert the recevied error in a response
#[derive(Error, Debug)]
pub enum RouterError {
    #[error("failed to serialize data")]
    Serialisation(#[from] serde_json::Error),
    #[error("failed to apply identity patch")]
    Internal(#[from] anyhow::Error),
    #[error("failled to convert to string")]
    StrConvert(#[from] ToStrError),
    #[error("should never be empty.")]
    EmptyResponse,
}

#[cfg(not(tarpaulin_include))]
impl SendError for RouterError {}

#[cfg(not(tarpaulin_include))]
impl IntoResponse for RouterError {
    fn into_response(self) -> Response {
        match self {
            RouterError::Serialisation(e) => {
                error!("{:?}", e);
                #[cfg(test)]
                let status_string = format!("INTERNAL_SERVER_ERROR {e}");
                #[cfg(not(test))]
                let status_string = "INTERNAL_SERVER_ERROR";
                (StatusCode::INTERNAL_SERVER_ERROR, status_string).into_response()
            }
            RouterError::Internal(e) => {
                error!("{:?}", e);
                #[cfg(test)]
                let status_string = format!("INTERNAL_SERVER_ERROR {e}");
                #[cfg(not(test))]
                let status_string = "INTERNAL_SERVER_ERROR";
                (StatusCode::INTERNAL_SERVER_ERROR, status_string).into_response()
            }
            RouterError::StrConvert(e) => {
                error!("{:?}, while converting str", e);
                #[cfg(test)]
                let status_string = format!("INTERNAL_SERVER_ERROR {e}");
                #[cfg(not(test))]
                let status_string = "INTERNAL_SERVER_ERROR";
                (StatusCode::INTERNAL_SERVER_ERROR, status_string).into_response()
            }
            RouterError::EmptyResponse => {
                error!("opa returned empty response");
                (StatusCode::INTERNAL_SERVER_ERROR, "INTERNAL_SERVER_ERROR").into_response()
            }
        }
    }
}
