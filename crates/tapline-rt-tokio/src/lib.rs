//! The tokio runtime: the one crate in the workspace that opens a socket.
//!
//! Everything above the leaves speaks to Steam through the traits in
//! `tapline-io`, and this is where those traits meet a real network. Keeping it
//! in one crate means a service that already runs its own reactor links
//! `tapline` without inheriting tokio, and it means the whole protocol stack can
//! be tested against recorded bytes with none of this compiled in.
//!
//! ```no_run
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! use tapline_rt_tokio::CmTransport;
//! use tapline_net::Session;
//!
//! let transport = CmTransport::connect("cmp1-iad1.steamserver.net:443").await?;
//! let mut session = Session::new(transport);
//! let outcome = session.logon_anonymous(0).await?;
//! println!("session {} in cell {}", outcome.session_id, outcome.cell_id);
//! # Ok(())
//! # }
//! ```

mod directory;
mod http;
mod tls;
mod ws;

pub use directory::{CmServer, cm_list};
pub use http::{HttpClient, SharedHttpClient};
pub use tls::connect_tls;
pub use ws::MAX_MESSAGE;

use std::io;
use tapline_io::Transport;

/// A connection to a Steam CM.
///
/// One Steam protocol message per WebSocket frame, which is why the
/// [`Transport`] trait is message-oriented rather than a byte stream.
pub struct CmTransport {
    socket: ws::WebSocket,
}

impl CmTransport {
    /// Connects to a CM at `endpoint`, given as `host:port`.
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
