# Vendored Steam protobuf definitions

These are Valve's own message definitions, extracted from the Steam client
binaries and tracked by [SteamDatabase/Protobufs][repo]. They are vendored here
rather than fetched at build time so that regenerating the Rust output is
reproducible offline, and so a Steam-side change never silently alters what this
workspace compiles.

Provenance: `SteamDatabase/Protobufs` at commit `6952ebc9741ed42b966bfb235631c468793f234c`
(2026-08-26). Only the transitive import closure of what tapline speaks is kept
— 14 files of the 111 upstream.

`google/protobuf/descriptor.proto` is deliberately absent. It is imported only so
the files can declare custom options (`(description)`, `(method_description)`),
which describe the schema and are not present on the wire. The parser in `xtask`
skips `extend` blocks for the same reason.

## Regenerating

```sh
cargo xtask gen-proto
```

Nothing here is compiled by `cargo build`. The generated Rust in
`crates/tapline-proto/src/generated/` is committed, which is what keeps `prost`
and `protoc` out of every downstream consumer's dependency graph.

## Updating

Re-copy from upstream, update the commit hash above, re-run the generator, and
read the diff in the generated output. A field that vanished upstream is a
protocol change worth understanding before it lands.

[repo]: https://github.com/SteamDatabase/Protobufs
