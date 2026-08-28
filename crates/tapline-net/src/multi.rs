use crate::{NetError, frame::Frame};
use tapline_proto::steammessages_base::CMsgMulti;

pub const MAX_NESTING: u32 = 8;

pub const MAX_UNZIPPED: usize = 32 * 1024 * 1024;

pub fn expand(frame: Frame) -> Result<Vec<Frame>, NetError> {
    let mut out = Vec::new();
    expand_into(frame, 0, &mut out)?;
    Ok(out)
}

fn expand_into(frame: Frame, depth: u32, out: &mut Vec<Frame>) -> Result<(), NetError> {
    use crate::frame::EMsg;

    if frame.emsg != EMsg::MULTI {
        out.push(frame);
        return Ok(());
    }
    if depth >= MAX_NESTING {
        return Err(NetError::MultiNestedTooDeep);
    }

    let multi: CMsgMulti = frame.decode_body()?;
    let body = multi.message_body.unwrap_or_default();

    let payload = match multi.size_unzipped {
        Some(0) | None => body,
        Some(size) => {
            let size = usize::try_from(size).map_err(|_| NetError::MultiTooLarge {
                claimed: u64::from(size),
            })?;
            if size > MAX_UNZIPPED {
                return Err(NetError::MultiTooLarge {
                    claimed: size as u64,
                });
            }
            crate::gzip::decompress(&body, size).map_err(NetError::Decompress)?
        }
    };

    let mut cursor = 0_usize;
    while cursor < payload.len() {
        let len_end = cursor.checked_add(4).ok_or(NetError::Truncated)?;
        let len_bytes: [u8; 4] = payload
            .get(cursor..len_end)
            .and_then(|s| s.try_into().ok())
            .ok_or(NetError::Truncated)?;
        let len = u32::from_le_bytes(len_bytes) as usize;

        let message_end = len_end.checked_add(len).ok_or(NetError::Truncated)?;
        let message = payload
            .get(len_end..message_end)
            .ok_or(NetError::Truncated)?;

        expand_into(Frame::decode(message)?, depth + 1, out)?;
        cursor = message_end;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::EMsg;
    use tapline_proto::steammessages_base::CMsgProtoBufHeader;
    use tapline_wire::Message;

    fn pack(frames: &[Frame]) -> Vec<u8> {
        let mut payload = Vec::new();
        for frame in frames {
            let encoded = frame.encode();
            payload.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
            payload.extend_from_slice(&encoded);
        }
        payload
    }

    fn multi_frame(payload: Vec<u8>, unzipped: Option<u32>) -> Frame {
        let multi = CMsgMulti {
            size_unzipped: unzipped,
            message_body: Some(payload),
        };
        Frame::new(
            EMsg::MULTI,
            CMsgProtoBufHeader::default(),
            multi.encode_to_vec(),
        )
    }

    fn plain(emsg: EMsg, body: &[u8]) -> Frame {
        Frame::new(emsg, CMsgProtoBufHeader::default(), body.to_vec())
    }

    #[test]
    fn a_non_batch_frame_passes_straight_through() {
        let frames = expand(plain(EMsg::CLIENT_LOGON_RESPONSE, b"x")).expect("must expand");
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames.first().map(|f| f.emsg),
            Some(EMsg::CLIENT_LOGON_RESPONSE)
        );
    }

    #[test]
    fn an_uncompressed_batch_yields_its_messages_in_order() {
        let inner = [
            plain(EMsg::CLIENT_LOGON_RESPONSE, b"one"),
            plain(EMsg::CLIENT_LICENSE_LIST, b"two"),
        ];
        let frames = expand(multi_frame(pack(&inner), None)).expect("must expand");

        assert_eq!(frames.len(), 2);
        assert_eq!(
            frames.first().map(|f| f.emsg),
            Some(EMsg::CLIENT_LOGON_RESPONSE)
        );
        assert_eq!(
            frames.first().map(|f| f.body.clone()),
            Some(b"one".to_vec())
        );
        assert_eq!(
            frames.get(1).map(|f| f.emsg),
            Some(EMsg::CLIENT_LICENSE_LIST)
        );
    }

    #[test]
    fn a_gzipped_batch_is_decompressed() {
        let inner = [plain(EMsg::CLIENT_LICENSE_LIST, b"compressed payload")];
        let raw = pack(&inner);
        let gzipped = crate::gzip::compress(&raw);

        let frames = expand(multi_frame(gzipped, Some(raw.len() as u32))).expect("must expand");
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames.first().map(|f| f.body.clone()),
            Some(b"compressed payload".to_vec())
        );
    }

    #[test]
    fn nested_batches_are_flattened() {
        let innermost = [plain(EMsg::CLIENT_LOGON_RESPONSE, b"deep")];
        let inner_multi = multi_frame(pack(&innermost), None);
        let outer = multi_frame(pack(&[inner_multi]), None);

        let frames = expand(outer).expect("must expand");
        assert_eq!(frames.len(), 1);
        assert_eq!(
            frames.first().map(|f| f.body.clone()),
            Some(b"deep".to_vec())
        );
    }

    #[test]
    fn nesting_is_bounded() {
        let mut frame = multi_frame(pack(&[plain(EMsg::CLIENT_HEARTBEAT, b"")]), None);
        for _ in 0..(MAX_NESTING + 2) {
            frame = multi_frame(pack(&[frame]), None);
        }
        assert!(matches!(expand(frame), Err(NetError::MultiNestedTooDeep)));
    }

    #[test]
    fn an_absurd_unzipped_size_is_refused_before_decompressing() {
        let frame = multi_frame(vec![0x1F, 0x8B, 0x08], Some(u32::MAX));
        assert!(matches!(expand(frame), Err(NetError::MultiTooLarge { .. })));
    }

    #[test]
    fn a_truncated_batch_payload_is_an_error() {
        let mut payload = Vec::new();
        payload.extend_from_slice(&1000_u32.to_le_bytes());
        payload.extend_from_slice(b"short");

        assert!(matches!(
            expand(multi_frame(payload, None)),
            Err(NetError::Truncated)
        ));
    }

    #[test]
    fn an_empty_batch_yields_nothing_rather_than_failing() {
        let frames = expand(multi_frame(Vec::new(), None)).expect("must expand");
        assert!(frames.is_empty());
    }
}
