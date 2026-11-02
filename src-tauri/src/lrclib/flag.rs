use crate::lrclib::client::{self, LrclibError, RetryPolicy};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Request {
    track_id: i64,
    #[serde(skip_serializing_if = "String::is_empty")]
    content: String,
}

pub async fn request(
    track_id: i64,
    content: &str,
    publish_token: &str,
    lrclib_instance: &str,
) -> Result<(), LrclibError> {
    let data = Request {
        track_id,
        content: content.to_owned(),
    };

    client::post_json(
        lrclib_instance,
        "/api/flag",
        &data,
        &[("X-Publish-Token", publish_token)],
        RetryPolicy::NoRetry,
    )
    .await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::Request;

    #[test]
    fn serializes_content_instead_of_reason() {
        let request = Request {
            track_id: 42,
            content: "wrong lyrics".to_owned(),
        };
        let json = serde_json::to_value(request).unwrap();

        assert_eq!(json["trackId"], 42);
        assert_eq!(json["content"], "wrong lyrics");
        assert!(json.get("reason").is_none());
    }

    #[test]
    fn omits_empty_content() {
        let request = Request {
            track_id: 42,
            content: String::new(),
        };
        let json = serde_json::to_value(request).unwrap();

        assert!(json.get("content").is_none());
    }
}
