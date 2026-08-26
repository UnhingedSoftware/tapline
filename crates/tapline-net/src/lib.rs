//! Steam's CM message layer.
//!
//! This crate speaks the protocol; it does not open the socket. Everything here
//! runs over the [`tapline_io::Transport`] trait, which means the framing, the
//! batch unpacking and the job correlation are all testable against recorded
//! bytes — no account, no network, no flakiness.
//!
//! # The shape of a session
//!
//! 1. Connect a [`tapline_io::Transport`] to a CM (WebSocket; see
//!    `tapline-rt-tokio`).
//! 2. Send `ClientHello`.
//! 3. Send `ClientLogon` — anonymously for dedicated-server content, which is
//!    the common case, or with credentials.
//! 4. Heartbeat at the interval the logon response asks for, or Steam drops the
//!    session.
//! 5. Send requests, correlate replies by job id, and expect most of them to
//!    arrive wrapped in a `Multi` batch alongside unrelated traffic.

mod frame;
mod gzip;
mod multi;
mod session;

pub use frame::{EMsg, Frame, NO_JOB, PROTOBUF_FLAG};
pub use gzip::GzipError;
pub use multi::{MAX_NESTING, MAX_UNZIPPED, expand};
pub use session::{LogonOutcome, Session};

use std::fmt;
use tapline_wire::WireError;

/// What went wrong talking to a CM.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    /// A message ended before its header or body did.
    Truncated,
    /// A message body did not decode.
    Wire(WireError),
    /// A message arrived without the protobuf flag.
    ///
    /// The struct-header format belongs to the TCP transport Steam retired, and
    /// inventing a layout for it on a WebSocket would be guessing.
    NotProtobuf {
        /// The message type, with the flag bit cleared.
        emsg: u32,
    },
    /// A batch claimed a decompressed size we refuse to allocate.
    MultiTooLarge {
        /// What it claimed.
        claimed: u64,
    },
    /// Batches nested past [`MAX_NESTING`].
    MultiNestedTooDeep,
    /// A gzipped batch failed to decompress, or failed its checksum.
    Decompress(GzipError),
    /// The transport failed.
    Io(String),
    /// Steam refused the logon, or ended the session.
    ///
    /// Carries Steam's own result code, because the difference between
    /// "rate limited" and "invalid password" is the difference between waiting
    /// and giving up.
    Steam {
        /// The `EResult` Steam sent.
        eresult: i32,
    },
    /// The peer closed the connection.
    Disconnected,
    /// A reply arrived for a request we were not waiting on, or a request got a
    /// reply of the wrong type.
    UnexpectedReply {
        /// What arrived.
        emsg: u32,
    },
}

impl fmt::Display for NetError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("message ended mid-field"),
            Self::Wire(e) => write!(f, "malformed message: {e}"),
            Self::NotProtobuf { emsg } => {
                write!(f, "message {emsg} arrived without the protobuf flag")
            }
            Self::MultiTooLarge { claimed } => {
                write!(f, "batch claims {claimed} decompressed bytes")
            }
            Self::MultiNestedTooDeep => write!(f, "batches nested deeper than {MAX_NESTING}"),
            Self::Decompress(error) => write!(f, "batch decompression failed: {error}"),
            Self::Io(message) => write!(f, "transport failure: {message}"),
            Self::Steam { eresult } => write!(f, "Steam returned EResult {eresult}"),
            Self::Disconnected => f.write_str("the connection closed"),
            Self::UnexpectedReply { emsg } => write!(f, "unexpected reply {emsg}"),
        }
    }
}

impl std::error::Error for NetError {}

impl From<WireError> for NetError {
    fn from(error: WireError) -> Self {
        Self::Wire(error)
    }
}

impl From<std::io::Error> for NetError {
    fn from(error: std::io::Error) -> Self {
        if error.kind() == std::io::ErrorKind::UnexpectedEof {
            return Self::Disconnected;
        }
        Self::Io(error.to_string())
    }
}
