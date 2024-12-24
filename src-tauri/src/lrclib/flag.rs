use std::time::Duration;

use serde::{Deserialize,Serialize};
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  track_id: i64,
  reason: String
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn request(track_id: i64, reason: &str, publish_token: &str, lrclib_instance: &str) -> Result<()> {
  let data = Request {
    track_id,
    reason: reason.to_owned()
  };

  let version = env!("CARGO_PKG_VERSION");
  let user_agent = format!("LRCGET v{} (https://github.com/tranxuanthang/lrcget)", version);
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .user_agent(user_agent)
    .build()?;
  let api_endpoint = format!("{}/api/flag", lrclib_instance.trim_end_matches('/'));
  let url = reqwest::Url::parse(&api_endpoint)?;
  let res = client.post(url).header("X-Publish-Token", publish_token).json(&data).send().await?;

  match res.status() {
    reqwest::StatusCode::CREATED => {
      Ok(())
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
