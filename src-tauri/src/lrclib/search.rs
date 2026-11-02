use crate::lrclib::client::{self, LrclibError, RetryPolicy};
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct SearchItem {
    id: i64,
    name: Option<String>,
    track_name: Option<String>,
    artist_name: Option<String>,
    album_name: Option<String>,
    duration: Option<f64>,
    instrumental: bool,
    #[serde(default)]
    has_word_sync: bool,
    plain_lyrics: Option<String>,
    synced_lyrics: Option<String>,
    lyricsfile: Option<String>,
}

#[derive(Deserialize, Serialize)]
pub struct Response(Vec<SearchItem>);

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

pub async fn request(
    title: &str,
    album_name: &str,
    artist_name: &str,
    q: &str,
    lrclib_instance: &str,
) -> Result<Response, LrclibError> {
    let params = build_params(title, album_name, artist_name, q);
    if params.is_empty() {
        return Ok(Response(Vec::new()));
    }

    let res = client::get(
        lrclib_instance,
        "/api/search",
        &params,
        RetryPolicy::Interactive,
    )
    .await?;

    client::parse_json(res).await
}

#[cfg(test)]
mod tests {
    use super::build_params;

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
}
