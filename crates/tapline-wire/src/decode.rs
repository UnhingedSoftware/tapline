use crate::{FieldKey, MAX_DEPTH, WireType};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WireError {
    Truncated,
    VarintOverflow,
    InvalidWireType(u32),
    InvalidFieldNumber,
    LengthOutOfBounds { claimed: u64, available: usize },
    DepthLimitExceeded,
    InvalidUtf8,
    PackedLengthMismatch,
}

impl fmt::Display for WireError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Truncated => f.write_str("input ended mid-value"),
            Self::VarintOverflow => f.write_str("varint too long for its target type"),
            Self::InvalidWireType(v) => write!(f, "invalid wire type {v}"),
            Self::InvalidFieldNumber => f.write_str("field number 0 is not legal"),
            Self::LengthOutOfBounds { claimed, available } => {
                write!(
                    f,
                    "length prefix claims {claimed} bytes, {available} remain"
                )
            }
            Self::DepthLimitExceeded => write!(f, "nesting deeper than {MAX_DEPTH}"),
            Self::InvalidUtf8 => f.write_str("string field is not valid UTF-8"),
            Self::PackedLengthMismatch => {
                f.write_str("packed field is not a whole number of elements")
            }
        }
    }
}

impl std::error::Error for WireError {}

#[derive(Debug)]
pub struct Decoder<'a> {
    input: &'a [u8],
    pos: usize,
    depth: u32,
}

impl<'a> Decoder<'a> {
    #[must_use]
    pub const fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            depth: 0,
        }
    }

    #[inline]
    #[must_use]
    pub const fn remaining(&self) -> usize {
        self.input.len() - self.pos
    }

    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    fn read_byte(&mut self) -> Result<u8, WireError> {
        let byte = *self.input.get(self.pos).ok_or(WireError::Truncated)?;
        self.pos += 1;
        Ok(byte)
    }

    fn take(&mut self, len: usize) -> Result<&'a [u8], WireError> {
        let end = self.pos.checked_add(len).ok_or(WireError::Truncated)?;
        let slice = self.input.get(self.pos..end).ok_or(WireError::Truncated)?;
        self.pos = end;
        Ok(slice)
    }

    pub fn read_varint(&mut self) -> Result<u64, WireError> {
        let mut value: u64 = 0;
        for shift in 0..10_u32 {
            let byte = self.read_byte()?;
            let payload = u64::from(byte & 0x7F);

            if shift == 9 && payload > 1 {
                return Err(WireError::VarintOverflow);
            }
            value |= payload << (shift * 7);

            if byte & 0x80 == 0 {
                return Ok(value);
            }
        }
        Err(WireError::VarintOverflow)
    }

    pub fn read_varint32(&mut self) -> Result<u32, WireError> {
        Ok(self.read_varint()? as u32)
    }

    pub fn read_bool(&mut self) -> Result<bool, WireError> {
        Ok(self.read_varint()? != 0)
    }

    pub fn read_fixed32(&mut self) -> Result<u32, WireError> {
        let bytes: [u8; 4] = self.take(4)?.try_into().map_err(|_| WireError::Truncated)?;
        Ok(u32::from_le_bytes(bytes))
    }

    pub fn read_fixed64(&mut self) -> Result<u64, WireError> {
        let bytes: [u8; 8] = self.take(8)?.try_into().map_err(|_| WireError::Truncated)?;
        Ok(u64::from_le_bytes(bytes))
    }

    pub fn read_sint32(&mut self) -> Result<i32, WireError> {
        Ok(crate::zigzag_decode_32(self.read_varint()? as u32))
    }

    pub fn read_sint64(&mut self) -> Result<i64, WireError> {
        Ok(crate::zigzag_decode_64(self.read_varint()?))
    }

    pub fn read_float(&mut self) -> Result<f32, WireError> {
        Ok(f32::from_bits(self.read_fixed32()?))
    }

    pub fn read_double(&mut self) -> Result<f64, WireError> {
        Ok(f64::from_bits(self.read_fixed64()?))
    }

    pub fn read_maybe_packed<T>(
        &mut self,
        wire_type: WireType,
        out: &mut Vec<T>,
        read_one: impl Fn(&mut Decoder<'_>) -> Result<T, WireError>,
    ) -> Result<(), WireError> {
        if wire_type != WireType::LengthDelimited {
            out.push(read_one(self)?);
            return Ok(());
        }
        let payload = self.read_bytes()?;
        let mut nested = Decoder::new(payload);
        while !nested.is_empty() {
            out.push(read_one(&mut nested)?);
        }
        Ok(())
    }

    pub fn read_bytes(&mut self) -> Result<&'a [u8], WireError> {
        let claimed = self.read_varint()?;
        let available = self.remaining();
        let len = usize::try_from(claimed)
            .map_err(|_| WireError::LengthOutOfBounds { claimed, available })?;
        if len > available {
            return Err(WireError::LengthOutOfBounds { claimed, available });
        }
        self.take(len)
    }

    pub fn read_string(&mut self) -> Result<&'a str, WireError> {
        std::str::from_utf8(self.read_bytes()?).map_err(|_| WireError::InvalidUtf8)
    }

    pub fn read_key(&mut self) -> Result<Option<FieldKey>, WireError> {
        if self.is_empty() {
            return Ok(None);
        }
        let key = self.read_varint()?;
        let number = u32::try_from(key >> 3).map_err(|_| WireError::InvalidFieldNumber)?;
        if number == 0 {
            return Err(WireError::InvalidFieldNumber);
        }
        let wire_type = WireType::from_bits((key & 0x7) as u32)?;
        Ok(Some(FieldKey { number, wire_type }))
    }

    pub fn read_nested<T>(
        &mut self,
        f: impl FnOnce(&mut Decoder<'_>) -> Result<T, WireError>,
    ) -> Result<T, WireError> {
        if self.depth >= MAX_DEPTH {
            return Err(WireError::DepthLimitExceeded);
        }
        let payload = self.read_bytes()?;
        let mut nested = Decoder {
            input: payload,
            pos: 0,
            depth: self.depth + 1,
        };
        f(&mut nested)
    }

    fn read_packed_fixed<T, const N: usize>(
        &mut self,
        out: &mut Vec<T>,
        convert: impl Fn([u8; N]) -> T,
    ) -> Result<(), WireError> {
        let payload = self.read_bytes()?;
        if payload.len() % N != 0 {
            return Err(WireError::PackedLengthMismatch);
        }
        out.reserve(payload.len() / N);
        for chunk in payload.chunks_exact(N) {
            let bytes: [u8; N] = chunk.try_into().map_err(|_| WireError::Truncated)?;
            out.push(convert(bytes));
        }
        Ok(())
    }

    pub fn read_packed_fixed32(&mut self, out: &mut Vec<u32>) -> Result<(), WireError> {
        self.read_packed_fixed::<u32, 4>(out, u32::from_le_bytes)
    }

    pub fn read_packed_fixed64(&mut self, out: &mut Vec<u64>) -> Result<(), WireError> {
        self.read_packed_fixed::<u64, 8>(out, u64::from_le_bytes)
    }

    pub fn read_packed_varint(&mut self, out: &mut Vec<u64>) -> Result<(), WireError> {
        let payload = self.read_bytes()?;
        let mut nested = Decoder::new(payload);
        while !nested.is_empty() {
            out.push(nested.read_varint()?);
        }
        Ok(())
    }

    pub fn skip_field(&mut self, wire_type: WireType) -> Result<(), WireError> {
        match wire_type {
            WireType::Varint => {
                self.read_varint()?;
            }
            WireType::Fixed32 => {
                self.take(4)?;
            }
            WireType::Fixed64 => {
                self.take(8)?;
            }
            WireType::LengthDelimited => {
                self.read_bytes()?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn varints_decode_the_specification_examples() {
        assert_eq!(Decoder::new(&[0x00]).read_varint(), Ok(0));
        assert_eq!(Decoder::new(&[0x01]).read_varint(), Ok(1));
        assert_eq!(Decoder::new(&[0xAC, 0x02]).read_varint(), Ok(300));
        assert_eq!(
            Decoder::new(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]).read_varint(),
            Ok(u64::from(u32::MAX))
        );
    }

    #[test]
    fn maximum_width_varint_decodes() {
        let bytes = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x01];
        assert_eq!(Decoder::new(&bytes).read_varint(), Ok(u64::MAX));
    }

    #[test]
    fn over_wide_varints_are_rejected_not_truncated() {
        let eleven = [0x80; 11];
        assert_eq!(
            Decoder::new(&eleven).read_varint(),
            Err(WireError::VarintOverflow)
        );

        let tenth_byte_too_fat = [0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0x02];
        assert_eq!(
            Decoder::new(&tenth_byte_too_fat).read_varint(),
            Err(WireError::VarintOverflow)
        );
    }

    #[test]
    fn truncated_varint_reports_truncation() {
        assert_eq!(
            Decoder::new(&[0x80]).read_varint(),
            Err(WireError::Truncated)
        );
        assert_eq!(Decoder::new(&[]).read_varint(), Err(WireError::Truncated));
    }

    #[test]
    fn lying_length_prefix_is_refused_before_allocating() {
        let bytes = [0x0A, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
        let mut d = Decoder::new(&bytes);
        assert_eq!(d.read_key().unwrap().map(|k| k.number), Some(1));
        assert!(matches!(
            d.read_bytes(),
            Err(WireError::LengthOutOfBounds { .. })
        ));
    }

    #[test]
    fn field_number_zero_is_rejected() {
        assert_eq!(
            Decoder::new(&[0x00]).read_key(),
            Err(WireError::InvalidFieldNumber)
        );
    }

    #[test]
    fn nesting_is_bounded() {
        let mut buf = vec![0x08, 0x01];
        for _ in 0..(MAX_DEPTH + 2) {
            let mut outer = crate::Encoder::new();
            outer.write_bytes_field(1, &buf);
            buf = outer.into_vec();
        }

        fn descend(d: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = d.read_key()? {
                match key.wire_type {
                    WireType::LengthDelimited => d.read_nested(descend)?,
                    other => d.skip_field(other)?,
                }
            }
            Ok(())
        }

        assert_eq!(
            descend(&mut Decoder::new(&buf)),
            Err(WireError::DepthLimitExceeded)
        );
    }

    #[test]
    fn packed_fixed_fields_must_be_a_whole_number_of_elements() {
        let bytes = [0x0A, 0x05, 1, 2, 3, 4, 5];
        let mut d = Decoder::new(&bytes);
        let _ = d.read_key();
        let mut out = Vec::new();
        assert_eq!(
            d.read_packed_fixed32(&mut out),
            Err(WireError::PackedLengthMismatch)
        );
        assert!(out.is_empty(), "partial results must not leak out on error");
    }

    #[test]
    fn unknown_fields_are_skipped_not_fatal() {
        let bytes = [
            0x08, 0x2A, 0x15, 0x01, 0x00, 0x00, 0x00, 0x1A, 0x02, b'h', b'i',
        ];
        let mut d = Decoder::new(&bytes);
        let mut seen = Vec::new();
        while let Some(key) = d.read_key().unwrap() {
            seen.push(key.number);
            d.skip_field(key.wire_type).unwrap();
        }
        assert_eq!(seen, vec![1, 2, 3]);
        assert!(d.is_empty());
    }

    #[test]
    fn strings_reject_invalid_utf8() {
        let bytes = [0x0A, 0x02, 0xFF, 0xFE];
        let mut d = Decoder::new(&bytes);
        let _ = d.read_key();
        assert_eq!(d.read_string(), Err(WireError::InvalidUtf8));
    }
}
