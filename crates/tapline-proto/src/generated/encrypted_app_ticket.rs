//! Generated from `encrypted_app_ticket.proto`. Do not edit — run `cargo xtask gen-proto`.
//!
//! Provenance and regeneration are documented in
//! `crates/tapline-proto/protos/README.md`.
//!
//! Valve's own spelling is preserved throughout — `CPublishedFile_Vote_Request`
//! stays as written — so this can be cross-referenced against the schema
//! without a translation step. That is what the naming allows below are for.
#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]
#![allow(unused_imports, clippy::doc_markdown, clippy::too_many_lines)]
#![allow(clippy::match_single_binding, clippy::struct_excessive_bools)]
#![allow(clippy::used_underscore_binding, clippy::unreadable_literal)]

use tapline_wire::{Decoder, Encoder, Message, WireError, WireType};

/// `EncryptedAppTicket` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct EncryptedAppTicket {
    /// Field 1.
    pub ticket_version_no: Option<u32>,
    /// Field 2.
    pub crc_encryptedticket: Option<u32>,
    /// Field 3.
    pub cb_encrypteduserdata: Option<u32>,
    /// Field 4.
    pub cb_encrypted_appownershipticket: Option<u32>,
    /// Field 5.
    pub encrypted_ticket: Option<Vec<u8>>,
}

impl Message for EncryptedAppTicket {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.ticket_version_no = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.crc_encryptedticket = Some(decoder.read_varint()? as u32);
                }
                3 => {
                    self.cb_encrypteduserdata = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.cb_encrypted_appownershipticket = Some(decoder.read_varint()? as u32);
                }
                5 => {
                    self.encrypted_ticket = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.ticket_version_no {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.crc_encryptedticket {
            encoder.write_varint_field(2, u64::from(*value));
        }
        if let Some(value) = &self.cb_encrypteduserdata {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.cb_encrypted_appownershipticket {
            encoder.write_varint_field(4, u64::from(*value));
        }
        if let Some(value) = &self.encrypted_ticket {
            encoder.write_bytes_field(5, value);
        }
    }
}
