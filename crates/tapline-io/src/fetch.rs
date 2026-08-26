//! The HTTP side of the seam.
//!
//! Narrow on purpose: the CDN needs `GET`, optionally with a byte range, and
//! nothing else. A general HTTP trait would be a larger thing to implement for a
//! test double and would invite the rest of the workspace to reach for verbs the
//! protocol never uses.

use std::fmt;
use std::future::Future;

/// A `GET` request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// The absolute URL.
    pub url: String,
    /// Extra request headers, as name/value pairs.
    pub headers: Vec<(String, String)>,
    /// An optional byte range, inclusive of both ends.
    ///
    /// Used to resume a partially fetched chunk rather than starting over.
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

/// A response.
///
/// The body is held in memory because every body tapline fetches is one chunk —
/// a megabyte at the outside — and streaming it would buy nothing but a more
/// complicated trait. Manifests are larger but still bounded, and are fetched
/// once per depot.
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
    ///
    /// Distinct from an HTTP error status because the host pool treats them
    /// differently: a transport failure demotes the host, a 404 does not.
    Transport(String),
    /// The URL could not be parsed, or used a scheme we do not speak.
    InvalidUrl(String),
    /// The response was not valid HTTP.
    MalformedResponse(String),
    /// The body was larger than the caller said it would accept.
    ///
    /// A CDN that returns a gigabyte for a one-megabyte chunk is either broken
    /// or hostile, and either way the download must not follow it into swap.
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

/// Something that can perform HTTP `GET`s.
///
/// Takes `&self` so one fetcher backs many concurrent requests — the connection
/// pool lives inside the implementation, which is where the per-host limits and
/// keep-alive reuse belong.
pub trait Fetch: Send + Sync {
    /// Performs the request.
    ///
    /// `limit` caps the body size. It is a parameter rather than a property of
    /// the fetcher because the caller is the only one who knows how big the
    /// thing it asked for should be: a chunk's size comes from the manifest, so
    /// an over-long response is detectable before it is fully read.
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
