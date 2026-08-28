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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetError {
    Truncated,
    Wire(WireError),
    NotProtobuf { emsg: u32 },
    MultiTooLarge { claimed: u64 },
    MultiNestedTooDeep,
    Decompress(GzipError),
    Io(String),
    Steam { eresult: i32 },
    Disconnected,
    UnexpectedReply { emsg: u32 },
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
