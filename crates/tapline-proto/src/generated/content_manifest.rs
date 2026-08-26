//! Generated from `content_manifest.proto`. Do not edit — run `cargo xtask gen-proto`.
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

/// `EContentDeltaChunkDataLocation`, as a newtype so an unrecognised value round-trips instead of
/// being rejected. Valve adds values without warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
#[repr(transparent)]
pub struct EContentDeltaChunkDataLocation(pub i32);

impl EContentDeltaChunkDataLocation {
    /// `k_EContentDeltaChunkDataLocationInProtobuf` = `0`
    pub const k_EContentDeltaChunkDataLocationInProtobuf: Self = Self(0);
    /// `k_EContentDeltaChunkDataLocationAfterProtobuf` = `1`
    pub const k_EContentDeltaChunkDataLocationAfterProtobuf: Self = Self(1);
    /// The underlying value, as it appears on the wire.
    #[must_use]
    pub const fn value(self) -> i32 {
        self.0
    }
}

impl From<i32> for EContentDeltaChunkDataLocation {
    fn from(value: i32) -> Self {
        Self(value)
    }
}

/// Types nested inside [`ContentManifestPayload`].
pub mod content_manifest_payload {
    use super::*;

    /// Types nested inside [`FileMapping`].
    pub mod file_mapping {
        use super::*;

        /// `ChunkData` — generated from Valve's schema.
        #[derive(Debug, Clone, PartialEq, Default)]
        pub struct ChunkData {
            /// Field 1.
            pub sha: Option<Vec<u8>>,
            /// Field 2.
            pub crc: Option<u32>,
            /// Field 3.
            pub offset: Option<u64>,
            /// Field 4.
            pub cb_original: Option<u32>,
            /// Field 5.
            pub cb_compressed: Option<u32>,
        }

        impl Message for ChunkData {
            fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
                while let Some(key) = decoder.read_key()? {
                    match key.number {
                        1 => {
                            self.sha = Some(decoder.read_bytes()?.to_vec());
                        }
                        2 => {
                            self.crc = Some(decoder.read_fixed32()?);
                        }
                        3 => {
                            self.offset = Some(decoder.read_varint()?);
                        }
                        4 => {
                            self.cb_original = Some(decoder.read_varint()? as u32);
                        }
                        5 => {
                            self.cb_compressed = Some(decoder.read_varint()? as u32);
                        }
                        _ => decoder.skip_field(key.wire_type)?,
                    }
                }
                Ok(())
            }

            fn encode_raw(&self, encoder: &mut Encoder) {
                if let Some(value) = &self.sha {
                    encoder.write_bytes_field(1, value);
                }
                if let Some(value) = &self.crc {
                    encoder.write_fixed32_field(2, *value);
                }
                if let Some(value) = &self.offset {
                    encoder.write_varint_field(3, *value);
                }
                if let Some(value) = &self.cb_original {
                    encoder.write_varint_field(4, u64::from(*value));
                }
                if let Some(value) = &self.cb_compressed {
                    encoder.write_varint_field(5, u64::from(*value));
                }
            }
        }
    }

    /// `FileMapping` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct FileMapping {
        /// Field 1.
        pub filename: Option<String>,
        /// Field 2.
        pub size: Option<u64>,
        /// Field 3.
        pub flags: Option<u32>,
        /// Field 4.
        pub sha_filename: Option<Vec<u8>>,
        /// Field 5.
        pub sha_content: Option<Vec<u8>>,
        /// Field 6.
        pub chunks: Vec<crate::content_manifest::content_manifest_payload::file_mapping::ChunkData>,
        /// Field 7.
        pub linktarget: Option<String>,
    }

    impl Message for FileMapping {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.filename = Some(decoder.read_string()?.to_owned());
                    }
                    2 => {
                        self.size = Some(decoder.read_varint()?);
                    }
                    3 => {
                        self.flags = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.sha_filename = Some(decoder.read_bytes()?.to_vec());
                    }
                    5 => {
                        self.sha_content = Some(decoder.read_bytes()?.to_vec());
                    }
                    6 => {
                        self.chunks.push({ let mut nested = crate::content_manifest::content_manifest_payload::file_mapping::ChunkData::default(); decoder.read_nested(|d| nested.merge(d))?; nested });
                    }
                    7 => {
                        self.linktarget = Some(decoder.read_string()?.to_owned());
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.filename {
                encoder.write_string_field(1, value);
            }
            if let Some(value) = &self.size {
                encoder.write_varint_field(2, *value);
            }
            if let Some(value) = &self.flags {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.sha_filename {
                encoder.write_bytes_field(4, value);
            }
            if let Some(value) = &self.sha_content {
                encoder.write_bytes_field(5, value);
            }
            for value in &self.chunks {
                encoder.write_message_field(6, value);
            }
            if let Some(value) = &self.linktarget {
                encoder.write_string_field(7, value);
            }
        }
    }
}

/// `ContentManifestPayload` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentManifestPayload {
    /// Field 1.
    pub mappings: Vec<crate::content_manifest::content_manifest_payload::FileMapping>,
}

impl Message for ContentManifestPayload {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.mappings.push({
                        let mut nested =
                            crate::content_manifest::content_manifest_payload::FileMapping::default(
                            );
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        for value in &self.mappings {
            encoder.write_message_field(1, value);
        }
    }
}

/// `ContentManifestMetadata` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentManifestMetadata {
    /// Field 1.
    pub depot_id: Option<u32>,
    /// Field 2.
    pub gid_manifest: Option<u64>,
    /// Field 3.
    pub creation_time: Option<u32>,
    /// Field 4.
    pub filenames_encrypted: Option<bool>,
    /// Field 5.
    pub cb_disk_original: Option<u64>,
    /// Field 6.
    pub cb_disk_compressed: Option<u64>,
    /// Field 7.
    pub unique_chunks: Option<u32>,
    /// Field 8.
    pub crc_encrypted: Option<u32>,
    /// Field 9.
    pub crc_clear: Option<u32>,
}

impl Message for ContentManifestMetadata {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.gid_manifest = Some(decoder.read_varint()?);
                }
                3 => {
                    self.creation_time = Some(decoder.read_varint()? as u32);
                }
                4 => {
                    self.filenames_encrypted = Some(decoder.read_bool()?);
                }
                5 => {
                    self.cb_disk_original = Some(decoder.read_varint()?);
                }
                6 => {
                    self.cb_disk_compressed = Some(decoder.read_varint()?);
                }
                7 => {
                    self.unique_chunks = Some(decoder.read_varint()? as u32);
                }
                8 => {
                    self.crc_encrypted = Some(decoder.read_varint()? as u32);
                }
                9 => {
                    self.crc_clear = Some(decoder.read_varint()? as u32);
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.gid_manifest {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.creation_time {
            encoder.write_varint_field(3, u64::from(*value));
        }
        if let Some(value) = &self.filenames_encrypted {
            encoder.write_bool_field(4, *value);
        }
        if let Some(value) = &self.cb_disk_original {
            encoder.write_varint_field(5, *value);
        }
        if let Some(value) = &self.cb_disk_compressed {
            encoder.write_varint_field(6, *value);
        }
        if let Some(value) = &self.unique_chunks {
            encoder.write_varint_field(7, u64::from(*value));
        }
        if let Some(value) = &self.crc_encrypted {
            encoder.write_varint_field(8, u64::from(*value));
        }
        if let Some(value) = &self.crc_clear {
            encoder.write_varint_field(9, u64::from(*value));
        }
    }
}

/// `ContentManifestSignature` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentManifestSignature {
    /// Field 1.
    pub signature: Option<Vec<u8>>,
}

impl Message for ContentManifestSignature {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.signature = Some(decoder.read_bytes()?.to_vec());
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.signature {
            encoder.write_bytes_field(1, value);
        }
    }
}

/// Types nested inside [`ContentDeltaChunks`].
pub mod content_delta_chunks {
    use super::*;

    /// `DeltaChunk` — generated from Valve's schema.
    #[derive(Debug, Clone, PartialEq, Default)]
    pub struct DeltaChunk {
        /// Field 1.
        pub sha_source: Option<Vec<u8>>,
        /// Field 2.
        pub sha_target: Option<Vec<u8>>,
        /// Field 3.
        pub size_original: Option<u32>,
        /// Field 4.
        pub patch_method: Option<u32>,
        /// Field 5.
        pub chunk: Option<Vec<u8>>,
        /// Field 6.
        pub size_delta: Option<u32>,
    }

    impl Message for DeltaChunk {
        fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
            while let Some(key) = decoder.read_key()? {
                match key.number {
                    1 => {
                        self.sha_source = Some(decoder.read_bytes()?.to_vec());
                    }
                    2 => {
                        self.sha_target = Some(decoder.read_bytes()?.to_vec());
                    }
                    3 => {
                        self.size_original = Some(decoder.read_varint()? as u32);
                    }
                    4 => {
                        self.patch_method = Some(decoder.read_varint()? as u32);
                    }
                    5 => {
                        self.chunk = Some(decoder.read_bytes()?.to_vec());
                    }
                    6 => {
                        self.size_delta = Some(decoder.read_varint()? as u32);
                    }
                    _ => decoder.skip_field(key.wire_type)?,
                }
            }
            Ok(())
        }

        fn encode_raw(&self, encoder: &mut Encoder) {
            if let Some(value) = &self.sha_source {
                encoder.write_bytes_field(1, value);
            }
            if let Some(value) = &self.sha_target {
                encoder.write_bytes_field(2, value);
            }
            if let Some(value) = &self.size_original {
                encoder.write_varint_field(3, u64::from(*value));
            }
            if let Some(value) = &self.patch_method {
                encoder.write_varint_field(4, u64::from(*value));
            }
            if let Some(value) = &self.chunk {
                encoder.write_bytes_field(5, value);
            }
            if let Some(value) = &self.size_delta {
                encoder.write_varint_field(6, u64::from(*value));
            }
        }
    }
}

/// `ContentDeltaChunks` — generated from Valve's schema.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ContentDeltaChunks {
    /// Field 1.
    pub depot_id: Option<u32>,
    /// Field 2.
    pub manifest_id_source: Option<u64>,
    /// Field 3.
    pub manifest_id_target: Option<u64>,
    /// Field 4.
    pub delta_chunks: Vec<crate::content_manifest::content_delta_chunks::DeltaChunk>,
    /// Field 5.
    pub chunk_data_location: Option<crate::content_manifest::EContentDeltaChunkDataLocation>,
}

impl ContentDeltaChunks {
    /// Field 5 , or its schema default when absent.
    #[must_use]
    pub fn chunk_data_location_or_default(
        &self,
    ) -> crate::content_manifest::EContentDeltaChunkDataLocation {
        self.chunk_data_location.unwrap_or(crate::content_manifest::EContentDeltaChunkDataLocation::k_EContentDeltaChunkDataLocationInProtobuf)
    }
}

impl Message for ContentDeltaChunks {
    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {
        while let Some(key) = decoder.read_key()? {
            match key.number {
                1 => {
                    self.depot_id = Some(decoder.read_varint()? as u32);
                }
                2 => {
                    self.manifest_id_source = Some(decoder.read_varint()?);
                }
                3 => {
                    self.manifest_id_target = Some(decoder.read_varint()?);
                }
                4 => {
                    self.delta_chunks.push({
                        let mut nested =
                            crate::content_manifest::content_delta_chunks::DeltaChunk::default();
                        decoder.read_nested(|d| nested.merge(d))?;
                        nested
                    });
                }
                5 => {
                    self.chunk_data_location = Some(
                        crate::content_manifest::EContentDeltaChunkDataLocation::from(
                            decoder.read_varint()? as i32,
                        ),
                    );
                }
                _ => decoder.skip_field(key.wire_type)?,
            }
        }
        Ok(())
    }

    fn encode_raw(&self, encoder: &mut Encoder) {
        if let Some(value) = &self.depot_id {
            encoder.write_varint_field(1, u64::from(*value));
        }
        if let Some(value) = &self.manifest_id_source {
            encoder.write_varint_field(2, *value);
        }
        if let Some(value) = &self.manifest_id_target {
            encoder.write_varint_field(3, *value);
        }
        for value in &self.delta_chunks {
            encoder.write_message_field(4, value);
        }
        if let Some(value) = &self.chunk_data_location {
            encoder.write_varint_field(5, i64::from(value.value()) as u64);
        }
    }
}
