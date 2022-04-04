use std::time::Duration;

use anyhow::Result;
use once_cell::sync::Lazy;
use reqwest::{Body, Client, Response};

///This build the client only on the first call then return a reference to it.
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("failled to initialize HTTP client")
});

///This function make a PUT request to the given address,
///it can take any data that can be transformed into a Body.
pub async fn _put<T: Into<Body>>(addr: &str, body: T) -> Result<Response, reqwest::Error> {
    let response = HTTP_CLIENT
        .put(addr)
        .header("content-type", "application/json")
        .body(body)
        .send()
        .await?;
    response.error_for_status_ref()?;
    info!("PUT request sent");
    Ok(response)
}

///This function make a POST request to the given address,
///it can take any data that can be transformed into a Body.
pub async fn _post<T: Into<Body>>(addr: &str, body: T) -> Result<Response, reqwest::Error> {
    let response = HTTP_CLIENT.post(addr).body(body).send().await?;
    response.error_for_status_ref()?;
    info!("POST request sent");
    Ok(response)
}

///This function make a GET request to the given address.
pub async fn get(addr: &str) -> Result<Response, reqwest::Error> {
    let response = HTTP_CLIENT.get(addr).send().await?;
    response.error_for_status_ref()?;
    info!("GET request sent");
    Ok(response)
}

///This function make a DELETE request to the given address.
pub async fn _delete(addr: &str) -> Result<Response, reqwest::Error> {
    let response = HTTP_CLIENT.delete(addr).send().await?;
    response.error_for_status_ref()?;
    info!("DELETE request sent");
    Ok(response)
}

#[cfg(test)]
mod test_http_client {
    use super::*;
    use httpmock::prelude::*;
    use reqwest::StatusCode;

    #[rocket::async_test]
    async fn test_post() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|whent, then| {
                whent.method(POST).path("/examples");
                then.status(200).body(r#"{"examples":examples}"#);
            })
            .await;
        let response = _post(&server.url("/examples"), r#"{"examples":examples}"#)
            .await
            .unwrap();
        mock.assert();
        assert_eq!(response.status(), StatusCode::OK)
    }

    #[rocket::async_test]
    async fn test_put() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|whent, then| {
                whent.method(PUT).path("/examples");
                then.status(200).body(r#"{"examples":examples}"#);
            })
            .await;
        let response = _put(&server.url("/examples"), r#"{"examples":3}"#).await.unwrap();
        mock.assert_async().await;
        assert_eq!(response.status(), StatusCode::OK)
    }

    #[rocket::async_test]
    async fn test_get() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|whent, then| {
                whent.method(GET).path("/examples");
                then.status(200);
            })
            .await;
        let response = get(&server.url("/examples")).await.unwrap();
        mock.assert_async().await;
        assert_eq!(response.status(), StatusCode::OK)
    }

    #[rocket::async_test]
    async fn test_delete() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|whent, then| {
                whent.method(DELETE).path("/examples");
                then.status(200);
            })
            .await;
        let response = _delete(&server.url("/examples")).await.unwrap();
        mock.assert_async().await;
        assert_eq!(response.status(), StatusCode::OK)
    }
}
