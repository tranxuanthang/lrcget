use serde::Deserialize;
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
  pub prefix: String,
  pub target: String
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn request() -> Result<Response> {
  let client = reqwest::Client::new();
  let url = reqwest::Url::parse("https://lrclib.net/api/request-challenge")?;
  let res = client.post(url).send().await?;

  match res.status() {
    reqwest::StatusCode::OK => {
      let response = res.json::<Response>().await?;
      Ok(response)
    },

    reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::SERVICE_UNAVAILABLE | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
      let error = res.json::<ResponseError>().await?;
      Err(error.into())
    },

    _ => {
      Err(ResponseError {
        status_code: None,
        error: "UnknownError".to_string(),
        message: "Unknown error happened".to_string()
      }.into())
    }
  }
}
