use crate::{FieldKey, Message, WireType};

#[derive(Debug, Default, Clone)]
pub struct Encoder {
    buf: Vec<u8>,
}

impl Encoder {
    #[must_use]
    pub const fn new() -> Self {
        Self { buf: Vec::new() }
    }

    #[must_use]
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            buf: Vec::with_capacity(capacity),
        }
    }

    #[must_use]
    pub fn into_vec(self) -> Vec<u8> {
        self.buf
    }

    #[must_use]
    pub fn as_slice(&self) -> &[u8] {
        &self.buf
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.buf.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    pub fn write_varint(&mut self, mut value: u64) {
        while value >= 0x80 {
            self.buf.push((value as u8) | 0x80);
            value >>= 7;
        }
        self.buf.push(value as u8);
    }

    pub fn write_key(&mut self, key: FieldKey) {
        self.write_varint((u64::from(key.number) << 3) | u64::from(key.wire_type.to_bits()));
    }

    pub fn write_varint_field(&mut self, number: u32, value: u64) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Varint,
        });
        self.write_varint(value);
    }

    pub fn write_int32_field(&mut self, number: u32, value: i32) {
        self.write_varint_field(number, value as i64 as u64);
    }

    pub fn write_bool_field(&mut self, number: u32, value: bool) {
        self.write_varint_field(number, u64::from(value));
    }

    pub fn write_sint32_field(&mut self, number: u32, value: i32) {
        self.write_varint_field(number, u64::from(crate::zigzag_encode_32(value)));
    }

    pub fn write_sint64_field(&mut self, number: u32, value: i64) {
        self.write_varint_field(number, crate::zigzag_encode_64(value));
    }

    pub fn write_float_field(&mut self, number: u32, value: f32) {
        self.write_fixed32_field(number, value.to_bits());
    }

    pub fn write_double_field(&mut self, number: u32, value: f64) {
        self.write_fixed64_field(number, value.to_bits());
    }

    pub fn write_packed_fixed64(&mut self, number: u32, values: &[u64]) {
        if values.is_empty() {
            return;
        }
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });
        self.write_varint((values.len() * 8) as u64);
        for value in values {
            self.buf.extend_from_slice(&value.to_le_bytes());
        }
    }

    pub fn write_fixed32_field(&mut self, number: u32, value: u32) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Fixed32,
        });
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_fixed64_field(&mut self, number: u32, value: u64) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::Fixed64,
        });
        self.buf.extend_from_slice(&value.to_le_bytes());
    }

    pub fn write_bytes_field(&mut self, number: u32, value: &[u8]) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });
        self.write_varint(value.len() as u64);
        self.buf.extend_from_slice(value);
    }

    pub fn write_string_field(&mut self, number: u32, value: &str) {
        self.write_bytes_field(number, value.as_bytes());
    }

    pub fn write_message_field(&mut self, number: u32, message: &impl Message) {
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });

        let placeholder = self.buf.len();
        self.buf.push(0);
        let start = self.buf.len();

        message.encode_raw(self);

        let len = self.buf.len() - start;
        if len < 0x80 {
            if let Some(slot) = self.buf.get_mut(placeholder) {
                *slot = len as u8;
            }
        } else {
            let mut encoded_len = Encoder::new();
            encoded_len.write_varint(len as u64);
            let extra = encoded_len.len() - 1;
            self.buf.splice(placeholder..=placeholder, encoded_len.buf);
            debug_assert_eq!(self.buf.len(), start + len + extra);
        }
    }

    pub fn write_packed_fixed32(&mut self, number: u32, values: &[u32]) {
        if values.is_empty() {
            return;
        }
        self.write_key(FieldKey {
            number,
            wire_type: WireType::LengthDelimited,
        });
        self.write_varint((values.len() * 4) as u64);
        for value in values {
            self.buf.extend_from_slice(&value.to_le_bytes());
        }
    }

    pub fn write_packed_varint(&mut self, number: u32, values: &[u64]) {
        if values.is_empty() {
            return;
        }
        let mut payload = Encoder::new();
        for value in values {
            payload.write_varint(*value);
        }
        self.write_bytes_field(number, payload.as_slice());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Decoder;

    #[test]
    fn varints_encode_the_specification_examples() {
        let mut e = Encoder::new();
        e.write_varint(0);
        e.write_varint(1);
        e.write_varint(300);
        assert_eq!(e.as_slice(), &[0x00, 0x01, 0xAC, 0x02]);
    }

    #[test]
    fn every_varint_round_trips() {
        for value in [0_u64, 1, 127, 128, 300, u32::MAX as u64, u64::MAX] {
            let mut e = Encoder::new();
            e.write_varint(value);
            assert_eq!(
                Decoder::new(e.as_slice()).read_varint(),
                Ok(value),
                "{value}"
            );
        }
    }

    #[test]
    fn negative_int32_uses_the_ten_byte_sign_extended_form() {
        let mut e = Encoder::new();
        e.write_int32_field(1, -1);
        assert_eq!(
            e.as_slice(),
            &[
                0x08, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01
            ]
        );

        let mut d = Decoder::new(e.as_slice());
        let _ = d.read_key();
        assert_eq!(d.read_varint32().map(|v| v as i32), Ok(-1));
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Inner {
        payload: Vec<u8>,
    }

    impl Message for Inner {
        fn merge(&mut self, d: &mut Decoder<'_>) -> Result<(), crate::WireError> {
            while let Some(key) = d.read_key()? {
                if key.number == 1 {
                    self.payload = d.read_bytes()?.to_vec();
                } else {
                    d.skip_field(key.wire_type)?;
                }
            }
            Ok(())
        }

        fn encode_raw(&self, e: &mut Encoder) {
            e.write_bytes_field(1, &self.payload);
        }
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct Outer {
        inner: Inner,
        tail: u64,
    }

    impl Message for Outer {
        fn merge(&mut self, d: &mut Decoder<'_>) -> Result<(), crate::WireError> {
            while let Some(key) = d.read_key()? {
                match key.number {
                    1 => {
                        let mut inner = Inner::default();
                        d.read_nested(|nested| inner.merge(nested))?;
                        self.inner = inner;
                    }
                    2 => self.tail = d.read_varint()?,
                    _ => d.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, e: &mut Encoder) {
            e.write_message_field(1, &self.inner);
            e.write_varint_field(2, self.tail);
        }
    }

    #[test]
    fn short_nested_message_round_trips() {
        let msg = Outer {
            inner: Inner {
                payload: b"hi".to_vec(),
            },
            tail: 7,
        };
        let encoded = msg.encode_to_vec();
        assert_eq!(Outer::decode(&encoded), Ok(msg));
    }

    #[test]
    fn nested_message_longer_than_the_placeholder_splices_correctly() {
        let msg = Outer {
            inner: Inner {
                payload: vec![0xAB; 200],
            },
            tail: 12_345,
        };
        let encoded = msg.encode_to_vec();
        let decoded = Outer::decode(&encoded).expect("must round trip");
        assert_eq!(decoded.inner.payload.len(), 200);
        assert_eq!(decoded.tail, 12_345);
        assert_eq!(decoded, msg);
    }

    #[test]
    fn deeply_long_nested_messages_round_trip_at_every_length_boundary() {
        for len in [
            0_usize, 1, 126, 127, 128, 129, 16_382, 16_383, 16_384, 16_385,
        ] {
            let msg = Outer {
                inner: Inner {
                    payload: vec![0x5A; len],
                },
                tail: 99,
            };
            let decoded = Outer::decode(&msg.encode_to_vec()).expect("must round trip");
            assert_eq!(decoded.inner.payload.len(), len, "payload length {len}");
            assert_eq!(decoded.tail, 99, "trailing field corrupted at length {len}");
        }
    }

    #[test]
    fn empty_packed_fields_are_omitted_entirely() {
        let mut e = Encoder::new();
        e.write_packed_fixed32(1, &[]);
        e.write_packed_varint(2, &[]);
        assert!(e.is_empty());
    }

    #[test]
    fn packed_fields_round_trip() {
        let values = [1_u64, 300, u64::MAX];
        let mut e = Encoder::new();
        e.write_packed_varint(1, &values);

        let mut d = Decoder::new(e.as_slice());
        let _ = d.read_key();
        let mut out = Vec::new();
        d.read_packed_varint(&mut out).expect("must decode");
        assert_eq!(out, values);
    }
}
