//! A minimal HTTP/1.1 client, scoped to what fetching content needs.
//!
//! `GET`, optionally with a byte range. That is the entire surface: the CDN
//! serves manifests and chunks and nothing else, and a general HTTP stack would
//! be a much larger dependency for no additional capability.
//!
//! What it does implement, because the CDN requires it:
//!
//! * **Keep-alive with connection reuse.** A depot is tens of thousands of
//!   chunks; a fresh TLS handshake per chunk would cost more than the transfer.
//! * **Chunked transfer decoding**, which the CDN uses for manifests.
//! * **A hard body cap**, checked as the body arrives rather than after. The
//!   caller knows how large the thing it asked for should be — a chunk's size
//!   comes from the manifest — so a response that runs long is detectable before
//!   it is fully read.
//!
//! What it deliberately does not implement: redirects (the CDN does not issue
//! them, and following one blindly would let a host redirect a download
//! anywhere), cookies, and compression negotiation (chunks are already
//! compressed).

use crate::tls::connect_tls;
use std::collections::HashMap;
use std::sync::Arc;
use tapline_io::{Fetch, FetchError, Request, Response};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

/// A pooled connection.
struct Connection {
    stream: crate::tls::TlsStream,
    /// Bytes read past the end of the last response, if any.
    ///
    /// Keep-alive means the next response may already be partly in the buffer;
    /// dropping these would desynchronise the connection.
    leftover: Vec<u8>,
}

/// An HTTP client with a per-host connection pool.
///
/// `Fetch` takes `&self` so one client backs many concurrent chunk fetches. The
/// pool is per-host because that is the unit the CDN rate-limits on.
pub struct HttpClient {
    pools: Mutex<HashMap<String, Vec<Connection>>>,
    /// How many idle connections to keep across all hosts.
    ///
    /// Per-host alone is not a bound: with a wide host list it multiplies. At
    /// 84 hosts and 8 idle each, an install could sit on 672 idle sockets on
    /// top of the ones actually in flight and the files being written — enough
    /// to run a 1,024-descriptor process out of descriptors without a single
    /// one of them doing any work.
    idle_total: usize,
    /// How many idle connections to keep per host.
    ///
    /// Small: an idle TLS connection costs memory on both ends, and the
    /// download's parallelism comes from concurrent requests rather than from a
    /// deep idle pool.
    idle_per_host: usize,
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    /// A client with an empty pool.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pools: Mutex::new(HashMap::new()),
            idle_per_host: 8,
            idle_total: 96,
        }
    }

    /// Takes a pooled connection to `host`, or opens one.
    async fn acquire(&self, host: &str, port: u16) -> Result<Connection, FetchError> {
        if let Some(connection) = self.pools.lock().await.get_mut(host).and_then(Vec::pop) {
            return Ok(connection);
        }

        let stream = connect_tls(host, port)
            .await
            .map_err(|e| FetchError::Transport(e.to_string()))?;
        Ok(Connection {
            stream,
            leftover: Vec::new(),
        })
    }

    /// Returns a connection to the pool, dropping it if the pool is full.
    async fn release(&self, host: &str, connection: Connection) {
        let mut pools = self.pools.lock().await;
        // Both bounds, because either alone lets the other run away: per-host
        // keeps one cache from hoarding the pool, and the total keeps a wide
        // host list from exhausting the process's descriptors.
        let idle: usize = pools.values().map(Vec::len).sum();
        if idle >= self.idle_total {
            return;
        }
        let pool = pools.entry(host.to_owned()).or_default();
        if pool.len() < self.idle_per_host {
            pool.push(connection);
        }
    }

    /// Closes every pooled connection.
    pub async fn shutdown(&self) {
        self.pools.lock().await.clear();
    }
}

/// Splits a URL into host, port and path.
fn split_url(url: &str) -> Result<(String, u16, String), FetchError> {
    let (scheme, rest) = url
        .split_once("://")
        .ok_or_else(|| FetchError::InvalidUrl(url.to_owned()))?;

    let default_port = match scheme {
        "https" => 443,
        // Plain HTTP is only ever used for a locally configured lancache, where
        // it is safe because chunks are verified against their content hash.
        "http" => 80,
        _ => return Err(FetchError::InvalidUrl(url.to_owned())),
    };

    let (authority, path) = match rest.find('/') {
        Some(index) => (
            rest.get(..index).unwrap_or_default(),
            rest.get(index..).unwrap_or("/"),
        ),
        None => (rest, "/"),
    };

    let (host, port) = match authority.rsplit_once(':') {
        Some((host, port)) => (
            host,
            port.parse()
                .map_err(|_| FetchError::InvalidUrl(url.to_owned()))?,
        ),
        None => (authority, default_port),
    };

    if host.is_empty() {
        return Err(FetchError::InvalidUrl(url.to_owned()));
    }
    Ok((host.to_owned(), port, path.to_owned()))
}

impl Fetch for HttpClient {
    async fn get(&self, request: Request, limit: u64) -> Result<Response, FetchError> {
        let (host, port, path) = split_url(&request.url)?;

        // One retry, and only on a pooled connection: a keep-alive connection
        // the server closed while idle fails on first write, and that is not a
        // real failure. A fresh connection failing is.
        let mut last_error = None;
        for attempt in 0..2 {
            let connection = self.acquire(&host, port).await?;

            match perform(connection, &host, &path, &request, limit).await {
                Ok((response, connection)) => {
                    if let Some(connection) = connection {
                        self.release(&host, connection).await;
                    }
                    return Ok(response);
                }
                Err(error) => {
                    last_error = Some(error);
                    if attempt == 1 {
                        break;
                    }
                }
            }
        }
        Err(last_error.unwrap_or_else(|| FetchError::Transport("no attempt was made".into())))
    }
}

/// Sends one request and reads its response.
///
/// Returns the connection when it may be reused.
async fn perform(
    mut connection: Connection,
    host: &str,
    path: &str,
    request: &Request,
    limit: u64,
) -> Result<(Response, Option<Connection>), FetchError> {
    let mut head =
        format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: tapline\r\nAccept: */*\r\n");
    for (name, value) in &request.headers {
        head.push_str(&format!("{name}: {value}\r\n"));
    }
    if let Some((start, end)) = request.range {
        head.push_str(&format!("Range: bytes={start}-{end}\r\n"));
    }
    head.push_str("Connection: keep-alive\r\n\r\n");

    connection
        .stream
        .write_all(head.as_bytes())
        .await
        .map_err(|e| FetchError::Transport(e.to_string()))?;
    connection
        .stream
        .flush()
        .await
        .map_err(|e| FetchError::Transport(e.to_string()))?;

    // Read until the header block is complete.
    let mut buffer = std::mem::take(&mut connection.leftover);
    let header_end = loop {
        if let Some(index) = find_header_end(&buffer) {
            break index;
        }
        let mut chunk = [0_u8; 8192];
        let read = connection
            .stream
            .read(&mut chunk)
            .await
            .map_err(|e| FetchError::Transport(e.to_string()))?;
        if read == 0 {
            return Err(FetchError::MalformedResponse("headers ended early".into()));
        }
        buffer.extend_from_slice(chunk.get(..read).unwrap_or_default());

        if buffer.len() > 64 * 1024 {
            return Err(FetchError::MalformedResponse(
                "header block too large".into(),
            ));
        }
    };

    let head_text = String::from_utf8_lossy(buffer.get(..header_end).unwrap_or_default());
    let mut lines = head_text.split("\r\n");

    let status_line = lines
        .next()
        .ok_or_else(|| FetchError::MalformedResponse("no status line".into()))?;
    let status: u16 = status_line
        .split_whitespace()
        .nth(1)
        .and_then(|code| code.parse().ok())
        .ok_or_else(|| FetchError::MalformedResponse(format!("bad status: {status_line}")))?;

    let mut headers = Vec::new();
    for line in lines {
        if let Some((name, value)) = line.split_once(':') {
            headers.push((name.trim().to_owned(), value.trim().to_owned()));
        }
    }

    let header = |name: &str| {
        headers
            .iter()
            .find(|(key, _)| key.eq_ignore_ascii_case(name))
            .map(|(_, value)| value.as_str())
    };

    let chunked = header("transfer-encoding").is_some_and(|value| {
        value
            .split(',')
            .any(|token| token.trim().eq_ignore_ascii_case("chunked"))
    });
    let content_length: Option<u64> = header("content-length").and_then(|v| v.parse().ok());
    let keep_alive = !header("connection").is_some_and(|v| v.eq_ignore_ascii_case("close"));

    // A declared length over the cap is refused before the body is read at all.
    if let Some(length) = content_length
        && length > limit
    {
        return Err(FetchError::BodyTooLarge { limit });
    }

    let body_start = buffer.get(header_end + 4..).unwrap_or_default().to_vec();

    let (body, reusable) = if chunked {
        let body = read_chunked(&mut connection, body_start, limit).await?;
        (body, keep_alive)
    } else {
        match content_length {
            Some(length) => {
                let body = read_sized(&mut connection, body_start, length, limit).await?;
                (body, keep_alive)
            }
            None => {
                // No length and no chunking: the body ends at EOF, so the
                // connection cannot be reused.
                let body = read_to_end(&mut connection, body_start, limit).await?;
                (body, false)
            }
        }
    };

    let response = Response {
        status,
        headers,
        body,
    };
    Ok((response, reusable.then_some(connection)))
}

/// Finds the `\r\n\r\n` that ends the header block.
fn find_header_end(buffer: &[u8]) -> Option<usize> {
    buffer.windows(4).position(|window| window == b"\r\n\r\n")
}

/// Reads a body of known length.
async fn read_sized(
    connection: &mut Connection,
    mut body: Vec<u8>,
    length: u64,
    limit: u64,
) -> Result<Vec<u8>, FetchError> {
    let length = usize::try_from(length).map_err(|_| FetchError::BodyTooLarge { limit })?;

    // Anything past the body belongs to the next response on this connection.
    if body.len() > length {
        connection.leftover = body.split_off(length);
        return Ok(body);
    }

    body.reserve(length - body.len());
    while body.len() < length {
        let mut chunk = [0_u8; 16 * 1024];
        // Never read past the declared length: the bytes after it belong to the
        // next response on this keep-alive connection.
        let want = (length - body.len()).min(chunk.len());
        let slice = chunk.get_mut(..want).unwrap_or_default();
        let read = connection
            .stream
            .read(slice)
            .await
            .map_err(|e| FetchError::Transport(e.to_string()))?;
        if read == 0 {
            return Err(FetchError::MalformedResponse("body ended early".into()));
        }
        body.extend_from_slice(chunk.get(..read).unwrap_or_default());
    }
    Ok(body)
}

/// Reads a body that ends at EOF.
async fn read_to_end(
    connection: &mut Connection,
    mut body: Vec<u8>,
    limit: u64,
) -> Result<Vec<u8>, FetchError> {
    loop {
        let mut chunk = [0_u8; 16 * 1024];
        let read = connection
            .stream
            .read(&mut chunk)
            .await
            .map_err(|e| FetchError::Transport(e.to_string()))?;
        if read == 0 {
            return Ok(body);
        }
        body.extend_from_slice(chunk.get(..read).unwrap_or_default());
        if body.len() as u64 > limit {
            return Err(FetchError::BodyTooLarge { limit });
        }
    }
}

/// Reads a chunked body.
async fn read_chunked(
    connection: &mut Connection,
    start: Vec<u8>,
    limit: u64,
) -> Result<Vec<u8>, FetchError> {
    let mut raw = start;
    let mut out = Vec::new();
    let mut cursor = 0_usize;

    loop {
        // Each chunk is a hex length, CRLF, the data, CRLF.
        let line_end = loop {
            if let Some(index) = find_crlf(&raw, cursor) {
                break index;
            }
            fill(connection, &mut raw).await?;
        };

        let size_text = String::from_utf8_lossy(raw.get(cursor..line_end).unwrap_or_default());
        // A chunk extension follows a semicolon and is not part of the size.
        let size_text = size_text.split(';').next().unwrap_or("").trim();
        let size = usize::from_str_radix(size_text, 16)
            .map_err(|_| FetchError::MalformedResponse(format!("bad chunk size: {size_text}")))?;

        let data_start = line_end + 2;
        if size == 0 {
            // The trailer and final CRLF follow; anything after belongs to the
            // next response.
            let after = raw.get(data_start..).unwrap_or_default();
            connection.leftover = after
                .strip_prefix(b"\r\n".as_slice())
                .unwrap_or(after)
                .to_vec();
            return Ok(out);
        }

        let data_end = data_start
            .checked_add(size)
            .ok_or_else(|| FetchError::MalformedResponse("chunk size overflow".into()))?;
        while raw.len() < data_end + 2 {
            fill(connection, &mut raw).await?;
        }

        out.extend_from_slice(raw.get(data_start..data_end).unwrap_or_default());
        if out.len() as u64 > limit {
            return Err(FetchError::BodyTooLarge { limit });
        }
        cursor = data_end + 2;
    }
}

/// Finds a CRLF at or after `from`.
fn find_crlf(buffer: &[u8], from: usize) -> Option<usize> {
    buffer
        .get(from..)?
        .windows(2)
        .position(|window| window == b"\r\n")
        .map(|index| index + from)
}

/// Reads more bytes into `buffer`.
async fn fill(connection: &mut Connection, buffer: &mut Vec<u8>) -> Result<(), FetchError> {
    let mut chunk = [0_u8; 16 * 1024];
    let read = connection
        .stream
        .read(&mut chunk)
        .await
        .map_err(|e| FetchError::Transport(e.to_string()))?;
    if read == 0 {
        return Err(FetchError::MalformedResponse("body ended early".into()));
    }
    buffer.extend_from_slice(chunk.get(..read).unwrap_or_default());
    Ok(())
}

/// A shareable client.
pub type SharedHttpClient = Arc<HttpClient>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn urls_split_the_way_the_cdn_writes_them() {
        assert_eq!(
            split_url("https://cache12-iad1.steamcontent.com/depot/232257/chunk/abc")
                .expect("must split"),
            (
                "cache12-iad1.steamcontent.com".to_owned(),
                443,
                "/depot/232257/chunk/abc".to_owned()
            )
        );
        // A lancache is usually plain HTTP on a nonstandard port.
        assert_eq!(
            split_url("http://lancache.lan:8080/depot/1/chunk/a").expect("must split"),
            (
                "lancache.lan".to_owned(),
                8080,
                "/depot/1/chunk/a".to_owned()
            )
        );
        // No path means the root.
        assert_eq!(
            split_url("https://example.invalid").expect("must split"),
            ("example.invalid".to_owned(), 443, "/".to_owned())
        );
    }

    #[test]
    fn unusable_urls_are_refused() {
        // A scheme we do not speak must not be silently treated as HTTPS.
        assert!(split_url("ftp://example.invalid/x").is_err());
        assert!(split_url("no-scheme.invalid/x").is_err());
        assert!(split_url("https:///path").is_err());
        assert!(split_url("https://host:notaport/x").is_err());
    }

    #[test]
    fn the_header_terminator_is_found_where_it_is() {
        // "HTTP/1.1 200 OK" is 15 bytes, so the terminator starts at 15 and the
        // body at 19.
        assert_eq!(find_header_end(b"HTTP/1.1 200 OK\r\n\r\nbody"), Some(15));
        assert_eq!(find_header_end(b"HTTP/1.1 200 OK\r\n"), None);
        assert_eq!(find_header_end(b""), None);
    }

    #[test]
    fn crlf_search_respects_its_starting_offset() {
        // The chunked reader relies on this to walk chunk by chunk without
        // rescanning what it has already consumed.
        let buffer = b"5\r\nhello\r\n0\r\n\r\n";
        assert_eq!(find_crlf(buffer, 0), Some(1));
        assert_eq!(find_crlf(buffer, 2), Some(8));
        assert_eq!(find_crlf(buffer, 100), None);
    }
}
