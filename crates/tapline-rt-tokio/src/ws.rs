//! A WebSocket client, per RFC 6455, scoped to what Steam's CMs need.
//!
//! ```text
//!  0               1               2               3
//!  0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
//! +-+-+-+-+-------+-+-------------+-------------------------------+
//! |F|R|R|R| op    |M| payload len |    extended length (16 or 64)  |
//! |I|S|S|S| code  |A|    (7)      |                               |
//! |N|V|V|V|       |S|             |                               |
//! +-+-+-+-+-------+-+-------------+-------------------------------+
//! |               masking key (client frames only, 4 bytes)       |
//! +---------------------------------------------------------------+
//! |                          payload                              |
//! +---------------------------------------------------------------+
//! ```
//!
//! What is implemented and why:
//!
//! * **Client frames are always masked.** RFC 6455 requires it and Steam's
//!   servers close the connection on an unmasked client frame. The key comes
//!   from the OS RNG, not a counter — the requirement exists to stop cache
//!   poisoning through intermediaries, and a predictable key defeats it.
//! * **Server frames must not be masked**, and one that is gets the connection
//!   closed. That is the RFC's rule and following it costs nothing.
//! * **Fragmentation is handled.** Steam sends large messages — a PICS response
//!   is hundreds of kilobytes — and there is no promise they arrive in one frame.
//! * **Ping is answered with Pong** carrying the same payload, since a CM that
//!   pings and gets nothing back hangs up.
//!
//! Text frames are rejected rather than lossily converted: everything Steam
//! sends on this socket is a binary protocol message, so a text frame means we
//! misunderstood something.

use crate::tls::TlsStream;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

/// The largest message we will assemble, in bytes.
///
/// A frame header can claim a 64-bit length. Real Steam messages are at most a
/// few megabytes; this bound is far above that and far below anything that
/// would exhaust memory on a small node.
pub const MAX_MESSAGE: usize = 64 * 1024 * 1024;

/// Frame opcodes.
mod opcode {
    /// A continuation of the previous frame.
    pub const CONTINUATION: u8 = 0x0;
    /// A UTF-8 text message. Steam never sends one.
    pub const TEXT: u8 = 0x1;
    /// A binary message, which is every Steam protocol message.
    pub const BINARY: u8 = 0x2;
    /// The peer is closing.
    pub const CLOSE: u8 = 0x8;
    /// Answer with `PONG`.
    pub const PING: u8 = 0x9;
    /// An answer to our `PING`, or an unsolicited keepalive.
    pub const PONG: u8 = 0xA;
}

/// One parsed frame.
struct RawFrame {
    fin: bool,
    opcode: u8,
    payload: Vec<u8>,
}

/// A WebSocket connection over TLS.
pub struct WebSocket {
    stream: TlsStream,
    /// Set once a close frame has been sent or received.
    closed: bool,
}

impl WebSocket {
    /// Wraps an already-upgraded TLS stream.
    pub(crate) const fn new(stream: TlsStream) -> Self {
        Self {
            stream,
            closed: false,
        }
    }

    /// Sends one binary message.
    pub async fn send_binary(&mut self, payload: &[u8]) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        self.write_frame(opcode::BINARY, payload).await
    }

    /// Receives the next binary message, answering pings and reassembling
    /// fragments along the way.
    pub async fn recv_binary(&mut self) -> io::Result<Vec<u8>> {
        let mut assembled: Vec<u8> = Vec::new();
        let mut assembling = false;

        loop {
            let frame = self.read_frame().await?;

            match frame.opcode {
                opcode::PING => {
                    // A CM that pings and hears nothing back hangs up.
                    self.write_frame(opcode::PONG, &frame.payload).await?;
                }
                opcode::PONG => {}
                opcode::CLOSE => {
                    self.closed = true;
                    // Echo the close so the peer sees a clean shutdown rather
                    // than a reset.
                    let _ = self.write_frame(opcode::CLOSE, &[]).await;
                    return Err(io::Error::from(io::ErrorKind::UnexpectedEof));
                }
                opcode::TEXT => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "a text frame arrived on a binary protocol socket",
                    ));
                }
                opcode::BINARY | opcode::CONTINUATION => {
                    if frame.opcode == opcode::BINARY && assembling {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "a new message started before the previous one finished",
                        ));
                    }
                    if frame.opcode == opcode::CONTINUATION && !assembling {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "a continuation frame arrived with no message in progress",
                        ));
                    }

                    if assembled.len().saturating_add(frame.payload.len()) > MAX_MESSAGE {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            "message exceeded the size limit",
                        ));
                    }
                    assembled.extend_from_slice(&frame.payload);

                    if frame.fin {
                        return Ok(assembled);
                    }
                    assembling = true;
                }
                other => {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unknown WebSocket opcode {other}"),
                    ));
                }
            }
        }
    }

    /// Sends a close frame and shuts the stream down.
    pub async fn close(&mut self) -> io::Result<()> {
        if !self.closed {
            self.closed = true;
            // 1000: normal closure.
            let _ = self
                .write_frame(opcode::CLOSE, &1000_u16.to_be_bytes())
                .await;
        }
        self.stream.shutdown().await
    }

    /// Writes one frame, masked as a client frame must be.
    async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> io::Result<()> {
        let mut header = Vec::with_capacity(14);
        header.push(0x80 | opcode); // FIN set: we never fragment outbound.

        let mask_bit = 0x80_u8;
        match payload.len() {
            len if len < 126 => header.push(mask_bit | (len as u8)),
            len if len <= usize::from(u16::MAX) => {
                header.push(mask_bit | 126);
                header.extend_from_slice(&(len as u16).to_be_bytes());
            }
            len => {
                header.push(mask_bit | 127);
                header.extend_from_slice(&(len as u64).to_be_bytes());
            }
        }

        // A predictable mask defeats the anti-cache-poisoning reason the mask
        // exists, so this comes from the OS RNG rather than a counter.
        let mask = tapline_crypto::random_bytes::<4>();
        header.extend_from_slice(&mask);

        let mut masked = payload.to_vec();
        for (index, byte) in masked.iter_mut().enumerate() {
            // `index % 4` is always in range for a 4-byte array.
            if let Some(key) = mask.get(index % 4) {
                *byte ^= *key;
            }
        }

        self.stream.write_all(&header).await?;
        self.stream.write_all(&masked).await?;
        self.stream.flush().await
    }

    /// Reads one frame.
    async fn read_frame(&mut self) -> io::Result<RawFrame> {
        let mut first = [0_u8; 2];
        self.stream.read_exact(&mut first).await?;

        let byte0 = *first.first().unwrap_or(&0);
        let byte1 = *first.get(1).unwrap_or(&0);

        let fin = byte0 & 0x80 != 0;
        let opcode = byte0 & 0x0F;
        let masked = byte1 & 0x80 != 0;
        let short_len = byte1 & 0x7F;

        // RFC 6455: a server must not mask. Accepting one anyway would mean
        // accepting a frame from something that is not following the protocol.
        if masked {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "the server masked a frame, which the protocol forbids",
            ));
        }

        let length = match short_len {
            126 => {
                let mut buf = [0_u8; 2];
                self.stream.read_exact(&mut buf).await?;
                usize::from(u16::from_be_bytes(buf))
            }
            127 => {
                let mut buf = [0_u8; 8];
                self.stream.read_exact(&mut buf).await?;
                usize::try_from(u64::from_be_bytes(buf)).map_err(|_| {
                    io::Error::new(io::ErrorKind::InvalidData, "frame length exceeds usize")
                })?
            }
            other => usize::from(other),
        };

        // The length is a number from the network, and this is where it would
        // otherwise become an allocation.
        if length > MAX_MESSAGE {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("frame claims {length} bytes"),
            ));
        }

        let mut payload = vec![0_u8; length];
        self.stream.read_exact(&mut payload).await?;

        Ok(RawFrame {
            fin,
            opcode,
            payload,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a client frame the way [`WebSocket::write_frame`] does, so the
    /// length encoding can be checked without a socket.
    fn encode_header(opcode: u8, len: usize) -> Vec<u8> {
        let mut header = vec![0x80 | opcode];
        match len {
            l if l < 126 => header.push(0x80 | (l as u8)),
            l if l <= usize::from(u16::MAX) => {
                header.push(0x80 | 126);
                header.extend_from_slice(&(l as u16).to_be_bytes());
            }
            l => {
                header.push(0x80 | 127);
                header.extend_from_slice(&(l as u64).to_be_bytes());
            }
        }
        header
    }

    #[test]
    fn length_encoding_switches_at_the_right_boundaries() {
        // 125/126 and 65535/65536 are where the wire format changes shape, and
        // getting either wrong desynchronises the stream for good.
        assert_eq!(encode_header(opcode::BINARY, 0).len(), 2);
        assert_eq!(encode_header(opcode::BINARY, 125).len(), 2);
        assert_eq!(encode_header(opcode::BINARY, 126).len(), 4);
        assert_eq!(encode_header(opcode::BINARY, 65_535).len(), 4);
        assert_eq!(encode_header(opcode::BINARY, 65_536).len(), 10);
    }

    #[test]
    fn the_mask_bit_is_always_set_on_client_frames() {
        // Steam's servers close the connection on an unmasked client frame.
        for len in [0_usize, 125, 126, 70_000] {
            let header = encode_header(opcode::BINARY, len);
            let second = *header.get(1).expect("two bytes");
            assert_eq!(second & 0x80, 0x80, "mask bit missing at length {len}");
        }
    }

    #[test]
    fn masking_is_reversible_and_actually_changes_the_bytes() {
        let payload = b"a steam protocol message".to_vec();
        let mask = tapline_crypto::random_bytes::<4>();

        let mut masked = payload.clone();
        for (index, byte) in masked.iter_mut().enumerate() {
            if let Some(key) = mask.get(index % 4) {
                *byte ^= *key;
            }
        }
        assert_ne!(masked, payload, "masking left the payload unchanged");

        let mut unmasked = masked;
        for (index, byte) in unmasked.iter_mut().enumerate() {
            if let Some(key) = mask.get(index % 4) {
                *byte ^= *key;
            }
        }
        assert_eq!(unmasked, payload);
    }

    #[test]
    fn masks_differ_between_frames() {
        // A counter would satisfy "is masked" while defeating the reason the
        // mask exists.
        assert_ne!(
            tapline_crypto::random_bytes::<4>(),
            tapline_crypto::random_bytes::<4>()
        );
    }
}
