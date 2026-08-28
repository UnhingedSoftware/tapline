use crate::tls::TlsStream;
use std::io;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub const MAX_MESSAGE: usize = 64 * 1024 * 1024;

mod opcode {
    pub const CONTINUATION: u8 = 0x0;
    pub const TEXT: u8 = 0x1;
    pub const BINARY: u8 = 0x2;
    pub const CLOSE: u8 = 0x8;
    pub const PING: u8 = 0x9;
    pub const PONG: u8 = 0xA;
}

struct RawFrame {
    fin: bool,
    opcode: u8,
    payload: Vec<u8>,
}

pub struct WebSocket {
    stream: TlsStream,
    closed: bool,
}

impl WebSocket {
    pub(crate) const fn new(stream: TlsStream) -> Self {
        Self {
            stream,
            closed: false,
        }
    }

    pub async fn send_binary(&mut self, payload: &[u8]) -> io::Result<()> {
        if self.closed {
            return Err(io::Error::from(io::ErrorKind::BrokenPipe));
        }
        self.write_frame(opcode::BINARY, payload).await
    }

    pub async fn recv_binary(&mut self) -> io::Result<Vec<u8>> {
        let mut assembled: Vec<u8> = Vec::new();
        let mut assembling = false;

        loop {
            let frame = self.read_frame().await?;

            match frame.opcode {
                opcode::PING => {
                    self.write_frame(opcode::PONG, &frame.payload).await?;
                }
                opcode::PONG => {}
                opcode::CLOSE => {
                    self.closed = true;
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

    pub async fn close(&mut self) -> io::Result<()> {
        if !self.closed {
            self.closed = true;
            let _ = self
                .write_frame(opcode::CLOSE, &1000_u16.to_be_bytes())
                .await;
        }
        self.stream.shutdown().await
    }

    async fn write_frame(&mut self, opcode: u8, payload: &[u8]) -> io::Result<()> {
        let mut header = Vec::with_capacity(14);
        header.push(0x80 | opcode);

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

        let mask = tapline_crypto::random_bytes::<4>();
        header.extend_from_slice(&mask);

        let mut masked = payload.to_vec();
        for (index, byte) in masked.iter_mut().enumerate() {
            if let Some(key) = mask.get(index % 4) {
                *byte ^= *key;
            }
        }

        self.stream.write_all(&header).await?;
        self.stream.write_all(&masked).await?;
        self.stream.flush().await
    }

    async fn read_frame(&mut self) -> io::Result<RawFrame> {
        let mut first = [0_u8; 2];
        self.stream.read_exact(&mut first).await?;

        let byte0 = *first.first().unwrap_or(&0);
        let byte1 = *first.get(1).unwrap_or(&0);

        let fin = byte0 & 0x80 != 0;
        let opcode = byte0 & 0x0F;
        let masked = byte1 & 0x80 != 0;
        let short_len = byte1 & 0x7F;

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
        assert_eq!(encode_header(opcode::BINARY, 0).len(), 2);
        assert_eq!(encode_header(opcode::BINARY, 125).len(), 2);
        assert_eq!(encode_header(opcode::BINARY, 126).len(), 4);
        assert_eq!(encode_header(opcode::BINARY, 65_535).len(), 4);
        assert_eq!(encode_header(opcode::BINARY, 65_536).len(), 10);
    }

    #[test]
    fn the_mask_bit_is_always_set_on_client_frames() {
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
        assert_ne!(
            tapline_crypto::random_bytes::<4>(),
            tapline_crypto::random_bytes::<4>()
        );
    }
}
