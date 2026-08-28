use crate::NetError;
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_wire::Message;

pub const PROTOBUF_FLAG: u32 = 0x8000_0000;

pub const NO_JOB: u64 = u64::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct EMsg(pub u32);

impl EMsg {
    pub const MULTI: Self = Self(1);
    pub const CLIENT_HELLO: Self = Self(9805);
    pub const CLIENT_LOGON: Self = Self(5514);
    pub const CLIENT_LOGON_RESPONSE: Self = Self(751);
    pub const CLIENT_LOGGED_OFF: Self = Self(757);
    pub const CLIENT_HEARTBEAT: Self = Self(703);
    pub const SERVICE_METHOD_CALL: Self = Self(151);
    pub const SERVICE_METHOD_CALL_NON_AUTHED: Self = Self(9804);
    pub const SERVICE_METHOD_RESPONSE: Self = Self(147);
    pub const GET_DEPOT_DECRYPTION_KEY: Self = Self(5438);
    pub const GET_DEPOT_DECRYPTION_KEY_RESPONSE: Self = Self(5439);
    pub const PICS_PRODUCT_INFO_REQUEST: Self = Self(8903);
    pub const PICS_PRODUCT_INFO_RESPONSE: Self = Self(8904);
    pub const PICS_ACCESS_TOKEN_REQUEST: Self = Self(8905);
    pub const PICS_ACCESS_TOKEN_RESPONSE: Self = Self(8906);
    pub const CLIENT_LICENSE_LIST: Self = Self(780);

    #[must_use]
    pub const fn value(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub emsg: EMsg,
    pub header: CMsgProtoBufHeader,
    pub body: Vec<u8>,
}

impl Frame {
    #[must_use]
    pub fn new(emsg: EMsg, header: CMsgProtoBufHeader, body: Vec<u8>) -> Self {
        Self { emsg, header, body }
    }

    #[must_use]
    pub fn reply_to(&self) -> Option<u64> {
        match self.header.jobid_target {
            Some(NO_JOB) | None => None,
            Some(id) => Some(id),
        }
    }

    pub fn decode_body<T: Message>(&self) -> Result<T, NetError> {
        T::decode(&self.body).map_err(NetError::Wire)
    }

    #[must_use]
    pub fn encode(&self) -> Vec<u8> {
        let header = self.header.encode_to_vec();
        let mut out = Vec::with_capacity(8 + header.len() + self.body.len());
        out.extend_from_slice(&(self.emsg.value() | PROTOBUF_FLAG).to_le_bytes());
        out.extend_from_slice(&(header.len() as u32).to_le_bytes());
        out.extend_from_slice(&header);
        out.extend_from_slice(&self.body);
        out
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, NetError> {
        let raw_emsg = read_u32(bytes, 0).ok_or(NetError::Truncated)?;

        if raw_emsg & PROTOBUF_FLAG == 0 {
            return Err(NetError::NotProtobuf {
                emsg: raw_emsg & !PROTOBUF_FLAG,
            });
        }
        let emsg = EMsg(raw_emsg & !PROTOBUF_FLAG);

        let header_len = read_u32(bytes, 4).ok_or(NetError::Truncated)? as usize;
        let header_start = 8_usize;
        let header_end = header_start
            .checked_add(header_len)
            .ok_or(NetError::Truncated)?;

        let header_bytes = bytes
            .get(header_start..header_end)
            .ok_or(NetError::Truncated)?;
        let header = CMsgProtoBufHeader::decode(header_bytes).map_err(NetError::Wire)?;
        let body = bytes.get(header_end..).ok_or(NetError::Truncated)?.to_vec();

        Ok(Self { emsg, header, body })
    }
}

fn read_u32(bytes: &[u8], offset: usize) -> Option<u32> {
    let end = offset.checked_add(4)?;
    let slice = bytes.get(offset..end)?;
    let array: [u8; 4] = slice.try_into().ok()?;
    Some(u32::from_le_bytes(array))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header_with(jobid_target: Option<u64>) -> CMsgProtoBufHeader {
        CMsgProtoBufHeader {
            jobid_target,
            ..CMsgProtoBufHeader::default()
        }
    }

    #[test]
    fn a_frame_round_trips() {
        let frame = Frame::new(
            EMsg::CLIENT_LOGON,
            CMsgProtoBufHeader {
                client_sessionid: Some(7),
                jobid_source: Some(42),
                ..CMsgProtoBufHeader::default()
            },
            b"body bytes".to_vec(),
        );

        let decoded = Frame::decode(&frame.encode()).expect("must decode");
        assert_eq!(decoded.emsg, EMsg::CLIENT_LOGON);
        assert_eq!(decoded.header.client_sessionid, Some(7));
        assert_eq!(decoded.header.jobid_source, Some(42));
        assert_eq!(decoded.body, b"body bytes");
    }

    #[test]
    fn the_protobuf_flag_is_set_on_the_wire_and_stripped_on_the_way_in() {
        let frame = Frame::new(
            EMsg::CLIENT_HEARTBEAT,
            CMsgProtoBufHeader::default(),
            vec![],
        );
        let encoded = frame.encode();

        let raw = u32::from_le_bytes(
            encoded
                .get(..4)
                .and_then(|s| s.try_into().ok())
                .expect("four bytes"),
        );
        assert_eq!(raw & PROTOBUF_FLAG, PROTOBUF_FLAG, "flag must be set");
        assert_eq!(raw & !PROTOBUF_FLAG, EMsg::CLIENT_HEARTBEAT.value());

        let decoded = Frame::decode(&encoded).expect("must decode");
        assert_eq!(
            decoded.emsg,
            EMsg::CLIENT_HEARTBEAT,
            "flag must be stripped"
        );
    }

    #[test]
    fn the_no_job_sentinel_is_not_a_job_id() {
        assert_eq!(
            Frame::new(EMsg::MULTI, header_with(Some(NO_JOB)), vec![]).reply_to(),
            None
        );
        assert_eq!(
            Frame::new(EMsg::MULTI, header_with(None), vec![]).reply_to(),
            None
        );
        assert_eq!(
            Frame::new(EMsg::MULTI, header_with(Some(9)), vec![]).reply_to(),
            Some(9)
        );
    }

    #[test]
    fn a_non_protobuf_message_is_refused_rather_than_guessed_at() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&EMsg::CLIENT_LOGON.value().to_le_bytes());
        bytes.extend_from_slice(&[0; 16]);

        assert!(matches!(
            Frame::decode(&bytes),
            Err(NetError::NotProtobuf { emsg }) if emsg == EMsg::CLIENT_LOGON.value()
        ));
    }

    #[test]
    fn a_frame_cut_inside_its_header_is_an_error() {
        let header = CMsgProtoBufHeader {
            client_sessionid: Some(1),
            ..CMsgProtoBufHeader::default()
        };
        let header_end = 8 + header.encode_to_vec().len();
        let full = Frame::new(EMsg::CLIENT_LOGON, header, b"xyz".to_vec()).encode();

        for cut in 0..header_end {
            let prefix = full.get(..cut).expect("in range");
            assert!(
                Frame::decode(prefix).is_err(),
                "a {cut}-byte prefix, cut inside the header, decoded"
            );
        }

        for cut in header_end..=full.len() {
            let prefix = full.get(..cut).expect("in range");
            let decoded = Frame::decode(prefix).expect("a complete header must parse");
            assert_eq!(decoded.body.len(), cut - header_end);
        }
    }

    #[test]
    fn a_lying_header_length_does_not_read_past_the_message() {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&(EMsg::MULTI.value() | PROTOBUF_FLAG).to_le_bytes());
        bytes.extend_from_slice(&u32::MAX.to_le_bytes());
        bytes.extend_from_slice(b"short");

        assert!(matches!(Frame::decode(&bytes), Err(NetError::Truncated)));
    }
}
