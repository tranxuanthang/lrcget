use crate::lrclib::client::{self, LrclibError, RetryPolicy};
use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Response {
    pub prefix: String,
    pub target: String,
}

pub async fn request(lrclib_instance: &str) -> Result<Response, LrclibError> {
    let res = client::post(
        lrclib_instance,
        "/api/request-challenge",
        RetryPolicy::Interactive,
    )
    .await?;

    client::parse_json(res).await
}
