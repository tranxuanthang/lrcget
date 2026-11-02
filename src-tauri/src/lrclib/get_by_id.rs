use crate::lrclib::client::{self, LrclibError, LrclibErrorKind, RetryPolicy};
use crate::utils::strip_timestamp;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawResponse {
    pub id: Option<i64>,
    pub track_name: Option<String>,
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub lyricsfile: Option<String>,
    pub instrumental: bool,
    #[serde(default)]
    pub has_word_sync: bool,
    pub lang: Option<String>,
    pub isrc: Option<String>,
    pub spotify_id: Option<String>,
    pub name: Option<String>,
    pub album_name: Option<String>,
    pub artist_name: Option<String>,
    pub release_date: Option<String>,
    pub duration: Option<f64>,
}

#[derive(Serialize)]
#[serde(tag = "type", content = "lyrics")]
pub enum Response {
    SyncedLyrics(String, String),
    UnsyncedLyrics(String),
    IsInstrumental,
    None,
}

impl Response {
    pub fn from_raw_response(lrclib_response: RawResponse) -> Response {
        match lrclib_response.synced_lyrics {
            Some(synced_lyrics) => {
                let plain_lyrics = match lrclib_response.plain_lyrics {
                    Some(plain_lyrics) => plain_lyrics,
                    None => strip_timestamp(&synced_lyrics),
                };
                Response::SyncedLyrics(synced_lyrics, plain_lyrics)
            }
            None => match lrclib_response.plain_lyrics {
                Some(unsynced_lyrics) => Response::UnsyncedLyrics(unsynced_lyrics),
                None => {
                    if lrclib_response.instrumental {
                        Response::IsInstrumental
                    } else {
                        Response::None
                    }
                }
            },
        }
    }
}

async fn make_request(
    id: i64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<reqwest::Response, LrclibError> {
    let path = format!("/api/get/{}", id);
    client::get(lrclib_instance, &path, &[], policy).await
}

pub async fn request_raw(
    id: i64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<RawResponse, LrclibError> {
    let res = make_request(id, lrclib_instance, policy).await?;

    let lrclib_response: RawResponse = client::parse_json(res).await?;

    if lrclib_response.synced_lyrics.is_some()
        || lrclib_response.plain_lyrics.is_some()
        || lrclib_response.lyricsfile.is_some()
        || lrclib_response.instrumental
    {
        Ok(lrclib_response)
    } else {
        Err(LrclibError::not_found())
    }
}

pub async fn request(
    id: i64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<Response, LrclibError> {
    let res = match make_request(id, lrclib_instance, policy).await {
        Ok(res) => res,
        Err(error) if error.kind == LrclibErrorKind::NotFound => return Ok(Response::None),
        Err(error) => return Err(error),
    };

    let lrclib_response: RawResponse = client::parse_json(res).await?;

    Ok(Response::from_raw_response(lrclib_response))
}
