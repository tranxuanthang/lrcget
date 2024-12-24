use std::time::Duration;

use anyhow::Result;
use reqwest;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub prefix: String,
    pub target: String,
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
    status_code: Option<u16>,
    error: String,
    message: String,
}

pub async fn request(lrclib_instance: &str) -> Result<Response> {
    let version = env!("CARGO_PKG_VERSION");
    let user_agent = format!(
        "LRCGET v{} (https://github.com/tranxuanthang/lrcget)",
        version
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(user_agent)
        .build()?;
    let api_endpoint = format!(
        "{}/api/request-challenge",
        lrclib_instance.trim_end_matches('/')
    );
    let url = reqwest::Url::parse(&api_endpoint)?;
    let res = client.post(url).send().await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let response = res.json::<Response>().await?;
            Ok(response)
        }

        reqwest::StatusCode::BAD_REQUEST
        | reqwest::StatusCode::SERVICE_UNAVAILABLE
        | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            let error = res.json::<ResponseError>().await?;
            Err(error.into())
        }

        _ => Err(ResponseError {
            status_code: None,
            error: "UnknownError".to_string(),
            message: "Unknown error happened".to_string(),
        }
        .into()),
    }
}
