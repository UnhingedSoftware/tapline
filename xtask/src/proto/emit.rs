//! Turns a parsed schema into Rust that implements `tapline_wire::Message`.
//!
//! Design decisions worth knowing before reading the output:
//!
//! * **Enums become newtypes, not Rust enums.** `pub struct EResult(pub i32)`
//!   with associated constants. Valve ships new enum values constantly, and a
//!   real Rust enum would have to either reject them or carry an `Unknown`
//!   variant that every match arm has to handle. A newtype is total by
//!   construction: an unrecognised value round-trips unchanged.
//! * **Valve's spelling is preserved.** `k_EResultOK` stays `k_EResultOK`
//!   rather than becoming `K_E_RESULT_OK`. Renaming would make the generated
//!   code impossible to cross-reference against Valve's schema, and mangling
//!   introduces collisions of its own.
//! * **`required` is generated as optional.** proto2's `required` is a decode
//!   failure waiting to happen: a peer that omits the field makes the whole
//!   message unreadable. Steam still marks a handful of fields required, and
//!   treating a missing one as `None` is the difference between a degraded
//!   response and a dead connection.
//! * **Every file becomes one module.** Valve's schema has almost no packages,
//!   so type names are effectively global; cross-file references resolve through
//!   a symbol table built over the whole set.

use super::parse::{Enum, Field, FieldType, Label, Message, ProtoFile, Scalar};
use std::collections::BTreeMap;
use std::fmt::Write as _;

/// Maps a fully-qualified proto name (`.A.B.C`) to a Rust path.
pub type SymbolTable = BTreeMap<String, Symbol>;

/// What a proto name resolves to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Symbol {
    /// The Rust path, relative to the generated module root.
    pub path: String,
    /// Whether it names an enum, which changes how a field is encoded.
    pub is_enum: bool,
}

/// Builds the symbol table for a whole set of files.
pub fn build_symbols(files: &[ProtoFile]) -> SymbolTable {
    let mut table = SymbolTable::new();
    for file in files {
        let module = module_name(&file.name);
        for message in &file.messages {
            index_message(&mut table, &module, "", message);
        }
        for enumeration in &file.enums {
            let proto_path = format!(".{}", enumeration.name);
            table.insert(
                proto_path,
                Symbol {
                    path: format!("crate::{module}::{}", enumeration.name),
                    is_enum: true,
                },
            );
        }
    }
    table
}

/// Adds a message and everything nested inside it to the table.
fn index_message(table: &mut SymbolTable, module: &str, prefix: &str, message: &Message) {
    let proto_path = format!("{prefix}.{}", message.name);
    let rust_path = if prefix.is_empty() {
        format!("crate::{module}::{}", message.name)
    } else {
        // Nested types live in a module named for their parent, which is how a
        // reader finds `ChunkData` under `content_manifest_payload::file_mapping`.
        let parent_modules: Vec<String> = prefix
            .trim_start_matches('.')
            .split('.')
            .filter(|s| !s.is_empty())
            .map(snake_case)
            .collect();
        format!(
            "crate::{module}::{}::{}",
            parent_modules.join("::"),
            message.name
        )
    };

    table.insert(
        proto_path.clone(),
        Symbol {
            path: rust_path.clone(),
            is_enum: false,
        },
    );

    for nested in &message.messages {
        index_message(table, module, &proto_path, nested);
    }
    for enumeration in &message.enums {
        let enum_proto_path = format!("{proto_path}.{}", enumeration.name);
        let parent_modules: Vec<String> = proto_path
            .trim_start_matches('.')
            .split('.')
            .filter(|s| !s.is_empty())
            .map(snake_case)
            .collect();
        table.insert(
            enum_proto_path,
            Symbol {
                path: format!(
                    "crate::{module}::{}::{}",
                    parent_modules.join("::"),
                    enumeration.name
                ),
                is_enum: true,
            },
        );
    }
}

/// The Rust module name for a `.proto` file.
pub fn module_name(file_name: &str) -> String {
    file_name
        .trim_end_matches(".proto")
        .replace(['.', '-'], "_")
}

/// Converts a proto identifier to `snake_case`.
///
/// Handles the shapes Valve actually uses: `FileMapping` and `filenames_encrypted`
/// and the occasional `CMsgClientPICSProductInfoRequest`, where a run of capitals
/// must not become one underscore per letter.
pub fn snake_case(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 4);
    let chars: Vec<char> = input.chars().collect();

    for (i, ch) in chars.iter().enumerate() {
        if ch.is_ascii_uppercase() {
            let previous_is_lower = i > 0 && chars.get(i - 1).is_some_and(|c| c.is_lowercase());
            let next_is_lower = chars.get(i + 1).is_some_and(|c| c.is_lowercase());
            let previous_is_upper = i > 0 && chars.get(i - 1).is_some_and(|c| c.is_uppercase());

            if i > 0 && (previous_is_lower || (previous_is_upper && next_is_lower)) {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(*ch);
        }
    }

    // Collapse any doubled underscores the input already contained.
    while out.contains("__") {
        out = out.replace("__", "_");
    }
    out
}

/// Rust keywords that a proto field name may collide with.
const KEYWORDS: &[&str] = &[
    "as", "box", "break", "const", "continue", "crate", "else", "enum", "extern", "false", "fn",
    "for", "if", "impl", "in", "let", "loop", "match", "mod", "move", "mut", "override", "priv",
    "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type", "unsafe",
    "use", "where", "while", "async", "await", "dyn", "abstract", "become", "final", "macro",
    "typeof", "unsized", "virtual", "yield", "try", "gen",
];

/// A field name, escaped if it collides with a Rust keyword.
fn field_name(name: &str) -> String {
    let snake = snake_case(name);
    if KEYWORDS.contains(&snake.as_str()) {
        return format!("r#{snake}");
    }
    snake
}

/// The Rust type for a scalar.
const fn scalar_type(scalar: Scalar) -> &'static str {
    match scalar {
        Scalar::Double => "f64",
        Scalar::Float => "f32",
        Scalar::Int32 | Scalar::Sint32 | Scalar::Sfixed32 => "i32",
        Scalar::Int64 | Scalar::Sint64 | Scalar::Sfixed64 => "i64",
        Scalar::Uint32 | Scalar::Fixed32 => "u32",
        Scalar::Uint64 | Scalar::Fixed64 => "u64",
        Scalar::Bool => "bool",
        Scalar::String => "String",
        Scalar::Bytes => "Vec<u8>",
    }
}

/// Resolves a named type against the symbol table.
///
/// Valve's schema is almost entirely package-less, so a reference may be written
/// fully qualified (`.A.B`), relative to the enclosing message, or bare. All
/// three are tried, innermost scope first, which is protobuf's own rule.
fn resolve<'a>(table: &'a SymbolTable, scope: &str, name: &str) -> Option<&'a Symbol> {
    if name.starts_with('.') {
        return table.get(name);
    }

    // Walk outward from the enclosing scope.
    let mut prefix = scope.to_owned();
    loop {
        let candidate = format!("{prefix}.{name}");
        if let Some(symbol) = table.get(&candidate) {
            return Some(symbol);
        }
        match prefix.rfind('.') {
            Some(index) => prefix.truncate(index),
            None => break,
        }
    }
    table.get(&format!(".{name}"))
}

/// Everything the emitter needs to know about one field.
struct Resolved<'a> {
    field: &'a Field,
    rust_type: String,
    is_enum: bool,
    is_message: bool,
    scalar: Option<Scalar>,
}

fn resolve_field<'a>(
    table: &SymbolTable,
    scope: &str,
    field: &'a Field,
) -> Result<Resolved<'a>, String> {
    match &field.ty {
        FieldType::Scalar(scalar) => Ok(Resolved {
            field,
            rust_type: scalar_type(*scalar).to_owned(),
            is_enum: false,
            is_message: false,
            scalar: Some(*scalar),
        }),
        FieldType::Named(name) => {
            let symbol = resolve(table, scope, name)
                .ok_or_else(|| format!("unresolved type `{name}` referenced from `{scope}`"))?;
            Ok(Resolved {
                field,
                rust_type: symbol.path.clone(),
                is_enum: symbol.is_enum,
                is_message: !symbol.is_enum,
                scalar: None,
            })
        }
    }
}

/// Generates the Rust for one file.
///
/// `claimed` tracks which request types already carry an [`tapline_wire::Rpc`]
/// binding, and `notes` collects anything the caller should be told about — see
/// [`emit_service`] for why a method can end up without one.
pub fn emit_file(
    table: &SymbolTable,
    file: &ProtoFile,
    claimed: &mut BTreeMap<String, String>,
    notes: &mut Vec<String>,
) -> Result<String, String> {
    let mut out = String::new();

    let _ = writeln!(
        out,
        "//! Generated from `{}`. Do not edit — run `cargo xtask gen-proto`.\n\
         //!\n\
         //! Provenance and regeneration are documented in\n\
         //! `crates/tapline-proto/protos/README.md`.\n\
         //!\n\
         //! Valve's own spelling is preserved throughout — `CPublishedFile_Vote_Request`\n\
         //! stays as written — so this can be cross-referenced against the schema\n\
         //! without a translation step. That is what the naming allows below are for.\n\
         #![allow(non_upper_case_globals, non_snake_case, non_camel_case_types)]\n\
         #![allow(unused_imports, clippy::doc_markdown, clippy::too_many_lines)]\n\
         #![allow(clippy::match_single_binding, clippy::struct_excessive_bools)]\n\
         #![allow(clippy::used_underscore_binding, clippy::unreadable_literal)]\n",
        file.name
    );
    out.push_str("use tapline_wire::{Decoder, Encoder, Message, WireError, WireType};\n\n");

    for enumeration in &file.enums {
        emit_enum(&mut out, enumeration, 0);
    }
    for message in &file.messages {
        emit_message(&mut out, table, "", message, 0)?;
    }
    for service in &file.services {
        emit_service(&mut out, table, service, claimed, notes)?;
    }

    Ok(out)
}

/// Emits the `Rpc` bindings for one service.
///
/// A `service` block in the schema is what tells us that
/// `CAuthentication_BeginAuthSessionViaCredentials_Request` is answered by
/// `..._Response` and is addressed as
/// `Authentication.BeginAuthSessionViaCredentials`. Generating the binding means
/// the RPC layer is written once and a caller cannot pair a request with the
/// wrong reply type.
/// Valve sometimes points two methods at the same request type —
/// `PublishedFile.GetUserFiles` and `PublishedFile.GetUserFileCount` share both
/// their request and their response. `Rpc` is keyed on the request type, so only
/// the first can carry the binding; the rest get a target constant instead, so
/// they stay callable by naming the target explicitly. Every one of those is
/// reported rather than dropped quietly.
fn emit_service(
    out: &mut String,
    table: &SymbolTable,
    service: &super::parse::Service,
    claimed: &mut BTreeMap<String, String>,
    notes: &mut Vec<String>,
) -> Result<(), String> {
    for method in &service.methods {
        let request = resolve(table, "", &method.input).ok_or_else(|| {
            format!(
                "unresolved request type `{}` for {}.{}",
                method.input, service.name, method.name
            )
        })?;
        let response = resolve(table, "", &method.output).ok_or_else(|| {
            format!(
                "unresolved response type `{}` for {}.{}",
                method.output, service.name, method.name
            )
        })?;

        let target = format!("{}.{}", service.name, method.name);

        if let Some(existing) = claimed.get(&request.path) {
            notes.push(format!(
                "{target} shares its request type with {existing}; emitted as a target constant \
                 rather than an Rpc binding"
            ));
            let const_name = format!(
                "TARGET_{}_{}",
                snake_case(&service.name).to_uppercase(),
                snake_case(&method.name).to_uppercase()
            );
            let _ = writeln!(
                out,
                "/// Unified-message target for `{target}`.\n\
                 ///\n\
                 /// It shares its request type with `{existing}`, so it cannot carry an\n\
                 /// `Rpc` binding of its own — call it by naming this target.\n\
                 pub const {const_name}: &str = \"{target}\";\n"
            );
            continue;
        }

        claimed.insert(request.path.clone(), target.clone());
        let _ = writeln!(
            out,
            "impl tapline_wire::Rpc for {} {{\n\
             \x20   type Response = {};\n\
             \x20   const TARGET: &'static str = \"{target}\";\n\
             }}\n",
            request.path, response.path
        );
    }
    Ok(())
}

/// Emits an enum as a total newtype over `i32`.
fn emit_enum(out: &mut String, enumeration: &Enum, indent: usize) {
    let pad = "    ".repeat(indent);

    let _ = writeln!(
        out,
        "{pad}/// `{}`, as a newtype so an unrecognised value round-trips instead of\n\
         {pad}/// being rejected. Valve adds values without warning.\n\
         {pad}#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]\n\
         {pad}#[repr(transparent)]\n\
         {pad}pub struct {}(pub i32);\n",
        enumeration.name, enumeration.name
    );

    let _ = writeln!(out, "{pad}impl {} {{", enumeration.name);
    for (name, value) in &enumeration.values {
        let _ = writeln!(
            out,
            "{pad}    /// `{name}` = `{value}`\n{pad}    pub const {name}: Self = Self({value});"
        );
    }
    let _ = writeln!(
        out,
        "{pad}    /// The underlying value, as it appears on the wire.\n\
         {pad}    #[must_use]\n\
         {pad}    pub const fn value(self) -> i32 {{\n\
         {pad}        self.0\n\
         {pad}    }}\n\
         {pad}}}\n\n\
         {pad}impl From<i32> for {} {{\n\
         {pad}    fn from(value: i32) -> Self {{\n\
         {pad}        Self(value)\n\
         {pad}    }}\n\
         {pad}}}\n",
        enumeration.name
    );
}

/// Emits a message struct, its nested types, and its `Message` impl.
fn emit_message(
    out: &mut String,
    table: &SymbolTable,
    scope: &str,
    message: &Message,
    indent: usize,
) -> Result<(), String> {
    let pad = "    ".repeat(indent);
    let inner_scope = format!("{scope}.{}", message.name);

    // Nested types go in a module named for this message, so a reader can find
    // them by following the proto name.
    if !message.messages.is_empty() || !message.enums.is_empty() {
        let _ = writeln!(
            out,
            "{pad}/// Types nested inside [`{}`].\n{pad}pub mod {} {{\n{pad}    use super::*;\n",
            message.name,
            snake_case(&message.name)
        );
        for enumeration in &message.enums {
            emit_enum(out, enumeration, indent + 1);
        }
        for nested in &message.messages {
            emit_message(out, table, &inner_scope, nested, indent + 1)?;
        }
        let _ = writeln!(out, "{pad}}}\n");
    }

    let resolved: Vec<Resolved<'_>> = message
        .fields
        .iter()
        .map(|field| resolve_field(table, &inner_scope, field))
        .collect::<Result<_, _>>()?;

    // Struct definition.
    let _ = writeln!(
        out,
        "{pad}/// `{}` — generated from Valve's schema.\n\
         {pad}#[derive(Debug, Clone, PartialEq, Default)]\n\
         {pad}pub struct {} {{",
        message.name, message.name
    );
    for field in &resolved {
        let name = field_name(&field.field.name);
        let ty = storage_type(field, &message.name);
        let _ = writeln!(
            out,
            "{pad}    /// Field {}.\n{pad}    pub {name}: {ty},",
            field.field.number
        );
    }
    let _ = writeln!(out, "{pad}}}\n");

    emit_accessors(out, &resolved, &message.name, indent);
    emit_message_impl(out, &resolved, &message.name, indent);

    Ok(())
}

/// The struct field type for a resolved field.
fn storage_type(field: &Resolved<'_>, self_name: &str) -> String {
    let base = &field.rust_type;
    match field.field.label {
        Label::Repeated => format!("Vec<{base}>"),
        Label::Optional | Label::Required => {
            // A message that contains itself needs indirection or it would be
            // infinitely sized. Only direct self-reference is handled; indirect
            // recursion would fail to compile, which is a loud failure rather
            // than a silent one.
            if field.is_message && base.ends_with(&format!("::{self_name}")) {
                format!("Option<Box<{base}>>")
            } else {
                format!("Option<{base}>")
            }
        }
    }
}

/// Emits accessors for fields that carry a proto default.
fn emit_accessors(out: &mut String, fields: &[Resolved<'_>], name: &str, indent: usize) {
    let pad = "    ".repeat(indent);
    let with_defaults: Vec<&Resolved<'_>> = fields
        .iter()
        .filter(|f| f.field.default.is_some() && f.field.label != Label::Repeated)
        .collect();
    if with_defaults.is_empty() {
        return;
    }

    let _ = writeln!(out, "{pad}impl {name} {{");
    for field in with_defaults {
        let Some(default) = field.field.default.as_deref() else {
            continue;
        };
        let accessor = field_name(&field.field.name);
        let ty = &field.rust_type;
        let literal = default_literal(field, default);

        // `&str` rather than `String` for text, so the accessor does not
        // allocate just to hand back a constant.
        let (return_type, body) = if ty == "String" {
            (
                "&str".to_owned(),
                format!("self.{accessor}.as_deref().unwrap_or({literal})"),
            )
        } else if ty == "Vec<u8>" {
            (
                "&[u8]".to_owned(),
                format!("self.{accessor}.as_deref().unwrap_or(&[])"),
            )
        } else {
            (ty.clone(), format!("self.{accessor}.unwrap_or({literal})"))
        };

        let _ = writeln!(
            out,
            "{pad}    /// Field {} , or its schema default when absent.\n\
             {pad}    #[must_use]\n\
             {pad}    pub fn {accessor}_or_default(&self) -> {return_type} {{\n\
             {pad}        {body}\n\
             {pad}    }}",
            field.field.number
        );
    }
    let _ = writeln!(out, "{pad}}}\n");
}

/// Renders a proto default as a Rust literal.
fn default_literal(field: &Resolved<'_>, default: &str) -> String {
    if field.is_enum {
        // A named enum constant.
        return format!("{}::{default}", field.rust_type);
    }
    match field.scalar {
        Some(Scalar::String) => {
            if default.starts_with('"') {
                default.to_owned()
            } else {
                format!("{default:?}")
            }
        }
        Some(Scalar::Bytes) => "&[]".to_owned(),
        Some(Scalar::Bool) => default.to_owned(),
        Some(Scalar::Float | Scalar::Double) => {
            if default.contains('.') {
                default.to_owned()
            } else {
                format!("{default}.0")
            }
        }
        _ => {
            // Integer. `u64::MAX` and friends are written out in full by Valve,
            // and a plain literal of the right type is what the field needs.
            format!("{default}_{}", field.rust_type)
        }
    }
}

/// Emits the `Message` implementation: `merge` and `encode_raw`.
fn emit_message_impl(out: &mut String, fields: &[Resolved<'_>], name: &str, indent: usize) {
    let pad = "    ".repeat(indent);

    let _ = writeln!(
        out,
        "{pad}impl Message for {name} {{\n\
         {pad}    fn merge(&mut self, decoder: &mut Decoder<'_>) -> Result<(), WireError> {{\n\
         {pad}        while let Some(key) = decoder.read_key()? {{\n\
         {pad}            match key.number {{"
    );

    for field in fields {
        let accessor = field_name(&field.field.name);
        let number = field.field.number;
        let read = decode_expression(field);

        match field.field.label {
            Label::Repeated => {
                if field.is_message || matches!(field.scalar, Some(Scalar::String | Scalar::Bytes))
                {
                    let _ = writeln!(
                        out,
                        "{pad}                {number} => {{ self.{accessor}.push({read}); }}"
                    );
                } else {
                    // Numeric repeated fields may arrive packed or not, and a
                    // decoder must accept both regardless of what the schema
                    // says, because a sender may change its mind.
                    let read_one = decode_scalar_closure(field);
                    let _ = writeln!(
                        out,
                        "{pad}                {number} => decoder.read_maybe_packed(key.wire_type, &mut self.{accessor}, {read_one})?,"
                    );
                }
            }
            Label::Optional | Label::Required => {
                let _ = writeln!(
                    out,
                    "{pad}                {number} => {{ self.{accessor} = Some({read}); }}"
                );
            }
        }
    }

    let _ = writeln!(
        out,
        "{pad}                _ => decoder.skip_field(key.wire_type)?,\n\
         {pad}            }}\n\
         {pad}        }}\n\
         {pad}        Ok(())\n\
         {pad}    }}\n"
    );

    // A message with no fields still has to satisfy the trait, and naming the
    // parameter would leave it unused. Valve has plenty of empty request types.
    let encoder_binding = if fields.is_empty() {
        "_encoder"
    } else {
        "encoder"
    };
    let _ = writeln!(
        out,
        "{pad}    fn encode_raw(&self, {encoder_binding}: &mut Encoder) {{"
    );
    for field in fields {
        emit_encode_field(out, field, indent + 2, name);
    }
    let _ = writeln!(out, "{pad}    }}\n{pad}}}\n");
}

/// The expression that reads one value of this field's type.
fn decode_expression(field: &Resolved<'_>) -> String {
    if field.is_message {
        let boxed = field.rust_type.clone();
        return format!(
            "{{ let mut nested = {boxed}::default(); decoder.read_nested(|d| nested.merge(d))?; nested }}"
        );
    }
    if field.is_enum {
        return format!("{}::from(decoder.read_varint()? as i32)", field.rust_type);
    }
    match field.scalar {
        Some(Scalar::Bool) => "decoder.read_bool()?".to_owned(),
        Some(Scalar::String) => "decoder.read_string()?.to_owned()".to_owned(),
        Some(Scalar::Bytes) => "decoder.read_bytes()?.to_vec()".to_owned(),
        Some(Scalar::Int32) => "decoder.read_varint()? as i32".to_owned(),
        Some(Scalar::Int64) => "decoder.read_varint()? as i64".to_owned(),
        Some(Scalar::Uint32) => "decoder.read_varint()? as u32".to_owned(),
        Some(Scalar::Uint64) => "decoder.read_varint()?".to_owned(),
        Some(Scalar::Sint32) => "decoder.read_sint32()?".to_owned(),
        Some(Scalar::Sint64) => "decoder.read_sint64()?".to_owned(),
        Some(Scalar::Fixed32) => "decoder.read_fixed32()?".to_owned(),
        Some(Scalar::Fixed64) => "decoder.read_fixed64()?".to_owned(),
        Some(Scalar::Sfixed32) => "decoder.read_fixed32()? as i32".to_owned(),
        Some(Scalar::Sfixed64) => "decoder.read_fixed64()? as i64".to_owned(),
        Some(Scalar::Float) => "decoder.read_float()?".to_owned(),
        Some(Scalar::Double) => "decoder.read_double()?".to_owned(),
        None => "decoder.read_varint()?".to_owned(),
    }
}

/// A closure reading one element, for the packed/unpacked repeated path.
fn decode_scalar_closure(field: &Resolved<'_>) -> String {
    if field.is_enum {
        return format!(
            "|d: &mut Decoder<'_>| Ok({}::from(d.read_varint()? as i32))",
            field.rust_type
        );
    }
    let body = match field.scalar {
        Some(Scalar::Bool) => "d.read_bool()",
        Some(Scalar::Int32) => "d.read_varint().map(|v| v as i32)",
        Some(Scalar::Int64) => "d.read_varint().map(|v| v as i64)",
        Some(Scalar::Uint32) => "d.read_varint().map(|v| v as u32)",
        Some(Scalar::Uint64) => "d.read_varint()",
        Some(Scalar::Sint32) => "d.read_sint32()",
        Some(Scalar::Sint64) => "d.read_sint64()",
        Some(Scalar::Fixed32) => "d.read_fixed32()",
        Some(Scalar::Fixed64) => "d.read_fixed64()",
        Some(Scalar::Sfixed32) => "d.read_fixed32().map(|v| v as i32)",
        Some(Scalar::Sfixed64) => "d.read_fixed64().map(|v| v as i64)",
        Some(Scalar::Float) => "d.read_float()",
        Some(Scalar::Double) => "d.read_double()",
        _ => "d.read_varint()",
    };
    format!("|d: &mut Decoder<'_>| {body}")
}

/// The `Encoder` method that writes this field packed, if one exists.
///
/// Only fixed-width and `u64` varint fields have a packed writer today, which
/// covers every `[packed = true]` field in Valve's schema. Anything else falls
/// back to the unpacked form, which is always legal to send.
fn packed_writer(field: &Resolved<'_>) -> Option<&'static str> {
    match field.scalar? {
        Scalar::Fixed32 => Some("write_packed_fixed32"),
        Scalar::Fixed64 => Some("write_packed_fixed64"),
        Scalar::Uint64 => Some("write_packed_varint"),
        _ => None,
    }
}

/// Emits the encode side of one field.
fn emit_encode_field(out: &mut String, field: &Resolved<'_>, indent: usize, self_name: &str) {
    let pad = "    ".repeat(indent);
    let accessor = field_name(&field.field.name);
    let number = field.field.number;

    let write_one = |value: &str| -> String {
        if field.is_message {
            return format!("encoder.write_message_field({number}, {value});");
        }
        if field.is_enum {
            return format!(
                "encoder.write_varint_field({number}, i64::from({value}.value()) as u64);"
            );
        }
        match field.scalar {
            Some(Scalar::Bool) => format!("encoder.write_bool_field({number}, {value});"),
            Some(Scalar::String) => format!("encoder.write_string_field({number}, {value});"),
            Some(Scalar::Bytes) => format!("encoder.write_bytes_field({number}, {value});"),
            Some(Scalar::Int32) => format!("encoder.write_int32_field({number}, {value});"),
            Some(Scalar::Int64) => {
                format!("encoder.write_varint_field({number}, {value} as u64);")
            }
            Some(Scalar::Uint32) => {
                format!("encoder.write_varint_field({number}, u64::from({value}));")
            }
            Some(Scalar::Uint64) => format!("encoder.write_varint_field({number}, {value});"),
            Some(Scalar::Sint32) => format!("encoder.write_sint32_field({number}, {value});"),
            Some(Scalar::Sint64) => format!("encoder.write_sint64_field({number}, {value});"),
            Some(Scalar::Fixed32) => format!("encoder.write_fixed32_field({number}, {value});"),
            Some(Scalar::Fixed64) => format!("encoder.write_fixed64_field({number}, {value});"),
            Some(Scalar::Sfixed32) => {
                format!("encoder.write_fixed32_field({number}, {value} as u32);")
            }
            Some(Scalar::Sfixed64) => {
                format!("encoder.write_fixed64_field({number}, {value} as u64);")
            }
            Some(Scalar::Float) => format!("encoder.write_float_field({number}, {value});"),
            Some(Scalar::Double) => format!("encoder.write_double_field({number}, {value});"),
            None => format!("encoder.write_varint_field({number}, {value});"),
        }
    };

    // `value` is bound by reference in both arms below. Whether it needs
    // dereferencing depends on what the writer expects:
    //
    //   messages          take &impl Message           -> `value`
    //   strings and bytes coerce from &String / &Vec    -> `value`
    //   enums            call .value() through auto-deref -> `value`
    //   other scalars    are taken by value            -> `*value`
    //
    // This used to be a string substitution over the generated statement, which
    // rewrote the `.value()` in the enum path into `.*value()` and produced 63
    // syntax errors. The expression is built once, correctly, instead.
    let needs_deref = !field.is_message
        && !field.is_enum
        && !matches!(field.scalar, Some(Scalar::String | Scalar::Bytes));

    match field.field.label {
        Label::Repeated => {
            // proto2 does not pack by default, so a repeated numeric field is
            // written one key per value unless the schema asked for packing.
            // Steam accepts either form on the way in — protobuf requires every
            // decoder to — but writing what the schema says keeps our output
            // byte-identical to the reference implementation, which is the only
            // way the differential test can be an equality assertion.
            if field.field.packed {
                if let Some(packed_writer) = packed_writer(field) {
                    let _ = writeln!(
                        out,
                        "{pad}encoder.{packed_writer}({number}, &self.{accessor});"
                    );
                    return;
                }
            }
            let bound = if needs_deref { "*value" } else { "value" };
            let statement = write_one(bound);
            let _ = writeln!(
                out,
                "{pad}for value in &self.{accessor} {{\n{pad}    {statement}\n{pad}}}"
            );
        }
        Label::Optional | Label::Required => {
            let is_boxed = field.is_message && field.rust_type.ends_with(&format!("::{self_name}"));
            let bound = if is_boxed {
                "value.as_ref()"
            } else if needs_deref {
                "*value"
            } else {
                "value"
            };
            let statement = write_one(bound);
            let _ = writeln!(
                out,
                "{pad}if let Some(value) = &self.{accessor} {{\n{pad}    {statement}\n{pad}}}"
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snake_case_handles_valves_naming() {
        assert_eq!(snake_case("FileMapping"), "file_mapping");
        assert_eq!(snake_case("filenames_encrypted"), "filenames_encrypted");
        assert_eq!(snake_case("ChunkData"), "chunk_data");
        // A run of capitals must not become one underscore per letter.
        assert_eq!(
            snake_case("CMsgClientPICSProductInfoRequest"),
            "c_msg_client_pics_product_info_request"
        );
        assert_eq!(snake_case("EResult"), "e_result");
    }

    #[test]
    fn keyword_collisions_are_escaped() {
        assert_eq!(field_name("type"), "r#type");
        assert_eq!(field_name("match"), "r#match");
        assert_eq!(field_name("normal_field"), "normal_field");
    }

    #[test]
    fn module_names_survive_dotted_file_names() {
        assert_eq!(
            module_name("steammessages_auth.steamclient.proto"),
            "steammessages_auth_steamclient"
        );
        assert_eq!(module_name("content_manifest.proto"), "content_manifest");
    }

    #[test]
    fn nested_types_resolve_to_nested_modules() {
        let source = r#"
            message Outer {
                message Inner { optional uint32 a = 1; }
                optional .Outer.Inner inner = 1;
            }
        "#;
        let file = crate::proto::parse::parse("test.proto", source).expect("must parse");
        let table = build_symbols(std::slice::from_ref(&file));

        assert_eq!(
            table.get(".Outer.Inner").map(|s| s.path.as_str()),
            Some("crate::test::outer::Inner")
        );
    }

    #[test]
    fn relative_type_references_resolve_innermost_first() {
        // protobuf resolves a bare name by walking outward from the enclosing
        // scope, and Valve's schema relies on it.
        let source = r#"
            message A {
                message B { optional uint32 x = 1; }
                optional B b = 1;
            }
        "#;
        let file = crate::proto::parse::parse("test.proto", source).expect("must parse");
        let table = build_symbols(std::slice::from_ref(&file));
        let symbol = resolve(&table, ".A", "B").expect("must resolve B from inside A");
        assert_eq!(symbol.path, "crate::test::a::B");
    }

    #[test]
    fn an_unresolved_type_is_an_error() {
        // Better a failed generation than generated code that silently drops a
        // field.
        let source = "message M { optional .DoesNotExist x = 1; }";
        let file = crate::proto::parse::parse("test.proto", source).expect("must parse");
        let table = build_symbols(std::slice::from_ref(&file));
        let error = emit_file(&table, &file, &mut BTreeMap::new(), &mut Vec::new())
            .expect_err("must refuse to generate");
        assert!(error.contains("DoesNotExist"), "unhelpful error: {error}");
    }
}
