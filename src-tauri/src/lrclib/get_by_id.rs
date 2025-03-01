use std::time::Duration;

use crate::utils::strip_timestamp;
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RawResponse {
    pub plain_lyrics: Option<String>,
    pub synced_lyrics: Option<String>,
    pub instrumental: bool,
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

#[derive(Error, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
#[error("{error}: {message}")]
pub struct ResponseError {
    status_code: Option<u16>,
    error: String,
    message: String,
}

async fn make_request(id: i64, lrclib_instance: &str) -> Result<reqwest::Response> {
    let version = env!("CARGO_PKG_VERSION");
    let user_agent = format!(
        "LRCGET v{} (https://github.com/tranxuanthang/lrcget)",
        version
    );
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(user_agent)
        .build()?;
    let api_endpoint = format!("{}/api/get/{}", lrclib_instance.trim_end_matches('/'), id);
    Ok(client.get(&api_endpoint).send().await?)
}

pub async fn request_raw(id: i64, lrclib_instance: &str) -> Result<RawResponse> {
    let res = make_request(id, lrclib_instance).await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let lrclib_response = res.json::<RawResponse>().await?;

            if lrclib_response.synced_lyrics.is_some()
                || lrclib_response.plain_lyrics.is_some()
                || lrclib_response.instrumental
            {
                Ok(lrclib_response)
            } else {
                Err(ResponseError {
                    status_code: Some(404),
                    error: "NotFound".to_string(),
                    message: "There are no lyrics for this track".to_string(),
                }
                .into())
            }
        }

        reqwest::StatusCode::NOT_FOUND => Err(ResponseError {
            status_code: Some(404),
            error: "NotFound".to_string(),
            message: "There are no lyrics for this track".to_string(),
        }
        .into()),

        reqwest::StatusCode::BAD_REQUEST
        | reqwest::StatusCode::SERVICE_UNAVAILABLE
        | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            let error = res.json::<ResponseError>().await?;
            Err(error.into())
        }

        _ => Err(ResponseError {
            status_code: None,
            error: "UnknownError".to_string(),
            message: "An unknown error occurred".to_string(),
        }
        .into()),
    }
}

pub async fn request(id: i64, lrclib_instance: &str) -> Result<Response> {
    let res = make_request(id, lrclib_instance).await?;

    match res.status() {
        reqwest::StatusCode::OK => {
            let lrclib_response = res.json::<RawResponse>().await?;
            Ok(Response::from_raw_response(lrclib_response))
        }

        reqwest::StatusCode::NOT_FOUND => Ok(Response::None),

        reqwest::StatusCode::BAD_REQUEST
        | reqwest::StatusCode::SERVICE_UNAVAILABLE
        | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            let error = res.json::<ResponseError>().await?;
            Err(error.into())
        }

        _ => Err(ResponseError {
            status_code: None,
            error: "UnknownError".to_string(),
            message: "An unknown error occurred".to_string(),
        }
        .into()),
    }
}

use crate::utils::strip_timestamp;
use anyhow::Result;
use reqwest;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use thiserror::Error;

// Reusing similar structures from get.rs
use super::get::{RawResponse, Response, ResponseError};

async fn make_request(id: i64, lrclib_instance: &str) -> Result<reqwest::Response> {
    let version = env!("CARGO_PKG_VERSION");
    let user_agent = format!("LRCGET/{}", version);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(10))
        .user_agent(user_agent)
        .build()?;
    let api_endpoint = format!("{}/api/get/{}", lrclib_instance.trim_end_matches('/'), id);
    Ok(client.get(&api_endpoint).send().await?)
}

pub async fn request_by_id(id: i64, lrclib_instance: &str) -> Result<Response> {
    let response = make_request(id, lrclib_instance).await?;

    match response.status() {
        reqwest::StatusCode::OK => {
            let lrclib_response = response.json::<RawResponse>().await?;
            Ok(Response::from_raw_response(lrclib_response))
        }

        reqwest::StatusCode::NOT_FOUND => Ok(Response::None),

        reqwest::StatusCode::BAD_REQUEST
        | reqwest::StatusCode::SERVICE_UNAVAILABLE
        | reqwest::StatusCode::INTERNAL_SERVER_ERROR => {
            let error = response.json::<ResponseError>().await?;
            Err(error.into())
        }

        _ => Err(ResponseError {
            status_code: None,
            error: "UnknownError".to_string(),
            message: "An unknown error occurred".to_string(),
        }
        .into()),
    }
}
