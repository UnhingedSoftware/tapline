//! The HTTP side of the seam: `GET`, optionally ranged, nothing else.

use std::fmt;
use std::future::Future;

/// A `GET` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The absolute URL.
    pub url: String,
    /// Extra request headers, as name/value pairs.
    pub headers: Vec<(String, String)>,
    /// An optional byte range, inclusive of both ends, for resuming a chunk.
    pub range: Option<(u64, u64)>,
}

impl Request {
    /// A plain `GET` with no headers and no range.
    #[must_use]
    pub fn get(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            headers: Vec::new(),
            range: None,
        }
    }

    /// Adds a request header.
    #[must_use]
    pub fn header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }

    /// Restricts the request to a byte range, inclusive.
    #[must_use]
    pub const fn range(mut self, start: u64, end_inclusive: u64) -> Self {
        self.range = Some((start, end_inclusive));
        self
    }
}

/// A response, body in memory; everything tapline fetches is bounded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Response {
    /// The HTTP status code.
    pub status: u16,
    /// Response headers, as received.
    pub headers: Vec<(String, String)>,
    /// The body.
    pub body: Vec<u8>,
}

impl Response {
    /// Whether the status is 2xx.
    #[must_use]
    pub const fn is_success(&self) -> bool {
        self.status >= 200 && self.status < 300
    }

    /// The first value of a header, matched case-insensitively.
    #[must_use]
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(name))
            .map(|(_, v)| v.as_str())
    }
}

/// What went wrong fetching.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FetchError {
    /// The connection failed, timed out, or was reset.
    Transport(String),
    /// The URL could not be parsed, or used a scheme we do not speak.
    InvalidUrl(String),
    /// The response was not valid HTTP.
    MalformedResponse(String),
    /// The body was larger than the caller said it would accept.
    BodyTooLarge {
        /// The cap that was exceeded.
        limit: u64,
    },
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Transport(msg) => write!(f, "transport failure: {msg}"),
            Self::InvalidUrl(url) => write!(f, "invalid URL: {url}"),
            Self::MalformedResponse(msg) => write!(f, "malformed response: {msg}"),
            Self::BodyTooLarge { limit } => write!(f, "response body exceeded {limit} bytes"),
        }
    }
}

impl std::error::Error for FetchError {}

/// Something that can perform HTTP `GET`s; `&self` backs concurrent requests.
pub trait Fetch: Send + Sync {
    /// Performs the request; `limit` caps the body, and only the caller knows it.
    fn get(
        &self,
        request: Request,
        limit: u64,
    ) -> impl Future<Output = Result<Response, FetchError>> + Send;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_lookup_ignores_case() {
        let response = Response {
            status: 200,
            headers: vec![("Content-Length".into(), "42".into())],
            body: Vec::new(),
        };
        assert_eq!(response.header("content-length"), Some("42"));
        assert_eq!(response.header("CONTENT-LENGTH"), Some("42"));
        assert_eq!(response.header("missing"), None);
    }

    #[test]
    fn success_is_only_the_two_hundreds() {
        for (status, expected) in [
            (199, false),
            (200, true),
            (299, true),
            (300, false),
            (404, false),
        ] {
            let response = Response {
                status,
                headers: Vec::new(),
                body: Vec::new(),
            };
            assert_eq!(response.is_success(), expected, "status {status}");
        }
    }

    #[test]
    fn requests_build_up_fluently() {
        let request = Request::get("https://example.invalid/depot/1/chunk/aa")
            .header("Accept", "*/*")
            .range(0, 1023);
        assert_eq!(request.range, Some((0, 1023)));
        assert_eq!(request.headers.len(), 1);
    }
}
