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
    instrumental: bool,
    #[serde(default)]
    pub has_word_sync: bool,
    lang: Option<String>,
    isrc: Option<String>,
    spotify_id: Option<String>,
    name: Option<String>,
    album_name: Option<String>,
    artist_name: Option<String>,
    release_date: Option<String>,
    duration: Option<f64>,
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

fn build_params(
    title: &str,
    album_name: &str,
    artist_name: &str,
    duration: f64,
) -> Result<Vec<(String, String)>, LrclibError> {
    let title = title.trim();
    let artist_name = artist_name.trim();
    let album_name = album_name.trim();

    if title.is_empty() {
        return Err(LrclibError::validation("Track name is required"));
    }
    if artist_name.is_empty() {
        return Err(LrclibError::validation("Artist name is required"));
    }

    let mut params = Vec::with_capacity(4);
    params.push(("track_name".to_owned(), title.to_owned()));
    params.push(("artist_name".to_owned(), artist_name.to_owned()));

    if !album_name.is_empty() {
        params.push(("album_name".to_owned(), album_name.to_owned()));
    }

    let rounded_duration = duration.round();
    if (1.0..=3600.0).contains(&rounded_duration) {
        params.push(("duration".to_owned(), rounded_duration.to_string()));
    }

    Ok(params)
}

async fn make_request(
    title: &str,
    album_name: &str,
    artist_name: &str,
    duration: f64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<reqwest::Response, LrclibError> {
    let params = build_params(title, album_name, artist_name, duration)?;

    client::get(lrclib_instance, "/api/get", &params, policy).await
}

pub async fn request_raw(
    title: &str,
    album_name: &str,
    artist_name: &str,
    duration: f64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<RawResponse, LrclibError> {
    let res = make_request(
        title,
        album_name,
        artist_name,
        duration,
        lrclib_instance,
        policy,
    )
    .await?;

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
    title: &str,
    album_name: &str,
    artist_name: &str,
    duration: f64,
    lrclib_instance: &str,
    policy: RetryPolicy,
) -> Result<Response, LrclibError> {
    let res = match make_request(
        title,
        album_name,
        artist_name,
        duration,
        lrclib_instance,
        policy,
    )
    .await
    {
        Ok(res) => res,
        Err(error) if error.kind == LrclibErrorKind::NotFound => return Ok(Response::None),
        Err(error) => return Err(error),
    };

    let lrclib_response: RawResponse = client::parse_json(res).await?;

    Ok(Response::from_raw_response(lrclib_response))
}

#[cfg(test)]
mod tests {
    use super::build_params;

    #[test]
    fn omits_empty_optional_get_params() {
        let params = build_params("Song", "", "Artist", 0.0).unwrap();

        assert_eq!(
            params,
            vec![
                ("track_name".to_owned(), "Song".to_owned()),
                ("artist_name".to_owned(), "Artist".to_owned()),
            ]
        );
    }

    #[test]
    fn includes_album_and_valid_duration() {
        let params = build_params("Song", "Album", "Artist", 233.4).unwrap();

        assert!(params.contains(&("album_name".to_owned(), "Album".to_owned())));
        assert!(params.contains(&("duration".to_owned(), "233".to_owned())));
    }

    #[test]
    fn omits_duration_outside_documented_range() {
        for duration in [0.0, -1.0, 3601.0, f64::NAN, f64::INFINITY] {
            let params = build_params("Song", "", "Artist", duration).unwrap();
            assert!(!params.iter().any(|(key, _)| key.as_str() == "duration"));
        }
    }

    #[test]
    fn rejects_missing_required_fields() {
        assert!(build_params("", "", "Artist", 100.0).is_err());
        assert!(build_params("Song", "", "", 100.0).is_err());
    }

    #[test]
    fn trims_required_and_optional_values() {
        let params = build_params("  Song  ", "  Album  ", "  Artist  ", 100.0).unwrap();

        assert!(params.contains(&("track_name".to_owned(), "Song".to_owned())));
        assert!(params.contains(&("artist_name".to_owned(), "Artist".to_owned())));
        assert!(params.contains(&("album_name".to_owned(), "Album".to_owned())));
    }
}
