//! The HTTP client and retry timing shared by OAuth, MCP and downloads, and [`Downloader`]
//! for public endpoints on the same TLS stack.

use std::io::Read;
use std::time::Duration;

use reqwest::blocking::{Client, RequestBuilder, Response};
use reqwest::header::{CONTENT_TYPE, HeaderMap, RETRY_AFTER};
use reqwest::redirect::Policy;
use reqwest::{StatusCode, Url};

use crate::config::{RetryPolicy, secure_or_loopback};
use crate::error::{Error, Result};

/// Redirects a [`Downloader`] follows.
const MAX_REDIRECTS: usize = 10;

/// `creator_exec` may run for 30 s; allow queueing and transfer on top.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(20);

/// A blocking client. Redirects are not followed, so a bearer token never leaves the host it
/// was sent to. Do not create or use it inside an async runtime (`spawn_blocking` instead).
pub(crate) fn client() -> Result<Client> {
    client_with_timeout(REQUEST_TIMEOUT)
}

/// [`client`] with requests that time out after `timeout`.
pub(crate) fn client_with_timeout(timeout: Duration) -> Result<Client> {
    Ok(Client::builder()
        .user_agent(concat!("gen-animated-icons/", env!("CARGO_PKG_VERSION")))
        .timeout(timeout)
        .connect_timeout(CONNECT_TIMEOUT.min(timeout))
        .redirect(Policy::none())
        .build()?)
}

/// Requests to public endpoints (the icon library, review submissions) with the engine
/// client's TLS stack and trust roots (the OS store). Sends no credentials. URLs must be https,
/// or http on a loopback host, as the URL parser reads them; redirects are followed under the
/// same rule and never from https to http.
#[derive(Debug, Clone)]
pub struct Downloader {
    client: Client,
}

impl Downloader {
    /// A downloader whose requests time out after `timeout`. Do not create or use it inside an
    /// async runtime (`spawn_blocking` instead).
    pub fn new(timeout: Duration) -> Result<Self> {
        let redirects = Policy::custom(|attempt| {
            let downgrade = attempt.url().scheme() == "http"
                && attempt.previous().iter().any(|url| url.scheme() == "https");
            if attempt.previous().len() >= MAX_REDIRECTS {
                attempt.error("too many redirects")
            } else if downgrade || !secure_or_loopback(attempt.url()) {
                attempt.error("refused a redirect to plain http")
            } else {
                attempt.follow()
            }
        });
        let client = Client::builder()
            .user_agent(concat!("gen-animated-icons/", env!("CARGO_PKG_VERSION")))
            .timeout(timeout)
            .connect_timeout(CONNECT_TIMEOUT.min(timeout))
            .redirect(redirects)
            .build()?;
        Ok(Downloader { client })
    }

    /// The body of `url`, or `None` on 404. Bodies over `limit` bytes fail with
    /// [`Error::Protocol`]; other statuses with [`Error::Http`].
    pub fn get(&self, url: &str, limit: u64) -> Result<Option<Vec<u8>>> {
        let response = self.client.get(checked(url)?).send()?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        if !response.status().is_success() {
            let status = response.status().as_u16();
            return Err(Error::Http {
                status,
                url: redacted(response.url()),
                body: String::new(),
            });
        }
        read_limited(response, limit).map(Some)
    }

    /// The HTTP status a GET of `url` answers with; the body is not read.
    pub fn status(&self, url: &str) -> Result<u16> {
        Ok(self.client.get(checked(url)?).send()?.status().as_u16())
    }

    /// POSTs `body` as `content_type` with extra `headers` and returns the response body. Bodies
    /// over `limit` bytes fail with [`Error::Protocol`]; statuses other than 2xx with
    /// [`Error::Http`], which shows the start of the body.
    pub fn post(
        &self,
        url: &str,
        content_type: &str,
        headers: &[(&str, &str)],
        body: Vec<u8>,
        limit: u64,
    ) -> Result<Vec<u8>> {
        let request = self
            .client
            .post(checked(url)?)
            .header(CONTENT_TYPE, content_type);
        let request = headers
            .iter()
            .fold(request, |request: RequestBuilder, (name, value)| {
                request.header(*name, *value)
            });
        let response = request.body(body).send()?;
        if !response.status().is_success() {
            return Err(Error::Http {
                status: response.status().as_u16(),
                url: redacted(response.url()),
                body: body_snippet(response),
            });
        }
        read_limited(response, limit)
    }
}

/// `url` parsed, when it is https or http on a loopback host. The check runs on the parsed URL,
/// so spellings such as `http://host\@localhost/` are judged by the host they reach.
fn checked(url: &str) -> Result<Url> {
    let parsed = Url::parse(url).map_err(|e| Error::Config(format!("URL {url:?}: {e}")))?;
    if !secure_or_loopback(&parsed) {
        return Err(Error::Config(format!(
            "{} must use https (http is allowed for loopback only)",
            redacted(&parsed)
        )));
    }
    Ok(parsed)
}

/// The body, failing with [`Error::Protocol`] past `limit` bytes.
fn read_limited(response: Response, limit: u64) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    response
        .take(limit.saturating_add(1))
        .read_to_end(&mut body)?;
    if body.len() as u64 > limit {
        return Err(Error::Protocol(format!(
            "the response is over {limit} bytes"
        )));
    }
    Ok(body)
}

/// `url` without its query and fragment, where signed URLs carry their credential.
pub(crate) fn redacted(url: &reqwest::Url) -> String {
    let mut url = url.clone();
    url.set_query(None);
    url.set_fragment(None);
    url.into()
}

/// Statuses worth retrying: rate limiting and server-side failures (Cloudflare's 52x included).
pub(crate) fn is_transient(status: u16) -> bool {
    status == 429 || (500..600).contains(&status)
}

/// Network failures worth retrying for idempotent requests.
pub(crate) fn is_transient_error(error: &reqwest::Error) -> bool {
    error.is_connect() || error.is_timeout() || error.is_request()
}

/// The delay before retry number `retry` (1-based): `Retry-After` when the server sent one,
/// otherwise exponential backoff with up to 25% jitter, capped at `max_delay`.
pub(crate) fn backoff(policy: &RetryPolicy, retry: u32, headers: Option<&HeaderMap>) -> Duration {
    if let Some(after) = headers.and_then(retry_after) {
        return after.min(policy.max_delay);
    }
    let exp = policy
        .base_delay
        .saturating_mul(1u32 << retry.saturating_sub(1).min(16));
    let jitter = exp.mul_f64(f64::from(getrandom::u32().unwrap_or(0) % 256) / 1024.0);
    (exp + jitter).min(policy.max_delay)
}

/// `Retry-After` in its delta-seconds form.
fn retry_after(headers: &HeaderMap) -> Option<Duration> {
    let value = headers.get(RETRY_AFTER)?.to_str().ok()?;
    value.trim().parse::<u64>().ok().map(Duration::from_secs)
}

/// The response body as text, cut to a readable length for error messages.
pub(crate) fn body_snippet(response: Response) -> String {
    let text = response.text().unwrap_or_default();
    let text = text.trim();
    match text.char_indices().nth(300) {
        Some((cut, _)) => format!("{}…", &text[..cut]),
        None => text.to_owned(),
    }
}

/// Maps a final (non-retried) error status to an [`Error`].
pub(crate) fn status_error(response: Response) -> Error {
    let status = response.status().as_u16();
    match status {
        429 => Error::RateLimited,
        500..=599 => Error::Unavailable { status },
        _ => Error::Http {
            status,
            url: redacted(response.url()),
            body: body_snippet(response),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use reqwest::header::HeaderValue;

    #[test]
    fn backoff_grows_and_honours_retry_after() {
        let policy = RetryPolicy {
            attempts: 5,
            base_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(2),
        };
        let first = backoff(&policy, 1, None);
        assert!(first >= Duration::from_millis(100) && first <= Duration::from_millis(125));
        let third = backoff(&policy, 3, None);
        assert!(third >= Duration::from_millis(400) && third <= Duration::from_millis(500));
        assert_eq!(backoff(&policy, 30, None), Duration::from_secs(2));

        let mut headers = HeaderMap::new();
        headers.insert(RETRY_AFTER, HeaderValue::from_static("1"));
        assert_eq!(backoff(&policy, 1, Some(&headers)), Duration::from_secs(1));
        headers.insert(RETRY_AFTER, HeaderValue::from_static("120"));
        assert_eq!(backoff(&policy, 1, Some(&headers)), Duration::from_secs(2));
    }

    #[test]
    fn status_errors_name_the_url_without_its_query() {
        let url = reqwest::Url::parse("https://r2.example/doc/a.json?X-Amz-Signature=s#f").unwrap();
        assert_eq!(redacted(&url), "https://r2.example/doc/a.json");
    }

    #[test]
    fn downloads_follow_redirects_but_never_to_plain_http() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = rt.block_on(MockServer::start());
        let base = server.uri();
        let mounts = [
            (
                "/file",
                ResponseTemplate::new(200).set_body_string("0123456789"),
            ),
            (
                "/moved",
                ResponseTemplate::new(302).insert_header("location", format!("{base}/file")),
            ),
            (
                "/downgrade",
                ResponseTemplate::new(302).insert_header("location", "http://example.com/file"),
            ),
            ("/denied", ResponseTemplate::new(403)),
        ];
        for (route, response) in mounts {
            rt.block_on(
                Mock::given(method("GET"))
                    .and(path(route))
                    .respond_with(response)
                    .mount(&server),
            );
        }
        let downloader = Downloader::new(Duration::from_secs(10)).unwrap();
        let get = |route: &str, limit| downloader.get(&format!("{base}{route}?sig=secret"), limit);
        assert_eq!(get("/file", 10).unwrap().unwrap(), b"0123456789");
        assert_eq!(get("/moved", 10).unwrap().unwrap(), b"0123456789");
        assert_eq!(get("/missing", 10).unwrap(), None);
        assert!(matches!(get("/file", 9), Err(Error::Protocol(_))));
        let err = get("/downgrade", 10).unwrap_err();
        assert!(
            matches!(err, Error::Network(_)) && format!("{err:?}").contains("plain http"),
            "{err:?}"
        );
        let err = get("/denied", 10).unwrap_err();
        assert!(matches!(err, Error::Http { status: 403, .. }), "{err}");
        assert!(!err.to_string().contains("secret"), "{err}");
    }

    #[test]
    fn posts_and_status_probes_use_the_same_stack() {
        use wiremock::matchers::{body_bytes, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};
        let rt = tokio::runtime::Runtime::new().unwrap();
        let server = rt.block_on(MockServer::start());
        let base = server.uri();
        rt.block_on(
            Mock::given(method("POST"))
                .and(path("/submissions"))
                .and(header("content-type", "application/zip"))
                .and(header("x-bundle-sha256", "abc"))
                .and(body_bytes(b"PK".to_vec()))
                .respond_with(
                    ResponseTemplate::new(201).set_body_string(r#"{"status":"received"}"#),
                )
                .mount(&server),
        );
        rt.block_on(
            Mock::given(method("POST"))
                .and(path("/full"))
                .respond_with(ResponseTemplate::new(413).set_body_string("bundle too large"))
                .mount(&server),
        );
        rt.block_on(
            Mock::given(method("GET"))
                .and(path("/mcp"))
                .respond_with(ResponseTemplate::new(401))
                .mount(&server),
        );
        let downloader = Downloader::new(Duration::from_secs(10)).unwrap();
        let post = |route: &str, limit| {
            downloader.post(
                &format!("{base}{route}"),
                "application/zip",
                &[("X-Bundle-Sha256", "abc")],
                b"PK".to_vec(),
                limit,
            )
        };
        assert_eq!(
            post("/submissions", 100).unwrap(),
            br#"{"status":"received"}"#
        );
        assert!(matches!(post("/submissions", 4), Err(Error::Protocol(_))));
        let err = post("/full", 100).unwrap_err();
        assert!(
            matches!(&err, Error::Http { status: 413, body, .. } if body == "bundle too large"),
            "{err}"
        );
        assert_eq!(downloader.status(&format!("{base}/mcp")).unwrap(), 401);
        assert_eq!(downloader.status(&format!("{base}/missing")).unwrap(), 404);
    }

    #[test]
    fn requests_never_start_on_plain_http_to_another_host() {
        let downloader = Downloader::new(Duration::from_secs(2)).unwrap();
        // The parser reads `\` as `/`, so this goes to 192.0.2.1, not localhost.
        let sneaky = "http://192.0.2.1:9\\@localhost/lib";
        for err in [
            downloader.get(sneaky, 10).unwrap_err(),
            downloader.status(sneaky).unwrap_err(),
            downloader
                .post(sneaky, "application/zip", &[], Vec::new(), 10)
                .unwrap_err(),
            downloader
                .get("http://example.com/x?sig=secret", 10)
                .unwrap_err(),
        ] {
            assert!(matches!(err, Error::Config(_)), "{err}");
            assert!(!err.to_string().contains("secret"), "{err}");
        }
        assert!(matches!(
            downloader.get("not a url", 10),
            Err(Error::Config(_))
        ));
    }

    #[test]
    fn transient_statuses() {
        assert!(is_transient(429));
        assert!(is_transient(503));
        assert!(is_transient(522));
        assert!(!is_transient(401));
        assert!(!is_transient(404));
    }
}
