use serde::{Deserialize,Serialize};
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchItem {
  id: i64,
  name: Option<String>,
  artist_name: Option<String>,
  album_name: Option<String>,
  duration: Option<f64>,
  instrumental: bool,
  plain_lyrics: Option<String>,
  synced_lyrics: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Response(Vec<SearchItem>);

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn request(title: &str, album_name: &str, artist_name: &str) -> Result<Response> {
  let params: Vec<(String, String)> = vec![
    ("track_name".to_owned(), title.to_owned()),
    ("artist_name".to_owned(), artist_name.to_owned()),
    ("album_name".to_owned(), album_name.to_owned())
  ];

  let url = reqwest::Url::parse_with_params("https://lrclib.net/api/search", &params)?;
  let res = reqwest::get(url).await?;

  match res.status() {
    reqwest::StatusCode::OK => {
      let lrclib_response = res.json::<Response>().await?;
      Ok(lrclib_response)
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
