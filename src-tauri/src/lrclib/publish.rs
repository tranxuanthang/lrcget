use std::time::Duration;

use serde::{Deserialize,Serialize};
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
  track_name: String,
  album_name: String,
  artist_name: String,
  duration: f64,
  plain_lyrics: String,
  synced_lyrics: String
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn request(title: &str, album_name: &str, artist_name: &str, duration: f64, plain_lyrics: &str, synced_lyrics: &str, publish_token: &str) -> Result<()> {
  let data = Request {
    artist_name: artist_name.to_owned(),
    track_name: title.to_owned(),
    album_name: album_name.to_owned(),
    duration: duration.round(),
    plain_lyrics: plain_lyrics.to_owned(),
    synced_lyrics: synced_lyrics.to_owned(),
  };

  let version = env!("CARGO_PKG_VERSION");
  let user_agent = format!("LRCGET v{} (https://github.com/tranxuanthang/lrcget)", version);
  let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(10))
    .user_agent(user_agent)
    .build()?;
  let url = reqwest::Url::parse("https://lrclib.net/api/publish")?;
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
