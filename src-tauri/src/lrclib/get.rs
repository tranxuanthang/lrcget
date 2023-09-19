use serde::{Deserialize,Serialize};
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawResponse {
  pub plain_lyrics: Option<String>,
  pub synced_lyrics: Option<String>,
  instrumental: bool,
  lang: Option<String>,
  isrc: Option<String>,
  spotify_id: Option<String>,
  name: Option<String>,
  album_name: Option<String>,
  artist_name: Option<String>,
  release_date: Option<String>,
  duration: Option<f64>
}

#[derive(Serialize)]
#[serde(tag = "type", content = "lyrics")]
pub enum Response {
  SyncedLyrics(String),
  UnsyncedLyrics(String),
  IsInstrumental,
  None
}

impl Response {
  pub fn from_raw_response(lrclib_response: RawResponse) -> Response {
    match lrclib_response.synced_lyrics {
      Some(synced_lyrics) => {
        Response::SyncedLyrics(synced_lyrics)
      },
      None => {
        match lrclib_response.plain_lyrics {
          Some(unsynced_lyrics) => {
            Response::UnsyncedLyrics(unsynced_lyrics)
          }
          None => {
            if lrclib_response.instrumental {
              Response::IsInstrumental
            } else {
              Response::None
            }
          }
        }
      }
    }
  }
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn request_raw(title: &str, album_name: &str, artist_name: &str, duration: f64) -> Result<RawResponse> {
  let params: Vec<(String, String)> = vec![
    ("artist_name".to_owned(), artist_name.to_owned()),
    ("track_name".to_owned(), title.to_owned()),
    ("album_name".to_owned(), album_name.to_owned()),
    ("duration".to_owned(), duration.round().to_string())
  ];

  let url = reqwest::Url::parse_with_params("https://lrclib.net/api/get", &params)?;
  let res = reqwest::get(url).await?;

  match res.status() {
    reqwest::StatusCode::OK => {
      let lrclib_response = res.json::<RawResponse>().await?;

      if lrclib_response.synced_lyrics.is_some() || lrclib_response.plain_lyrics.is_some() {
        Ok(lrclib_response)
      } else {
        Err(ResponseError {
          status_code: Some(404),
          error: "NotFound".to_string(),
          message: "There is no lyrics for this track".to_string()
        }.into())
      }
    }

    reqwest::StatusCode::NOT_FOUND => {
      Err(ResponseError {
        status_code: Some(404),
        error: "NotFound".to_string(),
        message: "There is no lyrics for this track".to_string()
      }.into())
    }

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

pub async fn request(title: &str, album_name: &str, artist_name: &str, duration: f64) -> Result<Response> {
  let params: Vec<(String, String)> = vec![
    ("artist_name".to_owned(), artist_name.to_owned()),
    ("track_name".to_owned(), title.to_owned()),
    ("album_name".to_owned(), album_name.to_owned()),
    ("duration".to_owned(), duration.round().to_string())
  ];

  let url = reqwest::Url::parse_with_params("https://lrclib.net/api/get", &params)?;
  let res = reqwest::get(url).await?;

  match res.status() {
    reqwest::StatusCode::OK => {
      let lrclib_response = res.json::<RawResponse>().await?;

      Ok(Response::from_raw_response(lrclib_response))
    },

    reqwest::StatusCode::NOT_FOUND => {
      Ok(Response::None)
    }

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
