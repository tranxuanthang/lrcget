use crate::lrclib::client::{self, LrclibError, RetryPolicy};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    track_name: String,
    album_name: String,
    artist_name: String,
    duration: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    plain_lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    synced_lyrics: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    lyricsfile: Option<String>,
}

pub async fn request(
    title: &str,
    album_name: &str,
    artist_name: &str,
    duration: f64,
    plain_lyrics: Option<&str>,
    synced_lyrics: Option<&str>,
    lyricsfile: Option<&str>,
    publish_token: &str,
    lrclib_instance: &str,
) -> Result<(), LrclibError> {
    let data = Request {
        artist_name: artist_name.to_owned(),
        track_name: title.to_owned(),
        album_name: album_name.to_owned(),
        duration: duration.round(),
        plain_lyrics: plain_lyrics.map(str::to_owned),
        synced_lyrics: synced_lyrics.map(str::to_owned),
        lyricsfile: lyricsfile.map(str::to_owned),
    };

    client::post_json(
        lrclib_instance,
        "/api/publish",
        &data,
        &[("X-Publish-Token", publish_token)],
        RetryPolicy::NoRetry,
    )
    .await?;

    Ok(())
}
