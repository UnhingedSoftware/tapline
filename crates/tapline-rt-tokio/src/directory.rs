//! Finding a CM to connect to.
//!
//! `ISteamDirectory/GetCMListForConnect` is the bootstrap, and it is the one
//! place tapline uses the WebAPI as a primary path rather than an accelerator —
//! there is no CM session yet to ask, so there is nothing to fall back to.
//! Everything afterwards goes over the session.
//!
//! The response is JSON. Rather than take a JSON dependency for one endpoint
//! with three fields we care about, the fields are extracted directly; anything
//! unexpected yields no servers rather than a wrong one.

use crate::tls::connect_tls;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The directory host.
const DIRECTORY_HOST: &str = "api.steampowered.com";

/// A CM the directory offered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmServer {
    /// `host:port`, ready for [`crate::CmTransport::connect`].
    pub endpoint: String,
    /// The datacentre code, useful in a log line when one is misbehaving.
    pub datacentre: String,
    /// Steam's own load figure, lower being better. Used to order the list.
    pub load: u32,
}

/// Fetches CMs for `cell_id`, best first.
///
/// Only `websockets` servers are returned. Measured 2026-08-26, the directory
/// offers 52 of those, 6 `netfilter`, and no TCP at all — `netfilter` is the
/// Steam Datagram Relay transport, which is a different protocol, so returning
/// one here would hand the caller an endpoint that cannot speak CM messages.
pub async fn cm_list(cell_id: u32) -> io::Result<Vec<CmServer>> {
    let path = format!("/ISteamDirectory/GetCMListForConnect/v1/?cellid={cell_id}&maxcount=64");
    let body = get(DIRECTORY_HOST, &path).await?;

    let mut servers = parse_cm_list(&body);
    servers.sort_by_key(|server| server.load);
    Ok(servers)
}

/// Extracts the websocket entries from the directory's JSON.
fn parse_cm_list(body: &str) -> Vec<CmServer> {
    let mut servers = Vec::new();

    // Each entry is a flat object, so splitting on `{` and reading the fields
    // out of each fragment is enough — and cannot mis-nest, because there is no
    // nesting to get wrong.
    for fragment in body.split('{').skip(1) {
        let fragment = fragment.split('}').next().unwrap_or(fragment);

        if field(fragment, "type").as_deref() != Some("websockets") {
            continue;
        }
        let Some(endpoint) = field(fragment, "endpoint") else {
            continue;
        };

        servers.push(CmServer {
            endpoint,
            datacentre: field(fragment, "dc").unwrap_or_default(),
            load: number(fragment, "load").unwrap_or(u32::MAX),
        });
    }
    servers
}

/// Reads a string field out of a flat JSON fragment.
fn field(fragment: &str, name: &str) -> Option<String> {
    let needle = format!("\"{name}\":\"");
    let start = fragment.find(&needle)? + needle.len();
    let rest = fragment.get(start..)?;
    let end = rest.find('"')?;
    Some(rest.get(..end)?.to_owned())
}

/// Reads a numeric field out of a flat JSON fragment.
fn number(fragment: &str, name: &str) -> Option<u32> {
    let needle = format!("\"{name}\":");
    let start = fragment.find(&needle)? + needle.len();
    let rest = fragment.get(start..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest.get(..end)?.parse().ok()
}

/// A minimal HTTPS GET, enough for the one bootstrap endpoint.
async fn get(host: &str, path: &str) -> io::Result<String> {
    let mut stream = connect_tls(host, 443).await?;

    let request = format!(
        "GET {path} HTTP/1.1\r\n\
         Host: {host}\r\n\
         User-Agent: tapline\r\n\
         Accept: application/json\r\n\
         Connection: close\r\n\
         \r\n"
    );
    stream.write_all(request.as_bytes()).await?;
    stream.flush().await?;

    // `Connection: close` means the body ends at EOF, so no chunked decoding is
    // needed here. The CDN client needs keep-alive and gets its own
    // implementation; this one endpoint does not justify sharing it.
    let mut raw = Vec::new();
    stream.read_to_end(&mut raw).await?;
    let text = String::from_utf8_lossy(&raw).into_owned();

    let (head, body) = text
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "no header/body split"))?;

    if !head.starts_with("HTTP/1.1 200") {
        return Err(io::Error::other(format!(
            "directory returned: {}",
            head.lines().next().unwrap_or("")
        )));
    }
    Ok(body.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A trimmed copy of a real response, captured 2026-08-26.
    const REAL_RESPONSE: &str = r#"{"response":{"serverlist":[
        {"endpoint":"cmp1-iad1.steamserver.net:443","legacy_endpoint":"cmp1-iad1.steamserver.net:443","type":"websockets","dc":"iad1","realm":"steamglobal","load":13,"wtd_load":45.24},
        {"endpoint":"cmp2-ord1.steamserver.net:27018","legacy_endpoint":"cmp2-ord1.steamserver.net:27018","type":"websockets","dc":"ord1","realm":"steamglobal","load":12,"wtd_load":45.14},
        {"endpoint":"162.254.192.98:27017","type":"netfilter","dc":"iad1","realm":"steamglobal","load":5,"wtd_load":20.0}
    ]}}"#;

    #[test]
    fn websocket_servers_are_parsed_out_of_a_real_response() {
        let servers = parse_cm_list(REAL_RESPONSE);
        assert_eq!(servers.len(), 2, "expected only the websockets entries");
        assert_eq!(
            servers.first().map(|s| s.endpoint.as_str()),
            Some("cmp1-iad1.steamserver.net:443")
        );
        assert_eq!(servers.first().map(|s| s.load), Some(13));
        assert_eq!(servers.get(1).map(|s| s.datacentre.as_str()), Some("ord1"));
    }

    #[test]
    fn netfilter_servers_are_left_out() {
        // netfilter is the Steam Datagram Relay transport, a different protocol.
        // Handing one to CmTransport would produce a connection that cannot
        // speak CM messages at all.
        let servers = parse_cm_list(REAL_RESPONSE);
        assert!(
            servers.iter().all(|s| !s.endpoint.starts_with("162.254")),
            "a netfilter endpoint survived the filter"
        );
    }

    #[test]
    fn malformed_input_yields_no_servers_rather_than_a_wrong_one() {
        assert!(parse_cm_list("").is_empty());
        assert!(parse_cm_list("not json").is_empty());
        assert!(parse_cm_list(r#"{"response":{"serverlist":[{}]}}"#).is_empty());
        // A websockets entry with no endpoint is unusable, not a default.
        assert!(parse_cm_list(r#"[{"type":"websockets","dc":"x"}]"#).is_empty());
    }

    #[test]
    fn an_entry_missing_its_load_sorts_last_rather_than_first() {
        // Defaulting a missing load to zero would make the least-known server
        // look like the best one.
        let servers = parse_cm_list(r#"[{"endpoint":"a:443","type":"websockets","dc":"x"}]"#);
        assert_eq!(servers.first().map(|s| s.load), Some(u32::MAX));
    }
}
