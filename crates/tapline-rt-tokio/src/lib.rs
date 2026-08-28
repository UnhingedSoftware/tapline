mod directory;
mod http;
mod sink;
mod tls;
mod ws;

pub use directory::{CmServer, cm_list};
pub use http::{HttpClient, SharedHttpClient};
pub use sink::FileSink;
pub use tls::connect_tls;
pub use ws::MAX_MESSAGE;

use std::io;
use tapline_io::Transport;

pub struct CmTransport {
    socket: ws::WebSocket,
}

impl CmTransport {
    pub async fn connect(endpoint: &str) -> io::Result<Self> {
        Ok(Self {
            socket: tls::connect_cm(endpoint).await?,
        })
    }
}

impl Transport for CmTransport {
    async fn send(&mut self, message: &[u8]) -> io::Result<()> {
        self.socket.send_binary(message).await
    }

    async fn recv(&mut self) -> io::Result<Vec<u8>> {
        self.socket.recv_binary().await
    }

    async fn close(&mut self) -> io::Result<()> {
        self.socket.close().await
    }
}
