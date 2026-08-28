use crate::ws::WebSocket;
use rustls::pki_types::ServerName;
use std::io;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;

pub type TlsStream = tokio_rustls::client::TlsStream<TcpStream>;

const CM_PATH: &str = "/cmsocket/";

const WEBSOCKET_GUID: &str = "258EAFA5-E914-47DA-95CA-C5AB0DC85B11";

fn client_config() -> Arc<rustls::ClientConfig> {
    let roots = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    Arc::new(
        rustls::ClientConfig::builder()
            .with_root_certificates(roots)
            .with_no_client_auth(),
    )
}

pub async fn connect_tls(host: &str, port: u16) -> io::Result<TlsStream> {
    let tcp = TcpStream::connect((host, port)).await?;
    tcp.set_nodelay(true)?;

    let server_name = ServerName::try_from(host.to_owned())
        .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, format!("bad host {host}")))?;

    TlsConnector::from(client_config())
        .connect(server_name, tcp)
        .await
}

pub async fn connect_cm(endpoint: &str) -> io::Result<WebSocket> {
    let (host, port) = split_endpoint(endpoint)?;
    let stream = connect_tls(host, port).await?;
    upgrade(stream, host, port).await
}

fn split_endpoint(endpoint: &str) -> io::Result<(&str, u16)> {
    match endpoint.rsplit_once(':') {
        Some((host, port)) => {
            let port = port.parse::<u16>().map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("bad port in {endpoint}"),
                )
            })?;
            Ok((host, port))
        }
        None => Ok((endpoint, 443)),
    }
}

async fn upgrade(stream: TlsStream, host: &str, port: u16) -> io::Result<WebSocket> {
    let key = base64_encode(&tapline_crypto::random_bytes::<16>());

    let request = format!(
        "GET {CM_PATH} HTTP/1.1\r\n\
         Host: {host}:{port}\r\n\
         Upgrade: websocket\r\n\
         Connection: Upgrade\r\n\
         Sec-WebSocket-Key: {key}\r\n\
         Sec-WebSocket-Version: 13\r\n\
         \r\n"
    );

    let mut reader = BufReader::new(stream);
    reader.get_mut().write_all(request.as_bytes()).await?;
    reader.get_mut().flush().await?;

    let mut status = String::new();
    reader.read_line(&mut status).await?;
    if !status.starts_with("HTTP/1.1 101") {
        return Err(io::Error::new(
            io::ErrorKind::ConnectionRefused,
            format!("the CM refused the upgrade: {}", status.trim_end()),
        ));
    }

    let mut accept = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("sec-websocket-accept")
        {
            accept = Some(value.trim().to_owned());
        }
    }

    let expected = base64_encode(&tapline_crypto::sha1(
        format!("{key}{WEBSOCKET_GUID}").as_bytes(),
    ));
    match accept {
        Some(value) if value == expected => {}
        Some(value) => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Sec-WebSocket-Accept was {value}, expected {expected}"),
            ));
        }
        None => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the upgrade response carried no Sec-WebSocket-Accept",
            ));
        }
    }

    if !reader.buffer().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "the CM sent frame data before the handshake finished",
        ));
    }

    Ok(WebSocket::new(reader.into_inner()))
}

fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk.first().copied().unwrap_or(0);
        let b1 = chunk.get(1).copied().unwrap_or(0);
        let b2 = chunk.get(2).copied().unwrap_or(0);
        let triple = (u32::from(b0) << 16) | (u32::from(b1) << 8) | u32::from(b2);

        for shift in [18_u32, 12, 6, 0] {
            let index = ((triple >> shift) & 0x3F) as usize;
            let pad_from = match chunk.len() {
                1 => 2,
                2 => 3,
                _ => 4,
            };
            let position = (18 - shift) / 6;
            if position >= pad_from {
                out.push('=');
            } else {
                out.push(char::from(ALPHABET.get(index).copied().unwrap_or(b'A')));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn base64_matches_rfc_4648_vectors() {
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"f"), "Zg==");
        assert_eq!(base64_encode(b"fo"), "Zm8=");
        assert_eq!(base64_encode(b"foo"), "Zm9v");
        assert_eq!(base64_encode(b"foob"), "Zm9vYg==");
        assert_eq!(base64_encode(b"fooba"), "Zm9vYmE=");
        assert_eq!(base64_encode(b"foobar"), "Zm9vYmFy");
    }

    #[test]
    fn the_handshake_accept_value_matches_the_rfc_example() {
        let key = "dGhlIHNhbXBsZSBub25jZQ==";
        let accept = base64_encode(&tapline_crypto::sha1(
            format!("{key}{WEBSOCKET_GUID}").as_bytes(),
        ));
        assert_eq!(accept, "s3pPLMBiTxaQ9kYGzzhZRbK+xOo=");
    }

    #[test]
    fn endpoints_split_the_way_the_cm_directory_writes_them() {
        assert_eq!(
            split_endpoint("cmp1-ord1.steamserver.net:27018").expect("must split"),
            ("cmp1-ord1.steamserver.net", 27_018)
        );
        assert_eq!(
            split_endpoint("cmp1-iad1.steamserver.net:443").expect("must split"),
            ("cmp1-iad1.steamserver.net", 443)
        );
        assert_eq!(
            split_endpoint("cm.example.invalid").expect("must split"),
            ("cm.example.invalid", 443)
        );
        assert!(split_endpoint("host:not-a-port").is_err());
    }
}
