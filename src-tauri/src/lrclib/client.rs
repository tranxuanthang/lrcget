use std::fmt;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};

use reqwest::header::{HeaderMap, ACCEPT, RETRY_AFTER};
use reqwest::{Method, Response, StatusCode, Url};
use serde::de::DeserializeOwned;
use serde::Deserialize;

/// Public identifier of the registered LRCGET client application.
///
/// According to the LRCLIB API documentation, `Lrclib-App-Id` is not a secret
/// and is safe to ship in client-side applications.
pub const LRCLIB_APP_ID: &str = "XrVKboR0WWkjUKWZFHxduclBxCnu4qAXHjLwilNJzP8";

const LRCLIB_APP_ID_HEADER: &str = "Lrclib-App-Id";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const BODY_PREVIEW_LIMIT: usize = 300;
const MAX_BACKOFF: Duration = Duration::from_secs(8);
const INTERACTIVE_MAX_RETRY_DELAY: Duration = Duration::from_secs(3);

static HTTP_CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static COOLDOWN: Mutex<Option<Cooldown>> = Mutex::new(None);

#[derive(Debug)]
struct Cooldown {
    until: Instant,
    kind: LrclibErrorKind,
    status: Option<u16>,
    name: String,
    message: String,
    scope: Option<String>,
}

fn user_agent() -> String {
    format!(
        "LRCGET v{} (https://github.com/tranxuanthang/lrcget)",
        env!("CARGO_PKG_VERSION")
    )
}

fn http_client() -> &'static reqwest::Client {
    HTTP_CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .timeout(REQUEST_TIMEOUT)
            .user_agent(user_agent())
            .build()
            .expect("Failed to build LRCLIB HTTP client")
    })
}

fn is_official_instance(lrclib_instance: &str) -> bool {
    let Ok(url) = Url::parse(lrclib_instance) else {
        return false;
    };

    let Some(host) = url.host_str() else {
        return false;
    };

    let host = host.trim_end_matches('.');
    host == "lrclib.net" || host.ends_with(".lrclib.net")
}

fn app_id_for_instance(lrclib_instance: &str) -> Option<&'static str> {
    if is_official_instance(lrclib_instance) {
        Some(LRCLIB_APP_ID)
    } else {
        None
    }
}

fn endpoint(lrclib_instance: &str, path: &str) -> String {
    format!("{}{}", lrclib_instance.trim_end_matches('/'), path)
}

fn request_builder(method: Method, url: Url, lrclib_instance: &str) -> reqwest::RequestBuilder {
    let mut builder = http_client()
        .request(method, url)
        .header(ACCEPT, "application/json");

    if let Some(app_id) = app_id_for_instance(lrclib_instance) {
        builder = builder.header(LRCLIB_APP_ID_HEADER, app_id);
    }

    builder
}

/// Classification of LRCLIB API and transport errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LrclibErrorKind {
    NotFound,
    Validation,
    Unauthorized,
    RateLimited,
    ServerOverloaded,
    ServerError,
    Network,
    Unexpected,
}

/// Controls how aggressively a request is retried.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryPolicy {
    /// Background work such as mass downloads: retry transient failures and
    /// wait for the server-provided `Retry-After` before retrying.
    Batch,
    /// User-triggered requests: retry quick server/network failures, but do
    /// not block the UI for a long rate-limit cooldown.
    Interactive,
    /// One-shot requests such as publishing or flagging, where retrying with
    /// the same one-time publish token is not safe.
    NoRetry,
}

impl RetryPolicy {
    fn max_attempts(self) -> u32 {
        match self {
            RetryPolicy::Batch => 4,
            RetryPolicy::Interactive => 3,
            RetryPolicy::NoRetry => 1,
        }
    }
}

/// Error returned by the LRCLIB client.
///
/// The API error body uses `{ message, name, statusCode }`, and rate-limit
/// responses also carry `Retry-After` and a limiting `scope`.
#[derive(Debug, Clone)]
pub struct LrclibError {
    pub kind: LrclibErrorKind,
    pub status: Option<u16>,
    pub name: String,
    pub message: String,
    pub retry_after: Option<Duration>,
    pub scope: Option<String>,
}

impl LrclibError {
    pub fn not_found() -> Self {
        Self {
            kind: LrclibErrorKind::NotFound,
            status: Some(StatusCode::NOT_FOUND.as_u16()),
            name: "NotFound".to_owned(),
            message: "There is no lyrics for this track".to_owned(),
            retry_after: None,
            scope: None,
        }
    }

    /// Only retry writes when the server explicitly rejected the request with
    /// 429 or 503. Retrying a timeout or a server error could duplicate a
    /// publish/flag operation.
    pub fn is_retryable_write(&self) -> bool {
        matches!(
            self.kind,
            LrclibErrorKind::RateLimited | LrclibErrorKind::ServerOverloaded
        )
    }

    /// Build a client-side validation error for a malformed request.
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            kind: LrclibErrorKind::Validation,
            status: Some(StatusCode::BAD_REQUEST.as_u16()),
            name: "ValidationError".to_owned(),
            message: message.into(),
            retry_after: None,
            scope: None,
        }
    }

    fn invalid_url(message: String) -> Self {
        Self {
            kind: LrclibErrorKind::Unexpected,
            status: None,
            name: "InvalidUrl".to_owned(),
            message,
            retry_after: None,
            scope: None,
        }
    }

    fn should_retry(&self, policy: RetryPolicy) -> bool {
        match policy {
            RetryPolicy::NoRetry => false,
            RetryPolicy::Batch => matches!(
                self.kind,
                LrclibErrorKind::RateLimited
                    | LrclibErrorKind::ServerOverloaded
                    | LrclibErrorKind::ServerError
                    | LrclibErrorKind::Network
            ),
            RetryPolicy::Interactive => match self.kind {
                LrclibErrorKind::ServerOverloaded => self
                    .retry_after
                    .map_or(true, |delay| delay <= INTERACTIVE_MAX_RETRY_DELAY),
                LrclibErrorKind::ServerError | LrclibErrorKind::Network => true,
                _ => false,
            },
        }
    }

    fn from_status(status: StatusCode, retry_after: Option<Duration>, body: &str) -> Self {
        let kind = error_kind(status);
        let parsed = serde_json::from_str::<ApiErrorBody>(body).ok();
        let is_json = parsed.is_some();
        let (api_name, api_message, scope) = match parsed {
            Some(error) => (error.name, error.message, error.scope),
            None => (String::new(), String::new(), None),
        };

        let name = if kind == LrclibErrorKind::NotFound || api_name.trim().is_empty() {
            default_name(kind).to_owned()
        } else {
            api_name.trim().to_owned()
        };

        let message = if kind == LrclibErrorKind::NotFound {
            default_message(kind).to_owned()
        } else if !api_message.trim().is_empty() {
            api_message.trim().to_owned()
        } else if is_json {
            default_message(kind).to_owned()
        } else {
            let preview = body_preview(body);
            if preview.is_empty() {
                default_message(kind).to_owned()
            } else {
                preview
            }
        };

        Self {
            kind,
            status: Some(status.as_u16()),
            name,
            message,
            retry_after,
            scope,
        }
    }
}

impl fmt::Display for LrclibError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.name.is_empty() {
            write!(f, "{}: ", self.name)?;
        }

        if self.message.is_empty() {
            if self.name.is_empty() {
                write!(f, "Unknown LRCLIB error")?;
            }
        } else {
            write!(f, "{}", self.message)?;
        }

        if let Some(retry_after) = self.retry_after {
            if matches!(
                self.kind,
                LrclibErrorKind::RateLimited | LrclibErrorKind::ServerOverloaded
            ) {
                write!(f, " (retry after {} seconds)", retry_after.as_secs())?;
            }
        }

        Ok(())
    }
}

impl std::error::Error for LrclibError {}

impl From<reqwest::Error> for LrclibError {
    fn from(error: reqwest::Error) -> Self {
        let message = if error.is_timeout() {
            "Request to LRCLIB timed out".to_owned()
        } else if error.is_connect() {
            format!("Could not connect to LRCLIB: {}", error)
        } else {
            error.to_string()
        };

        Self {
            kind: LrclibErrorKind::Network,
            status: None,
            name: "NetworkError".to_owned(),
            message,
            retry_after: None,
            scope: None,
        }
    }
}

#[derive(Deserialize)]
struct ApiErrorBody {
    #[serde(default)]
    message: String,
    #[serde(default, alias = "error")]
    name: String,
    #[serde(default)]
    scope: Option<String>,
}

fn error_kind(status: StatusCode) -> LrclibErrorKind {
    match status {
        StatusCode::NOT_FOUND => LrclibErrorKind::NotFound,
        StatusCode::BAD_REQUEST => LrclibErrorKind::Validation,
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => LrclibErrorKind::Unauthorized,
        StatusCode::TOO_MANY_REQUESTS => LrclibErrorKind::RateLimited,
        StatusCode::SERVICE_UNAVAILABLE => LrclibErrorKind::ServerOverloaded,
        StatusCode::INTERNAL_SERVER_ERROR
        | StatusCode::BAD_GATEWAY
        | StatusCode::GATEWAY_TIMEOUT => LrclibErrorKind::ServerError,
        _ => LrclibErrorKind::Unexpected,
    }
}

fn default_name(kind: LrclibErrorKind) -> &'static str {
    match kind {
        LrclibErrorKind::NotFound => "NotFound",
        LrclibErrorKind::Validation => "ValidationError",
        LrclibErrorKind::Unauthorized => "Unauthorized",
        LrclibErrorKind::RateLimited => "RateLimitExceeded",
        LrclibErrorKind::ServerOverloaded => "ServerOverloaded",
        LrclibErrorKind::ServerError => "UnknownError",
        LrclibErrorKind::Network => "NetworkError",
        LrclibErrorKind::Unexpected => "UnexpectedError",
    }
}

fn default_message(kind: LrclibErrorKind) -> &'static str {
    match kind {
        LrclibErrorKind::NotFound => "There is no lyrics for this track",
        LrclibErrorKind::Validation => "LRCLIB rejected the request parameters",
        LrclibErrorKind::Unauthorized => "LRCLIB rejected the application ID or credentials",
        LrclibErrorKind::RateLimited => "Too many requests. Please wait before retrying.",
        LrclibErrorKind::ServerOverloaded => "The server is busy, please retry in a moment",
        LrclibErrorKind::ServerError => "Unexpected server error, try again later",
        LrclibErrorKind::Network => "Failed to reach LRCLIB",
        LrclibErrorKind::Unexpected => "Unexpected response from LRCLIB",
    }
}

fn body_preview(body: &str) -> String {
    let trimmed = body.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    let mut preview: String = trimmed.chars().take(BODY_PREVIEW_LIMIT).collect();
    if trimmed.chars().count() > BODY_PREVIEW_LIMIT {
        preview.push('…');
    }
    preview
}

/// LRCLIB documents `Retry-After` as an integer number of seconds.
fn parse_retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?.trim();
    value.parse::<u64>().ok().map(Duration::from_secs)
}

fn set_cooldown(error: &LrclibError) {
    let Some(delay) = error.retry_after else {
        return;
    };
    if delay.is_zero() {
        return;
    }
    let Some(until) = Instant::now().checked_add(delay) else {
        return;
    };

    let mut guard = COOLDOWN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    if guard
        .as_ref()
        .map_or(true, |cooldown| cooldown.until < until)
    {
        *guard = Some(Cooldown {
            until,
            kind: error.kind,
            status: error.status,
            name: error.name.clone(),
            message: error.message.clone(),
            scope: error.scope.clone(),
        });
    }
}

fn cooldown_error() -> Option<LrclibError> {
    let guard = COOLDOWN
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let cooldown = guard.as_ref()?;
    let remaining = cooldown.until.checked_duration_since(Instant::now())?;

    Some(LrclibError {
        kind: cooldown.kind,
        status: cooldown.status,
        name: cooldown.name.clone(),
        message: cooldown.message.clone(),
        retry_after: Some(remaining),
        scope: cooldown.scope.clone(),
    })
}

fn register_cooldown(error: &LrclibError) {
    let should_cooldown = match error.kind {
        LrclibErrorKind::RateLimited => true,
        LrclibErrorKind::ServerOverloaded => error.retry_after.is_some(),
        _ => false,
    };

    if !should_cooldown {
        return;
    }

    let mut error = error.clone();
    if error.retry_after.is_none() {
        error.retry_after = Some(Duration::from_secs(1));
    }

    set_cooldown(&error);
}

async fn wait_for_cooldown(policy: RetryPolicy) -> Result<(), LrclibError> {
    loop {
        let Some(error) = cooldown_error() else {
            return Ok(());
        };

        match policy {
            RetryPolicy::Batch => {
                if let Some(remaining) = error.retry_after {
                    tokio::time::sleep(remaining).await;
                }
            }
            RetryPolicy::Interactive | RetryPolicy::NoRetry => return Err(error),
        }
    }
}

fn backoff_delay(failed_attempt: u32) -> Duration {
    let exponent = failed_attempt.saturating_sub(1).min(4);
    Duration::from_millis(500 * (1u64 << exponent)).min(MAX_BACKOFF)
}

fn add_jitter(delay: Duration) -> Duration {
    let max_jitter = delay.mul_f64(0.3);
    let jitter = max_jitter.mul_f64(fastrand::f64());
    delay.checked_add(jitter).unwrap_or(delay)
}

fn retry_delay(error: &LrclibError, failed_attempt: u32) -> Duration {
    let base = error
        .retry_after
        .unwrap_or_else(|| backoff_delay(failed_attempt));

    add_jitter(base)
}

/// Wait before retrying a write request. Returns `false` when the requested
/// delay is longer than `max_wait`, so the caller can fail fast instead of
/// leaving an interactive modal blocked.
pub async fn wait_for_write_retry(error: &LrclibError, max_wait: Duration) -> bool {
    let delay = error.retry_after.unwrap_or(Duration::from_secs(1));
    if delay > max_wait {
        return false;
    }

    tokio::time::sleep(add_jitter(delay)).await;
    true
}

/// Send a request built by `build`, retrying transient failures according to
/// `policy`. Non-2xx responses are converted to `LrclibError`.
async fn send_with_retry<F>(policy: RetryPolicy, build: F) -> Result<Response, LrclibError>
where
    F: Fn() -> reqwest::RequestBuilder,
{
    let max_attempts = policy.max_attempts();
    let mut attempt = 0;

    loop {
        attempt += 1;
        wait_for_cooldown(policy).await?;

        let error = match build().send().await {
            Ok(response) if response.status().is_success() => return Ok(response),
            Ok(response) => parse_error_response(response).await,
            Err(error) => error.into(),
        };

        register_cooldown(&error);

        if attempt >= max_attempts || !error.should_retry(policy) {
            return Err(error);
        }

        let delay = retry_delay(&error, attempt);

        #[cfg(debug_assertions)]
        eprintln!(
            "[LRCLIB] attempt {}/{} failed ({}); retrying in {:.1}s",
            attempt,
            max_attempts,
            error,
            delay.as_secs_f64()
        );

        tokio::time::sleep(delay).await;
    }
}

fn parse_url(url: &str) -> Result<Url, LrclibError> {
    Url::parse(url)
        .map_err(|error| LrclibError::invalid_url(format!("Invalid endpoint: {}", error)))
}

fn parse_url_with_params(url: &str, params: &[(String, String)]) -> Result<Url, LrclibError> {
    Url::parse_with_params(url, params)
        .map_err(|error| LrclibError::invalid_url(format!("Invalid endpoint: {}", error)))
}

/// Convert a non-success response into a typed LRCLIB error.
async fn parse_error_response(response: Response) -> LrclibError {
    let status = response.status();
    let retry_after = parse_retry_after(response.headers());
    let body = response.text().await.unwrap_or_default();
    LrclibError::from_status(status, retry_after, &body)
}

/// Parse a JSON response body with a descriptive error on failure.
pub async fn parse_json<T: DeserializeOwned>(response: Response) -> Result<T, LrclibError> {
    let status = response.status();
    let body = response.text().await?;

    serde_json::from_str(&body).map_err(|error| LrclibError {
        kind: LrclibErrorKind::Unexpected,
        status: Some(status.as_u16()),
        name: "InvalidResponse".to_owned(),
        message: format!("Failed to parse LRCLIB response: {}", error),
        retry_after: None,
        scope: None,
    })
}

/// Send a GET request with the given query parameters.
pub async fn get(
    lrclib_instance: &str,
    path: &str,
    params: &[(String, String)],
    policy: RetryPolicy,
) -> Result<Response, LrclibError> {
    let url = parse_url_with_params(&endpoint(lrclib_instance, path), params)?;

    send_with_retry(policy, || {
        request_builder(Method::GET, url.clone(), lrclib_instance)
    })
    .await
}

/// Send a POST request with a JSON body and optional extra headers.
pub async fn post_json<T: serde::Serialize + ?Sized>(
    lrclib_instance: &str,
    path: &str,
    body: &T,
    headers: &[(&str, &str)],
    policy: RetryPolicy,
) -> Result<Response, LrclibError> {
    let url = parse_url(&endpoint(lrclib_instance, path))?;

    send_with_retry(policy, || {
        let mut builder = request_builder(Method::POST, url.clone(), lrclib_instance).json(body);

        for (name, value) in headers {
            builder = builder.header(*name, *value);
        }

        builder
    })
    .await
}

/// Send a POST request without a body.
pub async fn post(
    lrclib_instance: &str,
    path: &str,
    policy: RetryPolicy,
) -> Result<Response, LrclibError> {
    let url = parse_url(&endpoint(lrclib_instance, path))?;

    send_with_retry(policy, || {
        request_builder(Method::POST, url.clone(), lrclib_instance)
    })
    .await
}

#[cfg(test)]
mod tests {
    use super::{
        add_jitter, app_id_for_instance, backoff_delay, body_preview, endpoint,
        is_official_instance, parse_retry_after, retry_delay, wait_for_write_retry, LrclibError,
        LrclibErrorKind, RetryPolicy, LRCLIB_APP_ID, MAX_BACKOFF,
    };
    use reqwest::header::{HeaderMap, HeaderValue, RETRY_AFTER};
    use reqwest::StatusCode;
    use std::sync::{Arc, Mutex as StdMutex};
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn spawn_mock_server(
        responses: Vec<(u16, &'static str, &'static str)>,
    ) -> (String, Arc<StdMutex<Vec<String>>>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address = listener.local_addr().unwrap();
        let requests = Arc::new(StdMutex::new(Vec::new()));
        let captured_requests = Arc::clone(&requests);

        tokio::spawn(async move {
            for (status, extra_headers, body) in responses {
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
                    400 => "Bad Request",
                    404 => "Not Found",
                    429 => "Too Many Requests",
                    500 => "Internal Server Error",
                    503 => "Service Unavailable",
                    _ => "Unknown",
                };

                let response = format!(
                    "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n{}\r\n{}",
                    status,
                    reason,
                    body.len(),
                    extra_headers,
                    body
                );

                let _ = stream.write_all(response.as_bytes()).await;
                let _ = stream.shutdown().await;
            }
        });

        (format!("http://{}", address), requests)
    }

    async fn get_with_timeout(
        base_url: &str,
        policy: RetryPolicy,
    ) -> Result<reqwest::Response, LrclibError> {
        tokio::time::timeout(
            Duration::from_secs(5),
            super::get(base_url, "/api/get", &[], policy),
        )
        .await
        .expect("mock LRCLIB request timed out")
    }

    #[test]
    fn detects_official_instance() {
        assert!(is_official_instance("https://lrclib.net"));
        assert!(is_official_instance("https://lrclib.net/"));
        assert!(is_official_instance("https://api.lrclib.net"));
        assert!(!is_official_instance("https://lrclib.net.example.com"));
        assert!(!is_official_instance("https://example.com"));
        assert!(!is_official_instance("not a url"));
    }

    #[test]
    fn builds_endpoint_without_duplicate_slash() {
        assert_eq!(
            endpoint("https://lrclib.net/", "/api/get"),
            "https://lrclib.net/api/get"
        );
        assert_eq!(
            endpoint("https://lrclib.net", "/api/get"),
            "https://lrclib.net/api/get"
        );
    }

    #[test]
    fn parses_integer_retry_after() {
        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("7"));
        assert_eq!(parse_retry_after(&headers), Some(Duration::from_secs(7)));

        headers.insert(RETRY_AFTER, HeaderValue::from_static("not-a-number"));
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parses_new_error_schema() {
        let error = LrclibError::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(12)),
            r#"{"message":"Too many requests from this IP address","name":"RateLimitExceeded","statusCode":429,"scope":"ip"}"#,
        );

        assert_eq!(error.kind, LrclibErrorKind::RateLimited);
        assert_eq!(error.status, Some(429));
        assert_eq!(error.name, "RateLimitExceeded");
        assert_eq!(error.message, "Too many requests from this IP address");
        assert_eq!(error.retry_after, Some(Duration::from_secs(12)));
        assert_eq!(error.scope.as_deref(), Some("ip"));
    }

    #[test]
    fn accepts_legacy_error_field() {
        let error = LrclibError::from_status(
            StatusCode::BAD_REQUEST,
            None,
            r#"{"message":"duration: must be between 1 and 3600","error":"ValidationError"}"#,
        );

        assert_eq!(error.kind, LrclibErrorKind::Validation);
        assert_eq!(error.name, "ValidationError");
        assert_eq!(error.message, "duration: must be between 1 and 3600");
    }

    #[test]
    fn falls_back_for_non_json_errors() {
        let error = LrclibError::from_status(
            StatusCode::SERVICE_UNAVAILABLE,
            Some(Duration::from_secs(1)),
            "upstream connect error",
        );

        assert_eq!(error.kind, LrclibErrorKind::ServerOverloaded);
        assert_eq!(error.name, "ServerOverloaded");
        assert_eq!(error.message, "upstream connect error");
        assert_eq!(error.retry_after, Some(Duration::from_secs(1)));
    }

    #[test]
    fn maps_unauthorized_and_server_errors() {
        let unauthorized = LrclibError::from_status(
            StatusCode::UNAUTHORIZED,
            None,
            r#"{"message":"The provided Lrclib-App-Id is not a registered application","name":"InvalidAppIdError","statusCode":401}"#,
        );
        assert_eq!(unauthorized.kind, LrclibErrorKind::Unauthorized);
        assert_eq!(unauthorized.name, "InvalidAppIdError");

        let server = LrclibError::from_status(StatusCode::INTERNAL_SERVER_ERROR, None, "");
        assert_eq!(server.kind, LrclibErrorKind::ServerError);
        assert_eq!(server.name, "UnknownError");
    }

    #[test]
    fn displays_retry_after_for_retryable_errors() {
        let error = LrclibError::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(5)),
            r#"{"message":"Too many requests","name":"RateLimitExceeded"}"#,
        );

        assert_eq!(
            error.to_string(),
            "RateLimitExceeded: Too many requests (retry after 5 seconds)"
        );
    }

    #[test]
    fn computes_exponential_backoff() {
        assert_eq!(backoff_delay(1), Duration::from_millis(500));
        assert_eq!(backoff_delay(2), Duration::from_millis(1000));
        assert_eq!(backoff_delay(3), Duration::from_millis(2000));
        assert_eq!(backoff_delay(4), Duration::from_millis(4000));
        assert_eq!(backoff_delay(5), MAX_BACKOFF);
        assert_eq!(backoff_delay(20), MAX_BACKOFF);
    }

    #[test]
    fn jitter_is_bounded_to_thirty_percent() {
        let base = Duration::from_secs(10);
        let max = base + Duration::from_secs(3);

        for _ in 0..100 {
            let delay = add_jitter(base);
            assert!(delay >= base);
            assert!(delay <= max);
        }
    }

    #[test]
    fn retry_delay_honors_retry_after() {
        let error = LrclibError::from_status(
            StatusCode::SERVICE_UNAVAILABLE,
            Some(Duration::from_secs(2)),
            r#"{"message":"busy","name":"ServerOverloaded"}"#,
        );

        for _ in 0..100 {
            let delay = retry_delay(&error, 1);
            assert!(delay >= Duration::from_secs(2));
            assert!(delay <= Duration::from_millis(2700));
        }
    }

    #[test]
    fn retry_policy_controls_retryable_errors() {
        let rate_limited = LrclibError::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(1)),
            r#"{"message":"slow down","name":"RateLimitExceeded"}"#,
        );
        assert!(rate_limited.should_retry(RetryPolicy::Batch));
        assert!(!rate_limited.should_retry(RetryPolicy::Interactive));
        assert!(!rate_limited.should_retry(RetryPolicy::NoRetry));
        assert!(rate_limited.is_retryable_write());

        let short_overload = LrclibError::from_status(
            StatusCode::SERVICE_UNAVAILABLE,
            Some(Duration::from_secs(1)),
            "",
        );
        assert!(short_overload.should_retry(RetryPolicy::Interactive));
        assert!(short_overload.is_retryable_write());

        let long_overload = LrclibError::from_status(
            StatusCode::SERVICE_UNAVAILABLE,
            Some(Duration::from_secs(30)),
            "",
        );
        assert!(!long_overload.should_retry(RetryPolicy::Interactive));
        assert!(long_overload.should_retry(RetryPolicy::Batch));

        let server_error = LrclibError::from_status(StatusCode::INTERNAL_SERVER_ERROR, None, "");
        assert!(server_error.should_retry(RetryPolicy::Batch));
        assert!(server_error.should_retry(RetryPolicy::Interactive));
        assert!(!server_error.is_retryable_write());

        let not_found = LrclibError::not_found();
        assert!(!not_found.should_retry(RetryPolicy::Batch));
        assert!(!not_found.is_retryable_write());
    }

    #[tokio::test]
    async fn write_retry_honors_max_wait() {
        let long_wait = LrclibError::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(30)),
            r#"{"message":"slow down","name":"RateLimitExceeded"}"#,
        );
        assert!(!wait_for_write_retry(&long_wait, Duration::from_secs(5)).await);

        let no_wait = LrclibError::from_status(
            StatusCode::TOO_MANY_REQUESTS,
            Some(Duration::from_secs(0)),
            r#"{"message":"slow down","name":"RateLimitExceeded"}"#,
        );
        assert!(wait_for_write_retry(&no_wait, Duration::from_secs(5)).await);
    }

    #[test]
    fn adds_app_id_only_for_official_instance() {
        assert_eq!(
            app_id_for_instance("https://lrclib.net"),
            Some(LRCLIB_APP_ID)
        );
        assert_eq!(app_id_for_instance("http://127.0.0.1:1234"), None);
    }

    #[tokio::test]
    async fn retries_503_then_succeeds() {
        let (base_url, requests) = spawn_mock_server(vec![
            (
                503,
                "Retry-After: 0\r\n",
                r#"{"message":"busy","name":"ServerOverloaded","statusCode":503}"#,
            ),
            (200, "", r#"{"ok":true}"#),
        ])
        .await;

        let response = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect("request should succeed after retry");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(requests.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn retries_429_then_succeeds() {
        let (base_url, requests) = spawn_mock_server(vec![
            (
                429,
                "Retry-After: 0\r\n",
                r#"{"message":"slow down","name":"RateLimitExceeded","statusCode":429,"scope":"ip"}"#,
            ),
            (200, "", r#"{"ok":true}"#),
        ])
        .await;

        let response = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect("request should succeed after retry");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(requests.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn retries_500_then_succeeds() {
        let (base_url, requests) = spawn_mock_server(vec![
            (
                500,
                "Retry-After: 0\r\n",
                r#"{"message":"oops","name":"UnknownError","statusCode":500}"#,
            ),
            (200, "", r#"{"ok":true}"#),
        ])
        .await;

        let response = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect("request should succeed after retry");

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(requests.lock().unwrap().len(), 2);
    }

    #[tokio::test]
    async fn gives_up_after_batch_attempts() {
        let busy = r#"{"message":"busy","name":"ServerOverloaded","statusCode":503}"#;
        let (base_url, requests) = spawn_mock_server(vec![
            (503, "Retry-After: 0\r\n", busy),
            (503, "Retry-After: 0\r\n", busy),
            (503, "Retry-After: 0\r\n", busy),
            (503, "Retry-After: 0\r\n", busy),
        ])
        .await;

        let error = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect_err("request should fail after all retries");

        assert_eq!(error.kind, LrclibErrorKind::ServerOverloaded);
        assert_eq!(requests.lock().unwrap().len(), 4);
    }

    #[tokio::test]
    async fn no_retry_policy_does_not_retry() {
        let (base_url, requests) = spawn_mock_server(vec![(
            503,
            "Retry-After: 0\r\n",
            r#"{"message":"busy","name":"ServerOverloaded","statusCode":503}"#,
        )])
        .await;

        let error = get_with_timeout(&base_url, RetryPolicy::NoRetry)
            .await
            .expect_err("no-retry requests should fail after one attempt");

        assert_eq!(error.kind, LrclibErrorKind::ServerOverloaded);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn does_not_retry_not_found() {
        let (base_url, requests) = spawn_mock_server(vec![(
            404,
            "",
            r#"{"message":"not found","name":"TrackNotFound","statusCode":404}"#,
        )])
        .await;

        let error = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect_err("404 should not be retried");

        assert_eq!(error.kind, LrclibErrorKind::NotFound);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn does_not_retry_validation_errors() {
        let (base_url, requests) = spawn_mock_server(vec![(
            400,
            "",
            r#"{"message":"invalid duration","name":"ValidationError","statusCode":400}"#,
        )])
        .await;

        let error = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect_err("400 should not be retried");

        assert_eq!(error.kind, LrclibErrorKind::Validation);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn interactive_does_not_retry_rate_limits() {
        let (base_url, requests) = spawn_mock_server(vec![(
            429,
            "Retry-After: 0\r\n",
            r#"{"message":"slow down","name":"RateLimitExceeded","statusCode":429,"scope":"ip"}"#,
        )])
        .await;

        let error = get_with_timeout(&base_url, RetryPolicy::Interactive)
            .await
            .expect_err("interactive requests should fail fast on 429");

        assert_eq!(error.kind, LrclibErrorKind::RateLimited);
        assert_eq!(requests.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn sends_accept_and_omits_app_id_for_custom_instance() {
        let (base_url, requests) = spawn_mock_server(vec![(200, "", r#"{"ok":true}"#)]).await;

        let response = get_with_timeout(&base_url, RetryPolicy::Batch)
            .await
            .expect("request should succeed");

        assert_eq!(response.status(), StatusCode::OK);

        let request = requests.lock().unwrap()[0].to_lowercase();
        assert!(request.contains("accept: application/json"));
        assert!(!request.contains("lrclib-app-id"));
    }

    #[test]
    fn truncates_long_body_previews() {
        let body = "x".repeat(400);
        let preview = body_preview(&body);
        assert_eq!(preview.chars().count(), 301);
        assert!(preview.ends_with('…'));
    }
}
