# tapline — Rust SteamPipe client to replace steamcmd

## Context

`steamcmd` is the only supported way to install dedicated servers and Workshop
content. It is a closed 32-bit-legacy blob, mostly serial, noisy on stdout, has
no machine-readable output, no library surface, and a CLI grammar
(`+app_update 740 validate +quit`) every host tool shells out to and screen-scrapes.
Warden (`~/Documents/warden`) needs exactly this capability on nodes, without
spawning a foreign binary and parsing its console.

Goal: a from-scratch Rust implementation of Steam's content delivery — CM
protocol, auth, PICS, depot manifests, CDN chunk pipeline, Workshop/UGC —
shipped as **(a)** a permissively-licensed library other Rust projects link and
**(b)** a single static binary with a steamcmd-compatible CLI.

Scope boundary, stated up front: this downloads content the signed-in account is
entitled to, exactly as the real client does. Depot keys come from Steam, and
Steam only grants them for owned or anonymous-accessible depots. No key dumping,
no `keys.txt`, no unowned-depot access, no DRM removal, no `.lua` manifest
sideloading — that is the `DepotDownloaderMod` fork family and it is an explicit
non-goal. Upstream `SteamKit2`/`DepotDownloader` are the legitimate precedent.

## Prior art (checked 2026-08-26) — reference only, not dependencies

| crate | license | covers | why not used |
|---|---|---|---|
| [`steam-vent`](https://codeberg.org/steam-vent/steam-vent) 0.5.0 | MIT | CM transport, auth, protobufs | no content layer; we want the protocol in-house |
| [`steamroom`](https://github.com/landaire/steamroom) 0.3.0 | MIT/Apache | CM, CDN, manifests | young (3 releases, breaking each minor), no Workshop, no steamcmd compat |
| [`steamdepot`](https://github.com/elicunninghamdev/stemdepot) 0.3.3 | **LGPL-2.1-only** | depot lib + CLI | copyleft blocks linking into warden |

Nothing does Workshop + steamcmd grammar + ACF state + hardened extraction under
a permissive licence. Their sources stay useful as a **behavioural oracle** when
a protocol detail is ambiguous, alongside SteamKit2 and packet captures of the
real client — read to confirm, never vendored, never linked.

## Build principle: own the whole protocol

Everything Steam-specific is written here: wire codec, CM handshake and channel
encryption, message multiplexer, unified-message RPC, PICS, manifest format,
chunk container formats, VDF/ACF, HTTP client. No third-party Steam library at
any layer, including the transport.

"Depends on nothing" taken to its useful limit, not past it:

- **Crypto is the one exception, deliberately.** AES, RSA, SHA-1/256, HMAC come
  from RustCrypto. Hand-rolled block ciphers and padding are how you ship a
  timing side channel; that is not the place to demonstrate control. `tapline-crypto`
  is a thin wrapper so the primitive source is swappable in one file.
- **TLS is required, measured not assumed.** Checked against live Steam on
  2026-08-26: every one of the 20 hosts returned by
  `IContentServerDirectoryService/GetServersForSteamPipe` reports
  `"https_support":"mandatory"`. So `rustls` is a **core** dependency, not a
  feature — plain HTTP is used only for a locally-configured lancache, where it
  is safe because chunks are SHA-1 verified against their content-addressed ids
  regardless of transport. The CM TCP transport still does its own handshake
  (session key RSA-encrypted to Valve's universe public key, which authenticates
  the peer without a certificate stack), so `tapline-net`'s TCP path needs no TLS;
  the WebSocket path uses it, and gets to skip channel encryption in exchange.
- **Protobuf at runtime is ours.** `tapline-wire` is a small varint/zigzag/
  length-delimited codec plus a `Message` trait. Codegen runs in an `xtask` and
  the output is committed, so no consumer inherits `prost`, `protoc`, or a build
  script. Steam's `.proto` set is proto2 with custom options; the generator
  handles the subset Steam actually uses and errors loudly on anything else
  rather than guessing.
- **HTTP is ours.** `tapline-http` is a minimal HTTP/1.1 client over
  `tokio::net::TcpStream`: keep-alive, pipelined GETs, content-length and chunked
  bodies, range requests. That is the whole surface the CDN needs, and it means
  the connection pool is tuned by us rather than by hyper's defaults.

### The CM session is mandatory — WebAPI is never the primary path

Verified and worth stating because it constrains every target below: PICS product
info, `CMsgClientGetDepotDecryptionKey` and `GetManifestRequestCode` have **no
public HTTP equivalent**. Plenty of Workshop items and dedicated-server apps are
simply not resolvable over the WebAPI — legacy UFS UGC details and anything
behind an entitlement need the authenticated (even anonymous) Steam connection.

So the architecture is CM-first: an open CM session is the base capability, and
the HTTP surface (`ISteamDirectory` bootstrap, `ISteamRemoteStorage` legacy blob
fetch) is an optional accelerator that must always have a CM fallback. Anything
that can only work over WebAPI is treated as a partial mode, never the default.

### Dependency floor (target: ~15 direct, pure-Rust, no C toolchain)

`tokio` (net/rt/io-util/sync/time/fs only) · `rayon` · `bytes` · `rustls` +
`webpki-roots` · RustCrypto (`rsa`, `aes`, `cbc`, `sha1`, `sha2`, `hmac`) ·
`crc32fast` · `lzma-rs` · `ruzstd` · `cap-std` · `tracing`. Feature-gated:
`keyring`, `serde`/`serde_json` (CLI + JSON output), `clap` (CLI). `cargo-deny`
enforces the ceiling in CI, so adding a dependency is a visible decision.

Notably absent: reqwest, hyper, prost, protoc, openssl, any C build.

## Architecture — wide DAG for parallel compile

Compile parallelism comes from **DAG width**, not crate count: rustc can only
build crates whose dependencies are done. So the eight leaves below depend on
nothing internal and all build simultaneously from t=0.

The split is also what makes the thing embeddable: **every crate above the leaves
is IO-free, and all IO enters through four traits.** Nothing in the core opens a
socket or touches a filesystem itself.

```
LEAVES (no internal deps, no IO, no async — all compile in parallel)
  tapline-wire       protobuf varint/zigzag/len-delim codec + Message trait
  tapline-ids        SteamID, AppId, DepotId, ManifestId, PublishedFileId, EResult
  tapline-crypto     RSA-PKCS1 encrypt, AES-CBC/ECB, HMAC-SHA1, SHA1/256, CRC32
  tapline-vdf        text VDF + binary VDF/ACF, read and write
  tapline-lzma       VZ container + LZMA decode
  tapline-zstd       VSZ container + zstd decode
  tapline-event      progress/event vocabulary (plain types, zero deps)
  tapline-io         THE SEAM: traits only, no impls
                       Stream  : async read/write byte stream (CM socket)
                       Fetch   : async HTTP GET with headers/ranges
                       Sink    : sparse random-access file writer
                       Clock/Rng
MID (IO-free: they consume the traits, they never construct one)
  tapline-proto      generated Steam messages   <- wire, ids
                     (split -base / -client / -service if it dominates timings)
  tapline-net        CM handshake, channel crypto, multiplexer, job ids, RPC
  tapline-auth       login flows, refresh-token store
  tapline-pics       product info, access tokens, licenses, branch/depot resolve
  tapline-manifest   manifest ZIP + payload protobuf, filename decrypt, chunk index
  tapline-cdn        server directory, host pool, chunk fetch/decrypt/decompress
  tapline-state      appmanifest/appworkshop ACF read+write

IO IMPL (only one shipped; the seam exists for testing, not for portability theatre)
  tapline-rt-tokio   Stream=TcpStream+rustls, Fetch=own HTTP/1.1, Sink=pwrite/cap-std
  tapline-rt-mock    in-memory Stream/Fetch/Sink for tests (dev-dependency only)

TOP
  tapline            facade: plan/start orchestration, resume, delta
  tapline-cli        steamcmd-compatible grammar + native subcommands
  xtask              .proto -> committed Rust codegen
```

Consequences worth having independent of WASM: the whole protocol stack is
testable against an in-memory `Stream` with no network, a webapp that already
runs its own reactor is not forced to adopt ours, and `tokio` stops being a
dependency of anything except `tapline-rt-tokio`.

Compile-time rules that matter as much as the split:

- **Proc macros serialise builds.** No `serde` derive, no `thiserror` in leaf
  crates — error enums there are hand-written `Display`/`Error` impls. Derives
  live in `tapline-cli` and `tapline-state` where the cost is paid once, off the
  critical path.
- Workspace dev profile: `codegen-units = 256`, `debug = 1`,
  `split-debuginfo = "unpacked"`, no dev LTO.
- Generics kept shallow across crate boundaries (concrete types at the seams) so
  monomorphisation doesn't re-do work in every downstream crate.
- `cargo build --timings` is checked at M2 and M6; if `tapline-proto` is the
  critical path it gets split by `.proto` family.

## Protocol path to implement, in dependency order

1. **Wire** — varint, zigzag, length-delimited, packed repeated; proto2 optional/
   default semantics; unknown fields preserved.
2. **Transport** — TCP to a CM; `ChannelEncryptRequest` → session key encrypted
   to Valve's universe RSA public key → `ChannelEncryptResponse`/`Result`; then
   AES-256-CBC with HMAC-SHA1 framing. Message framing: `VT01` magic, length
   prefix, `EMsg` with the protobuf flag bit, `CMsgProtoBufHeader` for routing,
   job-id request/response correlation, multiplexed over one connection.
3. **CM discovery** — seed list refreshed from `CMsgClientCMList` on every
   session and cached to disk; HTTPS `ISteamDirectory/GetCMListForConnect`
   bootstrap only when the cache is cold and the `tls` feature is on.
4. **Logon** — `CMsgClientLogon` anonymous (the 90% case: dedicated servers) and
   credentialed via `Authentication.GetPasswordRSAPublicKey` +
   `BeginAuthSessionViaCredentials`/`ViaQR` + `PollAuthSessionStatus` → refresh
   and access JWTs. Heartbeat, reconnect, session resumption.
5. **Entitlement** — `CMsgClientLicenseList` → packages → PICS → appids.
6. **PICS** — access tokens + product info; parse `depots{}`: depot ids,
   `manifests{branch}`, `sharedinstall`, `dlcappid`, OS/arch filters,
   `encryptedmanifests` for password betas.
7. **Depot keys** — `CMsgClientGetDepotDecryptionKey`, cached per depot.
8. **CDN directory** — `ContentServerDirectory.GetServersForSteamPipe` → host
   pool; honour lancache (`lancache.steamcontent.com`) so hosting fleets keep
   their cache hits.
9. **Manifest** — `ContentServerDirectory.GetManifestRequestCode` (mandatory for
   modern manifests) → `GET /depot/{id}/manifest/{mid}/5/{code}` → ZIP holding
   payload/metadata/signature protobuf blocks; filenames AES-decrypted when
   `filenames_encrypted`.
10. **Chunks** — `GET /depot/{id}/chunk/{sha1hex}` → AES-256-ECB IV then CBC body
    → `VZ` (LZMA) or `VSZ` (zstd) → SHA-1 verified against the chunk id.
11. **Workshop** — `PublishedFile.GetDetails` → SteamPipe UGC (`hcontent_file` is
    a manifest id in the app's workshop depot) or legacy UFS
    (`ISteamRemoteStorage/GetUGCFileDetails` → plain blob). Both paths ship.
12. **State** — write `steamapps/appmanifest_<appid>.acf`,
    `steamapps/workshop/appworkshop_<appid>.acf`,
    `depotcache/<depot>_<manifest>.manifest` in Valve's format, so the Steam
    client, LinuxGSM and wings eggs read our installs as real ones — and so we
    can take over an existing steamcmd install and update it incrementally.

## Targets: server-side only

| target | verdict | notes |
|---|---|---|
| native CLI (static musl, x86_64 + aarch64) | primary | drop-in steamcmd replacement |
| **library inside a server-side webapp** (axum/actix/whatever) | **primary** | `default-features = false` drops `rayon`, `cap-std` and the CLI, leaving a metadata build for request handlers |
| `wasm32-wasip2` (wasmtime, Spin, wasmCloud) | free if wanted, not a milestone | the trait seam makes it a small `tapline-rt-wasi` crate; built when someone actually asks |
| browser / client-side | **out of scope** | — |

Browser is dropped deliberately and it costs nothing, because it was never
really available: checked live 2026-08-26, neither `api.steampowered.com` nor
`*.steamcontent.com` sends an `access-control-allow-origin` header, browsers have
no raw TCP, and per the CM-first constraint there is no HTTP fallback for depot
keys, manifest request codes or PICS. A browser build would have needed a
same-origin proxy — which is just this library running server-side. So: no
`tapline-rt-web`, no npm package, no `wasm-bindgen` anywhere in the tree.

The IO-trait seam stays regardless, because it pays for itself on the server:
the entire protocol stack tests against an in-memory `Stream` with no network,
and `tokio` is a dependency of `tapline-rt-tokio` alone, so a webapp that already
runs its own reactor is never forced to adopt ours.

### Lightness budget (gated in CI, not aspirational)

- Native CLI, `--release` + `opt-level="z"` + `panic="abort"` + strip:
  **≤ 4 MB** static musl. (steamcmd's tree is ~250 MB unpacked.)
- Metadata-only library build: **no `rayon`, no `cap-std`, no `clap`, no
  `serde`** in the dependency graph — asserted by a `cargo tree` test, not by hope.
- Idle footprint matters for a webapp holding a session open: one CM connection,
  one heartbeat timer, no background threads until a download starts. Target
  **< 5 MB RSS** for an idle logged-in session.
- No `format!`/`panic!` machinery on hot paths in the core crates; errors are
  plain enums, the same rule that keeps proc macros out of the leaves.
- `cargo bloat` sizes recorded per release in the README, with a CI check that
  fails when a budget regresses by >10%.

## Performance design

steamcmd's weakness is a near-serial chunk pipeline. Wins in order of size:

- **Parallel chunk fetch across the CDN pool** — our own connection pool,
  bounded per host (default 8, global 32), work-stealing queue, per-host latency
  and failure scoring, demote and re-pick bad hosts.
- **CPU off the IO threads** — decrypt, decompress and SHA-1 on `rayon`; SHA-NI
  backend for SHA-1. Bounded channels between stages give backpressure and a flat
  memory ceiling: target < 256 MB RSS on a 100 GB depot.
- **Chunk dedupe** — manifests repeat chunk ids across files; fetch once, write to
  every offset.
- **Delta updates** — diff old manifest against new by chunk id; fetch only new
  chunks, copy retained ones out of the existing files locally. A 30 GB server
  update becomes a 200 MB one.
- **Write path** — `fallocate` the file, `pwrite` at chunk offsets from many
  threads, no seek serialisation; `io_uring` behind a feature.
- **Resume** — per-chunk completion journal; a killed download restarts at the
  first unwritten chunk with no full re-verify.
- **Rate-limit safety** — 429/403 → exponential backoff plus host demotion. Fast
  must never look like abuse; a locked account is worse than a slow download.

Target: ≥ 2× steamcmd wall-clock on a multi-GB depot over a fat link, parity on a
thin one, ≥ 10× on a small delta update.

## Security design

Written plainly, because this is the part that bites.

- **Path traversal is the top risk.** Manifest filenames are attacker-influenced
  for Workshop content — anyone can publish a Workshop item. Every path resolves
  against a rooted directory handle (`cap-std`); `..` and absolute paths are
  rejected; files open `O_NOFOLLOW` so an earlier symlink in the same manifest
  cannot redirect a later write outside the install root.
- **Symlinks from the manifest** are validated to resolve inside the root before
  creation. One escaping symlink fails the whole install rather than being
  silently skipped.
- **Chunk integrity before write.** SHA-1 of every decrypted, decompressed chunk
  must equal its chunk id, checked before it reaches the filesystem. This is what
  makes plain-HTTP CDN fetching safe, and what stops a poisoned lancache node —
  relevant because hosting fleets deliberately put a caching proxy in this path.
- **Decompression bounds.** Output size comes from the manifest and is enforced;
  a lying `VZ`/`VSZ` header cannot zip-bomb the process.
- **No code execution, ever.** Depots ship `installscript.vdf` and steamcmd runs
  them. tapline parses and reports them, never executes. Installing a game server
  must not be a remote-code-execution primitive.
- **Credentials.** Password encrypted with Steam's RSA key, never written to disk.
  Only the refresh token persists — OS keyring by default, `0600` file fallback,
  opt-in either way, `zeroize`d in memory. Anonymous login is the default for
  dedicated-server workloads and never touches credentials.
- **Session crypto is ours, so it gets reviewed like it.** The CM handshake,
  CBC padding handling and HMAC verification are the highest-risk code we write:
  constant-time compares for MACs, reject on any padding anomaly without leaking
  which, and test vectors captured from a real session.
- **TLS, where enabled**, is `rustls` with verification on and no escape hatch to
  disable it.
- **Supply chain.** `cargo-deny` + `cargo-audit` in CI, with the dependency floor
  above as the enforced ceiling.

## Library API sketch

```rust
let session = Session::anonymous().await?;                  // or ::login(creds)
let plan = session.app(232250)
    .branch("public")
    .os(Os::Linux)
    .install_dir("/srv/tf2")
    .plan().await?;                                         // what would change, no IO
println!("{} to download, {} reused", plan.download_bytes, plan.reused_bytes);
let mut events = plan.start();                              // Stream<Item = Event>
while let Some(ev) = events.next().await { /* Progress | FileDone | Done */ }

session.workshop(4000).item(2942526891).download_to(dir).await?;
```

`plan()` before `start()` matters for warden: a scheduler wants the byte cost
before committing disk. Errors are concrete enums, no panics, no `unwrap` in
library code, no logging side effects (`tracing` only).

## CLI surface

Compat mode — steamcmd's grammar verbatim, so existing scripts work unchanged:

```
tapline +login anonymous +force_install_dir /srv/tf2 +app_update 232250 validate +quit
tapline +login anonymous +workshop_download_item 4000 2942526891 +quit
```

Native mode:

```
tapline app download 232250 --dir /srv/tf2 --branch public --validate --json
tapline app plan 232250 --dir /srv/tf2 --json      # dry run, byte cost
tapline workshop download 4000 2942526891 --dir ...
tapline login --qr | tapline logout | tapline whoami
```

`--json` emits newline-delimited events. Static musl binary, no runtime deps, no
32-bit legacy.

## Milestones (each a reviewable commit range)

The gate column is the milestone's test, written in the same commits as the code.
No milestone closes on "it ran once by hand".

Status as of 2026-08-26: **M0–M5 done**, each verified against live Steam where
the gate says so. 181 offline tests; the live tests are `#[ignore]`d.

| # | milestone | gate |
|---|---|---|
| M0 ✅ | repo, workspace, licence, CI, `cargo-deny`, dev profile | CI green on empty crates |
| M1 ✅ | leaves: `wire`, `ids`, `crypto`, `vdf`, `event`, `io` traits + fuzz targets | codec round-trips a real captured message; ACF round-trips byte-identical |
| M2 ✅ | `xtask` codegen → committed `tapline-proto`; `--timings` check | every Steam `.proto` compiles; no `prost`/`protoc` in the consumer tree |
| M3 ✅ | `tapline-rt-tokio` + `tapline-net`: CM handshake (TCP + WS), mux, anon logon | `tapline whoami` on a real CM; captured-session vectors pass against an in-memory `Stream` with no network |
| M4 ✅ | `tapline-pics` → `app info 232250 --json` | depots/branches match SteamDB |
| M5 ✅ | depot keys + `tapline-manifest` → `app plan` | our manifest parse is byte-identical to steamcmd's `depotcache/*.manifest` |
| M6 | `tapline-cdn` + `tapline-fs`: first full install | `diff -r` vs a steamcmd install of 232250 is empty |
| M7 | `tapline-state`, delta update, resume, `validate`/repair | update across two builds touches only changed files |
| M8 | Workshop: SteamPipe UGC + legacy UFS | `diff -r` vs steamcmd `workshop_download_item` |
| M9 | credentialed login (password/QR/Guard), token store, owned-app download | login once, re-login from stored token |
| M10 | perf pass, steamcmd grammar, musl release, size budgets in CI, docs, publish | benchmark + size table in README; `cargo tree` test on the metadata build |

## Testing as you go, and disk hygiene

Tests land **in the same commit as the code they cover**, not in a later "add
tests" pass. Every milestone in the table above ships its own test layer, and the
gate column is what the tests assert. Rules:

- Each leaf crate gets unit tests + a `cargo-fuzz` target in the milestone that
  creates it (M1, M2), before anything depends on it. Parsers are the whole
  attack surface; they do not get to exist untested.
- `cargo test --workspace` and `cargo clippy --workspace --all-targets -D warnings`
  run per commit in CI (kirie's `ci.yml` pattern). Network-touching tests are
  `#[ignore]`d so CI stays offline and deterministic.
- Fixtures are recorded once and committed **small**: one captured CM handshake,
  a handful of manifests, a few dozen chunks. Total test-data budget **≤ 20 MB**
  in-repo — anything bigger is generated at test time and deleted, never committed.

Disk hygiene, because this tool's entire job is writing tens of GB:

- **Never download into `/tmp`.** `/tmp` on this box is tmpfs — a 30 GB depot
  test would be 30 GB of RAM. Integration tests write to a disk-backed scratch
  root, `TAPLINE_TEST_DIR` (default `~/.cache/tapline-test`), asserted at test
  start to not be a tmpfs mount.
- Every integration test wraps its install root in a `Drop` guard that removes it,
  including on panic. Differential steamcmd runs delete **both** trees after the
  `diff -r`, pass or fail — the diff output is what's kept, not the 30 GB.
- Big-app tests (740, 896660) are `#[ignore]`d and opt-in; the default suite uses
  232250 (~1 GB) and single Workshop items (MB-scale).
- The suite refuses to start if free space on the scratch filesystem is below
  2× the planned download size — `plan()` already returns that number, so it is a
  cheap precondition rather than an ENOSPC halfway through.
- `cargo clean` on `target/` between perf runs is scripted, and CI runners already
  have the hourly `_work` sweep from the runner-disk fix.

## Deviations from this plan, and why

- **The CM transport is WebSocket-only. There is no TCP path to write.** Measured
  2026-08-26: `ISteamDirectory/GetCMListForConnect` returns 58 servers — 52
  `websockets`, 6 `netfilter`, and **zero** TCP. Ports 443 and 27018 both answer
  with TLS 1.3 and a valid certificate (`CN=cmp1-iad1.steamserver.net`,
  verification OK); the legacy raw-TCP port 27017 is filtered. So the plan's
  "TCP path needs no TLS, and the WebSocket path skips channel encryption in
  exchange" is half right: there is only the second half.

  Consequences, all of them simplifications:
  * No `ChannelEncryptRequest`/`Response` handshake to implement. TLS provides
    confidentiality and authenticates the server through ordinary PKI.
  * **Valve's universe RSA public key is not needed at all.** It existed to
    encrypt the session key during that handshake. Password encryption at M9 uses
    a per-account key from `Authentication.GetPasswordRSAPublicKey`, so nothing in
    tapline ever needs a hardcoded Valve constant — which also removes the one
    constant that could not have been verified from first principles.
  * `tapline-crypto`'s message encryption is still needed, but for depot content:
    manifest filename decryption and chunk decryption, not the channel.
  * We owe a WebSocket client (RFC 6455 framing) instead of the CM handshake.

- **`tapline-lzma` and `tapline-zstd` moved from M1 to M6.** They were planned as
  leaves to build early, but the `VZ` and `VSZ` container headers around the
  compressed payload are undocumented, and the only way to know their layout is
  to read a real chunk. A real chunk needs a depot key and a manifest request
  code, which need a CM session — so the earliest honest moment to write them is
  M6, alongside the CDN client that fetches the sample. Writing them now would
  mean inventing a header and testing it against my own invention, which would
  pass every test and fail against Valve.

## Verification

- **Differential against steamcmd** is the primary gate at M6, M7, M8: install
  the same app/branch with both, then `diff -r` plus a mode/symlink compare. Zero
  diff or the milestone isn't done. steamcmd is a test oracle only — never a
  runtime dependency. Anonymous test apps: 232250 (TF2 DS, small), 740 (CS2 DS),
  896660 (Valheim DS), 4000 (GMod, Workshop).
- **Captured-session vectors**: one real CM handshake and logon recorded and
  committed as fixtures, so `tapline-net` has non-network regression tests for
  the crypto and framing.
- **Offline unit tests**: manifest and chunk fixtures committed (small,
  licence-free depots only) so CI needs no Steam access. Network tests are
  `#[ignore]`d and run manually.
- **Fuzzing** (`cargo-fuzz`): wire codec, manifest parser, VDF/ACF parser, and the
  `VZ`/`VSZ` decoders — every one of them eats untrusted bytes.
- **Path-traversal corpus**: hand-built manifests with `../`, absolute paths,
  escaping symlinks, and `..` hidden behind a symlinked directory. Each must fail
  the install, with a test asserting nothing was written outside the root.
- **Benchmarks**: wall-clock and peak RSS vs steamcmd on the same host and link,
  recorded in the README; delta-update benchmark across two known builds.
- **Integration into warden**: replace the planned `steamcmd` shell-out in
  `warden-games`/`warden-gamepack` with the library as the real-world smoke test.

## Open decisions (answer before M0)

1. **Name** — `tapline` (recommended), `standpipe`, or `steamduct`. All three
   verified free on crates.io; `spigot`, `sluice`, `cistern` are taken.
2. **Licence** — MPL-2.0 (recommended: file-level copyleft, still linkable from
   warden's private tree) vs Apache-2.0/MIT vs AGPL. AGPL would poison warden.
3. **Repo visibility** — public from M0 (invites contributors, and Valve's
   attention) vs private until M6.
