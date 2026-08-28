use crate::tls::connect_tls;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

const DIRECTORY_HOST: &str = "api.steampowered.com";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CmServer {
    pub endpoint: String,
    pub datacentre: String,
    pub load: u32,
}

pub async fn cm_list(cell_id: u32) -> io::Result<Vec<CmServer>> {
    let path = format!("/ISteamDirectory/GetCMListForConnect/v1/?cellid={cell_id}&maxcount=64");
    let body = get(DIRECTORY_HOST, &path).await?;

    let mut servers = parse_cm_list(&body);
    servers.sort_by_key(|server| server.load);
    Ok(servers)
}

fn parse_cm_list(body: &str) -> Vec<CmServer> {
    let mut servers = Vec::new();

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

fn field(fragment: &str, name: &str) -> Option<String> {
    let needle = format!("\"{name}\":\"");
    let start = fragment.find(&needle)? + needle.len();
    let rest = fragment.get(start..)?;
    let end = rest.find('"')?;
    Some(rest.get(..end)?.to_owned())
}

fn number(fragment: &str, name: &str) -> Option<u32> {
    let needle = format!("\"{name}\":");
    let start = fragment.find(&needle)? + needle.len();
    let rest = fragment.get(start..)?;
    let end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    rest.get(..end)?.parse().ok()
}

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
        assert!(parse_cm_list(r#"[{"type":"websockets","dc":"x"}]"#).is_empty());
    }

    #[test]
    fn an_entry_missing_its_load_sorts_last_rather_than_first() {
        let servers = parse_cm_list(r#"[{"endpoint":"a:443","type":"websockets","dc":"x"}]"#);
        assert_eq!(servers.first().map(|s| s.load), Some(u32::MAX));
    }
}
