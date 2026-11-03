use crate::lrclib::client::{self, LrclibError, LrclibErrorKind, RetryPolicy};
use serde::{Deserialize, Serialize};

const SEARCH_V2_PATH: &str = "/api/v2/search";
const SEARCH_V1_PATH: &str = "/api/search";

/// Lightweight track metadata returned by the v2 search endpoint.
///
/// The v2 API returns availability flags instead of lyrics text. Callers must
/// fetch the selected track's lyrics with `/api/get/{id}` when needed.
#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TrackSummary {
    pub id: i64,
    pub track_name: Option<String>,
    pub artist_name: Option<String>,
    pub album_name: Option<String>,
    pub duration: Option<f64>,
    #[serde(default)]
    pub instrumental: bool,
    #[serde(default)]
    pub has_word_sync: bool,
    #[serde(default)]
    pub has_plain_lyrics: bool,
    #[serde(default)]
    pub has_synced_lyrics: bool,
    #[serde(default)]
    pub has_lyricsfile: bool,
}

/// Shape returned by the v1 `/api/search` endpoint, used as a fallback for
/// instances that predate `/api/v2/search`.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacySearchItem {
    id: i64,
    name: Option<String>,
    track_name: Option<String>,
    artist_name: Option<String>,
    album_name: Option<String>,
    duration: Option<f64>,
    #[serde(default)]
    instrumental: bool,
    #[serde(default)]
    has_word_sync: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
    lyricsfile: Option<String>,
}

impl LegacySearchItem {
    fn into_summary(self) -> TrackSummary {
        TrackSummary {
            id: self.id,
            track_name: self.track_name.or(self.name),
            artist_name: self.artist_name,
            album_name: self.album_name,
            duration: self.duration,
            instrumental: self.instrumental,
            has_word_sync: self.has_word_sync,
            has_plain_lyrics: has_content(self.plain_lyrics.as_deref()),
            has_synced_lyrics: has_content(self.synced_lyrics.as_deref()),
            has_lyricsfile: has_content(self.lyricsfile.as_deref()),
        }
    }
}

fn has_content(value: Option<&str>) -> bool {
    value.is_some_and(|value| !value.trim().is_empty())
}

fn build_params(
    title: &str,
    album_name: &str,
    artist_name: &str,
    q: &str,
) -> Vec<(String, String)> {
    let q = q.trim();
    if !q.is_empty() {
        return vec![("q".to_owned(), q.to_owned())];
    }

    let title = title.trim();
    if title.is_empty() {
        return Vec::new();
    }

    let mut params = Vec::with_capacity(3);
    params.push(("track_name".to_owned(), title.to_owned()));

    let artist_name = artist_name.trim();
    if !artist_name.is_empty() {
        params.push(("artist_name".to_owned(), artist_name.to_owned()));
    }

    let album_name = album_name.trim();
    if !album_name.is_empty() {
        params.push(("album_name".to_owned(), album_name.to_owned()));
    }

    params
}

/// A v2 request is only worth retrying against v1 when the instance does not
/// serve the v2 route at all. Rate limits, overloads, and network failures
/// should surface to the caller instead of doubling server load.
fn v2_unsupported(error: &LrclibError) -> bool {
    error.kind == LrclibErrorKind::NotFound || error.name == "InvalidResponse"
}

async fn request_v2(
    params: &[(String, String)],
    lrclib_instance: &str,
) -> Result<Vec<TrackSummary>, LrclibError> {
    let res = client::get(
        lrclib_instance,
        SEARCH_V2_PATH,
        params,
        RetryPolicy::Interactive,
    )
    .await?;

    client::parse_json(res).await
}

async fn request_v1(
    params: &[(String, String)],
    lrclib_instance: &str,
) -> Result<Vec<TrackSummary>, LrclibError> {
    let res = client::get(
        lrclib_instance,
        SEARCH_V1_PATH,
        params,
        RetryPolicy::Interactive,
    )
    .await?;

    let items: Vec<LegacySearchItem> = client::parse_json(res).await?;
    Ok(items
        .into_iter()
        .map(LegacySearchItem::into_summary)
        .collect())
}

pub async fn request(
    title: &str,
    album_name: &str,
    artist_name: &str,
    q: &str,
    lrclib_instance: &str,
) -> Result<Vec<TrackSummary>, LrclibError> {
    let params = build_params(title, album_name, artist_name, q);
    if params.is_empty() {
        return Ok(Vec::new());
    }

    match request_v2(&params, lrclib_instance).await {
        Ok(summaries) => Ok(summaries),
        Err(error) if v2_unsupported(&error) => request_v1(&params, lrclib_instance).await,
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        build_params, request, v2_unsupported, LegacySearchItem, TrackSummary, SEARCH_V1_PATH,
        SEARCH_V2_PATH,
    };
    use crate::lrclib::client::{LrclibError, LrclibErrorKind};
    use serde_json::json;
    use std::sync::{Arc, Mutex as StdMutex};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn spawn_mock_server(
        responses: Vec<(u16, &'static str)>,
    ) -> (String, Arc<StdMutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(StdMutex::new(Vec::new()));
        let captured_requests = Arc::clone(&requests);

        tokio::spawn(async move {
            for (status, body) in responses {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };

                let mut request = Vec::new();
                let mut buffer = [0u8; 1024];

                loop {
                    let read = stream.read(&mut buffer).await.unwrap_or(0);
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }

                captured_requests
                    .lock()
                    .unwrap()
                    .push(String::from_utf8_lossy(&request).into_owned());

                let reason = match status {
                    200 => "OK",
                    404 => "Not Found",
                    503 => "Service Unavailable",
                    _ => "Unknown",
                };

                let response = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    status,
                    reason,
                    body.len(),
                    body
                );

                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.shutdown().await;
            }
        });

        (format!("http://{}", address), requests)
    }

    fn error(kind: LrclibErrorKind, name: &str) -> LrclibError {
        LrclibError {
            kind,
            status: None,
            name: name.to_owned(),
            message: String::new(),
            retry_after: None,
            scope: None,
        }
    }

    #[test]
    fn q_takes_precedence_over_structured_fields() {
        let params = build_params("Song", "Album", "Artist", "  hello  ");

        assert_eq!(params, vec![("q".to_owned(), "hello".to_owned())]);
    }

    #[test]
    fn structured_search_omits_empty_optional_fields() {
        let params = build_params("Song", "Album", "", "");

        assert_eq!(
            params,
            vec![
                ("track_name".to_owned(), "Song".to_owned()),
                ("album_name".to_owned(), "Album".to_owned()),
            ]
        );
    }

    #[test]
    fn empty_search_builds_no_params() {
        assert!(build_params("", "Album", "Artist", "").is_empty());
    }

    #[test]
    fn trims_structured_values() {
        let params = build_params("  Song  ", "  Album  ", "  Artist  ", "");

        assert!(params.contains(&("track_name".to_owned(), "Song".to_owned())));
        assert!(params.contains(&("artist_name".to_owned(), "Artist".to_owned())));
        assert!(params.contains(&("album_name".to_owned(), "Album".to_owned())));
    }

    #[test]
    fn maps_legacy_search_item_to_summary() {
        let item: LegacySearchItem = serde_json::from_value(json!({
            "id": 7,
            "name": "Legacy Song",
            "trackName": "Legacy Song",
            "artistName": "Artist",
            "albumName": "Album",
            "duration": 180.0,
            "instrumental": false,
            "hasWordSync": true,
            "plainLyrics": "line one\nline two",
            "syncedLyrics": "[00:01.00] line one",
            "lyricsfile": "version: '1.0'"
        }))
        .unwrap();

        assert_eq!(
            item.into_summary(),
            TrackSummary {
                id: 7,
                track_name: Some("Legacy Song".to_owned()),
                artist_name: Some("Artist".to_owned()),
                album_name: Some("Album".to_owned()),
                duration: Some(180.0),
                instrumental: false,
                has_word_sync: true,
                has_plain_lyrics: true,
                has_synced_lyrics: true,
                has_lyricsfile: true,
            }
        );
    }

    #[test]
    fn legacy_mapping_uses_name_and_ignores_empty_lyrics() {
        let item: LegacySearchItem = serde_json::from_value(json!({
            "id": 8,
            "name": "Name Only",
            "artistName": "Artist",
            "albumName": "Album",
            "duration": 120.0,
            "instrumental": false,
            "plainLyrics": "   ",
            "syncedLyrics": "",
            "lyricsfile": null
        }))
        .unwrap();

        let summary = item.into_summary();

        assert_eq!(summary.track_name.as_deref(), Some("Name Only"));
        assert!(!summary.has_plain_lyrics);
        assert!(!summary.has_synced_lyrics);
        assert!(!summary.has_lyricsfile);
    }

    #[test]
    fn falls_back_only_when_v2_is_unsupported() {
        assert!(v2_unsupported(&error(
            LrclibErrorKind::NotFound,
            "NotFound"
        )));
        assert!(v2_unsupported(&error(
            LrclibErrorKind::Unexpected,
            "InvalidResponse"
        )));

        for (kind, name) in [
            (LrclibErrorKind::RateLimited, "RateLimitExceeded"),
            (LrclibErrorKind::ServerOverloaded, "ServerOverloaded"),
            (LrclibErrorKind::ServerError, "UnknownError"),
            (LrclibErrorKind::Network, "NetworkError"),
            (LrclibErrorKind::Validation, "ValidationError"),
        ] {
            assert!(!v2_unsupported(&error(kind, name)));
        }
    }

    #[tokio::test]
    async fn returns_v2_summaries_with_availability_flags() {
        let body = r#"[{"id":7,"trackName":"Song","artistName":"Artist","albumName":"Album","duration":180.0,"instrumental":false,"hasWordSync":true,"hasPlainLyrics":true,"hasSyncedLyrics":true,"hasLyricsfile":true}]"#;
        let (base_url, requests) = spawn_mock_server(vec![(200, body)]).await;

        let summaries = request("", "", "", "song", &base_url).await.unwrap();

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].track_name.as_deref(), Some("Song"));
        assert!(summaries[0].has_word_sync);
        assert!(summaries[0].has_synced_lyrics);

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 1);
        assert!(requests[0].contains(SEARCH_V2_PATH));
    }

    #[tokio::test]
    async fn falls_back_to_v1_when_v2_route_is_missing() {
        let v1_body = r#"[{"id":7,"name":"Legacy Song","trackName":"Legacy Song","artistName":"Artist","albumName":"Album","duration":180.0,"instrumental":false,"hasWordSync":false,"plainLyrics":"line one\nline two","syncedLyrics":null,"lyricsfile":null}]"#;
        let (base_url, requests) = spawn_mock_server(vec![(404, ""), (200, v1_body)]).await;

        let summaries = request("", "", "", "song", &base_url).await.unwrap();

        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].track_name.as_deref(), Some("Legacy Song"));
        assert!(summaries[0].has_plain_lyrics);
        assert!(!summaries[0].has_synced_lyrics);
        assert!(!summaries[0].has_lyricsfile);

        let requests = requests.lock().unwrap();
        assert_eq!(requests.len(), 2);
        assert!(requests[0].contains(SEARCH_V2_PATH));
        assert!(requests[1].contains(SEARCH_V1_PATH));
    }
}
