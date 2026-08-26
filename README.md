# tapline

A Rust implementation of Steam's content delivery: CM protocol, PICS, depot
manifests, the SteamPipe CDN chunk pipeline and Workshop/UGC. It replaces
`steamcmd` for installing dedicated servers and Workshop content.

It is two things at once:

- **a library** you link into a Rust service, with a real API instead of a
  subprocess whose stdout you parse;
- **a single binary** that understands `steamcmd`'s own command grammar, so
  existing scripts keep working.

```sh
# steamcmd's grammar, unchanged
tapline +login anonymous +force_install_dir /srv/valheim +app_update 896660 +quit

# or the native one, with machine-readable output
tapline app plan 896660 --dir /srv/valheim --json
tapline app download 896660 --dir /srv/valheim
tapline workshop download 4000 3790437566 --dir /srv/gmod
```

```rust
let mut session = Session::anonymous().await?;

// What would it cost? No content is fetched to answer this.
let plan = session.plan(AppId(896_660), &options).await?;
println!("{} to download, {} reused", plan.download_bytes, plan.reused_bytes);

session.install(AppId(896_660), &options).await?;
```

## Is it right?

The question that matters for a tool like this is not whether it runs but
whether it agrees with the thing it replaces. Measured on 2026-08-26, installing
Valheim Dedicated Server (app 896660, 1.7 GB, 793 files) with both tools on the
same machine and link:

| | result |
|---|---|
| Files installed | **793, byte-for-byte identical to steamcmd** |
| Differences of any kind | none |
| Workshop item 3790437566 | **byte-identical to `+workshop_download_item`** |

The comparison is a test, not a one-off: `cargo test -p tapline --test
differential` walks both trees and compares every file's contents.

## Is it fast?

Same install, same machine, same link, both starting cold:

| | wall clock |
|---|---|
| steamcmd | 19.0 s |
| tapline | **11.7 s** |

That is 1.47 GB of downloaded content at roughly 125 MB/s, which is close enough
to a gigabit link's ceiling that **both tools are probably limited by the network
rather than by themselves**. The honest reading is "at least as fast as
steamcmd, on a link where neither can go faster" — not a 1.6× claim that would
generalise.

The number worth quoting is the one before the concurrency work: the same
install took **238 seconds** when chunks were fetched one at a time. Most of
that was not the network. `#[tokio::test]` defaults to a current-thread runtime,
so sixteen "concurrent" tasks were sixteen tasks taking turns on one thread,
each blocking the others while it decompressed a megabyte of LZMA. Moving the
decode to `spawn_blocking` and using a real runtime took it to 18 s in the test
harness and 11.7 s in a release build.

An update that finds nothing changed downloads **0 bytes** and rewrites **0
files**.

## What it does not do

tapline downloads content the signed-in account is entitled to, exactly as the
real client does. Depot keys come from Steam, and Steam only hands them out for
owned or anonymous-accessible depots. There is no key dumping, no `keys.txt`, no
unowned-depot access, no DRM removal and no `.lua` manifest sideloading, and
those are not oversights to be filed as feature requests.

It also never executes a depot's `installscript.vdf`. steamcmd does; tapline
parses it and reports it. Installing a game server should not be a
remote-code-execution primitive.

## Shape

Twelve crates. Everything above the leaves is IO-free and reaches the network
through four traits, so the whole protocol stack is tested against recorded
bytes — and `tokio` is a dependency of one crate, not of anything that links the
library.

```
wire      protobuf codec          crypto   AES/RSA/SHA-1, Steam's compositions
ids       SteamID, EResult        vdf      KeyValues and appmanifest files
chunk     VZ (LZMA), VSZ (zstd)   fs       path validation
io        the IO traits           event    progress vocabulary
proto     404 generated messages  net      CM framing, batches, job correlation
pics      depots and branches     manifest the manifest format
cdn       host pool, chunks       state    the install record
auth      login, token store      rt-tokio the only crate that opens a socket
tapline   the facade              cli      the binary
```

The protobuf types are generated ahead of time by `cargo xtask gen-proto` and
committed, so no consumer inherits `prost`, `protoc`, or a build script. Our
encoding of a real `ContentManifestPayload` is byte-identical to what Google's
`protoc` produces — a differential test, not a round-trip through our own
decoder.

## Safety

Workshop manifests name the paths tapline creates, and anyone can publish a
Workshop item. That is the input the path validation exists for: absolute paths,
`..` anywhere, drive prefixes, NUL bytes and symlinks whose targets escape are
all refused, and refused as a fatal error rather than a skipped file. 12.6M fuzz
executions assert that anything accepted resolves inside the install root.

Every chunk is verified against the SHA-1 the manifest named before it reaches
the disk. Hosting fleets deliberately put caching proxies in this path, and a
proxy returning the wrong object is an ordinary operational failure — one that
would otherwise be written to disk as game content.

`unsafe` is forbidden workspace-wide. `unwrap`, `expect`, `panic` and slice
indexing are denied in library code, because a panic in tapline is a panic in
the process that linked it.

## Building

```sh
cargo build --release                 # needs no extra toolchain
cargo test --workspace                # 301 tests, no network
cargo test --workspace -- --ignored   # the live tests, against real Steam
```

The live tests log on **anonymously** and download real content. They are
`#[ignore]`d so CI stays offline and deterministic.

A static musl build additionally needs a musl-targeting C compiler, because
`rustls`'s `ring` backend compiles C for the target:

```sh
sudo apt-get install musl-tools
cargo build --release --target x86_64-unknown-linux-musl
```

The glibc binary is 2.7 MB and links only `libc`, `libm` and `libgcc_s`.

## Licence

MPL-2.0.
