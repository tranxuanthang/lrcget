use serde::Deserialize;
use anyhow::Result;
use reqwest;
use thiserror::Error;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LrclibResponse {
  plain_lyrics: Option<String>,
  synced_lyrics: Option<String>,
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

pub enum LrclibResponseOption {
  SyncedLyrics(String),
  UnsyncedLyrics(String),
  IsInstrumental,
  None
}

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
struct LrclibResponseError {
  status_code: Option<u16>,
  error: String,
  message: String
}

pub async fn retrieve_lyrics(title: &str, album_name: &str, artist_name: &str, duration: f64) -> Result<LrclibResponseOption> {
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
      let lrclib_response = res.json::<LrclibResponse>().await?;

      match lrclib_response.synced_lyrics {
        Some(synced_lyrics) => {
          Ok(LrclibResponseOption::SyncedLyrics(synced_lyrics))
        },
        None => {
          match lrclib_response.plain_lyrics {
            Some(unsynced_lyrics) => {
              Ok(LrclibResponseOption::UnsyncedLyrics(unsynced_lyrics))
            }
            None => {
              if lrclib_response.instrumental {
                Ok(LrclibResponseOption::IsInstrumental)
              } else {
                Ok(LrclibResponseOption::None)
              }
            }
          }
        }
      }
    },

    reqwest::StatusCode::NOT_FOUND => {
      Ok(LrclibResponseOption::None)
    }

    reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::SERVICE_UNAVAILABLE | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
      let error = res.json::<LrclibResponseError>().await?;
      Err(error.into())
    },

    _ => {
      Err(LrclibResponseError {
        status_code: None,
        error: "UnknownError".to_string(),
        message: "Unknown error happened".to_string()
      }.into())
    }
  }
}
