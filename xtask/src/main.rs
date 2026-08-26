//! Build-time tooling for tapline.
//!
//! Currently one job: turn Valve's vendored `.proto` files into Rust that
//! implements `tapline_wire::Message`. The output is committed, so no consumer
//! of this workspace inherits `prost`, `protoc`, or a build script.
//!
//! ```sh
//! cargo xtask gen-proto          # regenerate crates/tapline-proto/src/generated
//! cargo xtask check-proto        # parse every vendored file, generate nothing
//! ```

mod proto;

use std::path::{Path, PathBuf};
use std::process::ExitCode;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    let command = args.next().unwrap_or_default();

    let result = match command.as_str() {
        "check-proto" => check_proto(&workspace_root()),
        "gen-proto" => gen_proto(&workspace_root()),
        other => {
            eprintln!("unknown command `{other}`");
            eprintln!("usage: cargo xtask [check-proto|gen-proto]");
            return ExitCode::FAILURE;
        }
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

/// The workspace root, found by walking up from this crate.
fn workspace_root() -> PathBuf {
    // CARGO_MANIFEST_DIR is xtask's own directory; the workspace is its parent.
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf()
}

/// Where the vendored schema lives.
fn protos_dir(root: &Path) -> PathBuf {
    root.join("crates/tapline-proto/protos")
}

/// Parses every vendored `.proto`, reporting what was found.
///
/// This is a real gate rather than a smoke test: the parser errors on any
/// construct it does not understand, so a clean run means the whole schema is
/// accounted for.
fn check_proto(root: &Path) -> Result<(), String> {
    let dir = protos_dir(root);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "proto"))
        .collect();
    paths.sort();

    if paths.is_empty() {
        return Err(format!("no .proto files in {}", dir.display()));
    }

    let mut messages = 0_usize;
    let mut enums = 0_usize;
    let mut services = 0_usize;

    for path in &paths {
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().into_owned())
            .unwrap_or_default();
        let source = std::fs::read_to_string(path)
            .map_err(|e| format!("cannot read {}: {e}", path.display()))?;

        let file = proto::parse::parse(&name, &source).map_err(|e| e.to_string())?;

        let file_messages = count_messages(&file.messages);
        let file_enums = count_enums(&file.messages) + file.enums.len();
        messages += file_messages;
        enums += file_enums;
        services += file.services.len();

        println!(
            "{name}: {file_messages} messages, {file_enums} enums, {} services",
            file.services.len()
        );
    }

    println!(
        "\n{} files: {messages} messages, {enums} enums, {services} services",
        paths.len()
    );
    Ok(())
}

/// Reads and parses every vendored `.proto`.
fn load_schema(root: &Path) -> Result<Vec<proto::parse::ProtoFile>, String> {
    let dir = protos_dir(root);
    let mut paths: Vec<PathBuf> = std::fs::read_dir(&dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "proto"))
        .collect();
    paths.sort();

    paths
        .iter()
        .map(|path| {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            let source = std::fs::read_to_string(path)
                .map_err(|e| format!("cannot read {}: {e}", path.display()))?;
            proto::parse::parse(&name, &source).map_err(|e| e.to_string())
        })
        .collect()
}

/// Generates the Rust message types and writes them to `tapline-proto`.
fn gen_proto(root: &Path) -> Result<(), String> {
    let files = load_schema(root)?;
    let symbols = proto::emit::build_symbols(&files);

    let out_dir = root.join("crates/tapline-proto/src/generated");
    std::fs::create_dir_all(&out_dir)
        .map_err(|e| format!("cannot create {}: {e}", out_dir.display()))?;

    // Anything left from a previous run whose source file has since been
    // dropped would otherwise linger and still compile.
    if let Ok(entries) = std::fs::read_dir(&out_dir) {
        for entry in entries.filter_map(Result::ok) {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    let mut modules = Vec::new();
    let mut claimed = std::collections::BTreeMap::new();
    let mut notes = Vec::new();
    for file in &files {
        let module = proto::emit::module_name(&file.name);
        let rust = proto::emit::emit_file(&symbols, file, &mut claimed, &mut notes)?;
        let path = out_dir.join(format!("{module}.rs"));
        std::fs::write(&path, rust).map_err(|e| format!("cannot write {}: {e}", path.display()))?;
        println!("wrote {}", path.display());
        modules.push(module);
    }

    let mut root_module = String::from(
        "//! Generated protobuf message types. Do not edit — run `cargo xtask gen-proto`.\n\
         //!\n\
         //! One module per `.proto` file. Provenance is documented in\n\
         //! `crates/tapline-proto/protos/README.md`.\n\n",
    );
    for module in &modules {
        root_module.push_str(&format!("pub mod {module};\n"));
    }
    let path = out_dir.join("mod.rs");
    std::fs::write(&path, root_module)
        .map_err(|e| format!("cannot write {}: {e}", path.display()))?;

    println!("\ngenerated {} modules", modules.len());
    // Never a silent cap: anything the generator could not express is said out
    // loud, so "it generated cleanly" does not quietly mean "minus three RPCs".
    for note in &notes {
        println!("note: {note}");
    }
    Ok(())
}

/// Counts messages including nested ones.
fn count_messages(messages: &[proto::parse::Message]) -> usize {
    messages
        .iter()
        .map(|m| 1 + count_messages(&m.messages))
        .sum()
}

/// Counts enums nested inside messages.
fn count_enums(messages: &[proto::parse::Message]) -> usize {
    messages
        .iter()
        .map(|m| m.enums.len() + count_enums(&m.messages))
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every vendored file must parse.
    ///
    /// This is the M2 gate, and it is worth more than the unit tests: it runs
    /// the parser over 190 KB of Valve's real schema, including the awkward
    /// corners — `extend` blocks declaring custom options, enum values carrying
    /// descriptions, `oneof`, and the 55 KB `EMsg` enum.
    #[test]
    fn every_vendored_proto_parses() {
        let dir = protos_dir(&workspace_root());
        let entries = std::fs::read_dir(&dir).expect("the vendored schema must be present");

        let mut parsed = 0;
        for entry in entries.filter_map(Result::ok) {
            let path = entry.path();
            if path.extension().is_none_or(|ext| ext != "proto") {
                continue;
            }
            let name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .into_owned();
            let source = std::fs::read_to_string(&path).expect("must read the file");

            if let Err(error) = proto::parse::parse(&name, &source) {
                panic!("{error}");
            }
            parsed += 1;
        }

        assert_eq!(parsed, 14, "the vendored file count changed unexpectedly");
    }

    /// The messages the protocol actually depends on must be found, with the
    /// field numbers the wire format uses.
    #[test]
    fn the_manifest_schema_is_read_correctly() {
        let path = protos_dir(&workspace_root()).join("content_manifest.proto");
        let source = std::fs::read_to_string(path).expect("must read content_manifest.proto");
        let file = proto::parse::parse("content_manifest.proto", &source).expect("must parse");

        let metadata = file
            .messages
            .iter()
            .find(|m| m.name == "ContentManifestMetadata")
            .expect("ContentManifestMetadata must be present");

        // These numbers are the wire format. If they ever move, every manifest
        // tapline has ever written becomes unreadable, so they are asserted
        // rather than assumed.
        let field = |name: &str| {
            metadata
                .fields
                .iter()
                .find(|f| f.name == name)
                .map(|f| f.number)
        };
        assert_eq!(field("depot_id"), Some(1));
        assert_eq!(field("gid_manifest"), Some(2));
        assert_eq!(field("filenames_encrypted"), Some(4));
        assert_eq!(field("cb_disk_original"), Some(5));
    }
}
