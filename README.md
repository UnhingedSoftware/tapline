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
tapline app download 896660 --dir /srv/valheim --concurrency 64
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
| Workshop item 3790437566 | **byte-identical to `+workshop_download_item`** |

The comparison is a test, not a one-off: `cargo test -p tapline --test
differential` walks both trees and compares every file's name, size, contents
and mode.

Modes are in that list because they were not always. Installing Garry's Mod
Dedicated Server (app 4020, 6.8 GB, 2,329 files) with both tools produced trees
whose contents matched on every single file and whose **permissions disagreed on
2,291 of them**: steamcmd sets `0o755` on everything it writes, including text,
models and sounds, while tapline was applying the manifest's executable flag.
The differential test had compared sizes and contents and never modes, so it
reported parity it had not checked — which is the more useful finding of the
two. See `FileModes` for which behaviour you get and why the blunt one is the
default.

## Is it fast?

Same machine, same link, both starting cold:

| app | downloaded | steamcmd | tapline |
|---|---|---|---|
| Valheim DS (896660) | 1.47 GB | 19.0 s | **8.3 s** |
| Garry's Mod DS (4020) | 3.54 GB | 29.5 s | **18.3 s** |

GMod is the more interesting measurement because tapline lost it first, at
41.3 s against 29.5 s. Instrumenting the stages, aggregated across the sixteen
concurrent slots:

```
fetch=331s  decode=93s  write=17s  fsync=13.5s
```

Only one of those was not overlapping with anything. `fsync` was being awaited
inline in the loop that dispatches chunk fetches, so for 13.5 seconds of a
41-second install nothing new was started and all sixteen slots drained to idle.
Moving finalisation onto its own blocking tasks gave back 11.8 s, which brought
GMod level with steamcmd at 29.5 s.

Level is where it stayed until the default concurrency was questioned. It had
been 16, chosen as "deliberately modest" with nothing measured behind it, and
documented with a claim that both tools were saturating the link. Sweeping it
says otherwise:

| concurrency | wall clock | throughput |
|---|---|---|
| 16 | 29.5 s | 120 MB/s |
| 32 | 21.1 s | 168 MB/s |
| **64** | **18.3 s** | 184 MB/s |
| 128 | 20.7 s | 163 MB/s |
| 256 | 21.5 s | 157 MB/s |

There was no link ceiling at 120 MB/s; 16 was leaving a third of the throughput
unused, and the sentence explaining that it could not go faster was reasoning
from two coincidentally similar numbers. The default is 64, where the curve
turns over.

## Several downloads at once

A process that installs more than one app shares one budget of chunks in flight
and one connection pool, through `Shared`. It is the default; concurrent
`install()` calls need nothing.

The reason is the curve above. Throughput turns over past 64, so N downloads
each taking a full 64 is slower than N splitting one 64. Three concurrent
Valheim installs, 4.40 GB total:

| total budget | wall clock | throughput | spread between finishes |
|---|---|---|---|
| **64 (shared, the default)** | **18.3–21.3 s** | **197–230 MB/s** | 0.9 s |
| 96 | 18.5–19.3 s | 218–227 MB/s | — |
| 128 | 22.9 s | 184 MB/s | — |
| 192 (a full budget each) | 24.7 s | 170 MB/s | 6.7 s |

Faster and fairer: the three finish within a second of each other rather than
nearly seven seconds apart. 64 and 96 are inside each other's run-to-run
variance — 64 produced both 197 and 230 MB/s — so the default did not move; the
gain is from sharing, not from a different number.

Worth noting against the single-download table above: three downloads sharing 64
beat *one* download at 64, which peaks near 184 MB/s. One download cannot keep
64 requests busy, because it stalls on its own per-file and per-depot ordering,
and another download's chunks fill those gaps.

## Where the ceiling actually is

Around **184 MB/s**, and it does not appear to be ours. Everything below was
measured on a 2.5 Gb link behind a 3 Gb connection, with a disk that writes at
1.9 GB/s — none of which is the constraint at 184 MB/s.

| change | result |
|---|---|
| more concurrency (128, 256) | slower: 163, 157 MB/s |
| more CDN hosts (40, 60) | much slower: 60, 59 MB/s |
| fewer CDN hosts (4, 8, 12) | no change within noise |
| sticky host affinity per slot | **40–55% slower** |

The sticky experiment is the informative failure. Chunks round-robin across the
host list, so a host's pooled connection can go cold between visits and pay a
fresh TLS handshake; pinning each in-flight slot to one host should have kept
every socket warm. It lost every single run, at 20 hosts and at 64. The reason
is the thing round-robin was quietly doing all along: hosts are not equally
fast, and dynamic assignment lets the quick ones absorb more work while a
pinned slot waits on whichever host it drew. That also explains the wide-host
result without any appeal to handshakes — Steam returns hosts best-first, so
asking for 40 or 60 means spending an equal share of requests on the worst ones.

What is left is what Steam serves one client from one cell. Pulling ~100 GB
inside an hour got that number cut roughly in half for a while, which is a
volume limit rather than a connection-count one, and is worth knowing before
reading any benchmark taken on a hot cache.

An update that finds nothing changed re-checks 6.8 GB across 2,329 files in
**2.1 s** and downloads 0 bytes.

The number worth quoting is the one before the concurrency work: the Valheim
install took **238 seconds** when chunks were fetched one at a time. Most of
that was not the network. `#[tokio::test]` defaults to a current-thread runtime,
so sixteen "concurrent" tasks were sixteen tasks taking turns on one thread,
each blocking the others while it decompressed a megabyte of LZMA. Moving the
decode to `spawn_blocking` and using a real runtime took it to 18 s in the test
harness and 11.7 s in a release build.

## Ready to run, not merely correct

A dedicated server that installs perfectly and will not start is not installed.
Two things that byte-comparison cannot see turned out to matter:

- **Permissions.** `InstallOptions::file_modes` defaults to `FileModes::SteamCmd`
  — `0o755` on everything, which is what steamcmd does. It is not the tidier
  choice; it is the compatible one. LinuxGSM, wings eggs and a decade of Docker
  images were built against trees that look like this, and a depot whose manifest
  forgets the executable flag on a start script still yields a runnable server
  under steamcmd. `FileModes::Manifest` gives the strict alternative for callers
  who want what the depot actually describes.
- **File descriptors.** An install held one open descriptor per file for the
  whole depot. Under the default 1,024 limit, Garry's Mod's 2,329 files failed
  with `Too many open files` — while succeeding on the interactive shell that
  first tested it, which happened to have the limit raised. Files are now synced
  and closed as their own last chunk lands, with a hard ceiling of 64 open at
  once regardless of how a depot distributes chunks.

Verified end to end: a GMod server installed by tapline and one installed by
steamcmd were both started with the same command line, both reached `VAC secure
mode is activated`, and both answered an A2S_INFO query as
`gm_construct / Sandbox / 0-4 players`.

## What it does not do

tapline downloads content the signed-in account is entitled to, exactly as the
real client does. Depot keys come from Steam, and Steam only hands them out for
owned or anonymous-accessible depots. There is no key dumping, no `keys.txt`, no
unowned-depot access, no DRM removal and no `.lua` manifest sideloading, and
those are not oversights to be filed as feature requests.

It also never executes a depot's `installscript.vdf`. steamcmd does; tapline
parses it and reports it. Installing a game server should not be a
remote-code-execution primitive.

## In a container

No steamcmd, no Steam client, no 32-bit runtime, no CA bundle: tapline speaks the
CM protocol itself and `rustls` carries Mozilla's roots compiled in. The
`Dockerfile` builds a **1.6 MB image on `scratch`** — one static binary and
nothing else, not even a shell.

```sh
docker build -t tapline .
docker run --rm -v /srv/gmod:/data tapline \
  +login anonymous +force_install_dir /data +app_update 4020 +quit
```

Verified by running a real Workshop download inside it: 348 files streamed in
0.60 s, with nothing in the image but the binary.

## From JavaScript

There is a C ABI and a TypeScript package on top of it, so Deno, Bun and Node
can install Steam content without spawning anything:

```ts
import { install, plan } from "tapline";

const { downloadBytes } = await plan({ app: 4020, dir: "/srv/gmod" });

await install({
  app: 4020,
  dir: "/srv/gmod",
  onProgress: (p) => console.log(`${p.percent.toFixed(1)}%`),
});
```

The same object is awaitable, async-iterable, cancellable and callback-able.
No function pointer crosses the FFI boundary — jobs push events into a queue
and the binding pulls them — because a download thread calling into a torn-down
isolate takes the host process with it, and all three runtimes share that
failure mode. See `bindings/js/README.md` for the reasoning and
`crates/tapline-ffi/include/tapline.h` for the C API.

Verified on Deno 2.9.5, Bun 1.3.14 and Node 26.7.0: the same test file, 10/10
on each, including real downloads.

## Extensions

Downloading a file is rarely the last step. An [`Extension`] is handed each file
as it lands and may act on it, so tapline does not grow a special case per game:

```rust
session.register(Arc::new(tapline_gmad::Extract::new()));
session.register(Arc::new(tapline_gmad::ToZip::new()));
```

`tapline-gmad` ships two, for Garry's Mod addons — which arrive as a single
`.gma` container. `gmad` unpacks it, `gmad-zip` converts it to a zip. Both are
selectable by name from the CLI and the C ABI:

```sh
tapline workshop download 4000 104691717 --dir /srv/gmod/garrysmod/addons \
  --flat --extensions gmad,gmad-zip
```

Names rather than function pointers, deliberately. An extension is Rust the
operator compiled in; nothing in a manifest, a Workshop item or a CDN response
can introduce one, which is the same line tapline draws by parsing
`installscript.vdf` and refusing to execute it.

Extensions run on a blocking thread after a file is synced, never on the loop
that dispatches chunk fetches — the [`fsync` lesson](#is-it-fast) applies more
strongly to unpacking an archive than it did to a sync.

The GMAD format was read off a real addon rather than a description of it. On
PAC3 (348 files, 8.69 MB): unpack 19 ms, convert to a stored zip 10 ms, convert
to a deflated zip 39 ms. Deflate runs across every core — it was 175 ms on one,
for byte-identical output — and the result is checked against `unzip -t`,
because an archive only this crate can read is not a zip.

### Streaming

A format whose contents arrive in a knowable order can be consumed as it
downloads, without the archive touching the disk at all. GMAD is one, and the
target is pluggable:

```sh
tapline workshop download 4000 104691717 --dir addons --stream            # unpack
tapline workshop download 4000 104691717 --dir addons --stream zip        # a .zip
tapline workshop download 4000 104691717 --dir addons --stream zip-stored # no deflate
```

Measured on PAC3 (348 files, 8.4 MB archive):

| | wall | read back | on disk |
|---|---|---|---|
| download, then unpack | 2.00 s | 13.3 MB | 16.6 MB |
| `--stream` | 1.78 s | 0.01 MB | 8.3 MB |
| download, then convert to zip | 2.00 s | 16.6 MB | 12.3 MB |
| `--stream zip` | **1.94 s** | **0.01 MB** | **4.0 MB** |

Chunks are still fetched in parallel — that is where the throughput is — and
reordered through a bounded window, so peak memory is the window rather than the
file. It does not save memory: streaming measured 30.9 MB resident against 26.8,
because that window is a fixed cost the archive size is not. It saves disk, and
that saving scales with the addon.

Streaming to a zip was 2.29 s before it batched: compressing each entry the
moment it completed gave up the parallel deflate to save the disk. Queueing
completed entries and deflating them together gets both.

Adding a target means writing an `EntrySink` — the byte-boundary state machine
is shared, and unpacking and zipping differ only in what they do with an entry.

### The pipeline

A typed chain, for when the download is the first step rather than the last:

```rust
tapline_pipe::workshop(4000, 104_691_717)
    .gma()                       // bytes -> entries
    .only("lua/**")              // optional
    .zip("/srv/out.zip")         // where it goes; ends the chain
    .run().await?;
```

The types change as you chain, and both mistakes are compile errors rather than
run-time ones. A `Source` has no `.zip()`, because there is nothing to zip until
the bytes have been interpreted. And choosing a destination **ends** the chain —
there is no `.zip(..).dir(..)`.

That is deliberate. A stream has a direction; writing one download to two places
is a fan-out, which is a different thing with different costs — a second sink
that buffers multiplies what the first holds. `tapline_gmad::Fanout` is there
for anyone who wants that explicitly, rather than by accident.

`.gma()` is one `Decoder`. The sinks, the filter and the pipeline are written
against `ArchiveEntry` rather than any container, so a second format is a
decoder and nothing else changes. Whether a format can be streamed at all is a
property of the format: GMAD works because its index comes first and its
contents follow in index order.

No session appears in that. One is taken from a process-wide pool and given
back, so concurrent chains get different sessions and never wait on each other,
while still sharing one chunk budget and one connection pool. `run_with(&mut
session)` is there for anyone who wants to own it.

A `Session` is `&mut` because one CM connection carries one request at a time:
it allocates a job id, writes the frame, and reads until that reply arrives.
That is what the code does, and it is not something a caller should have to
think about — hence the pool. Measured: a second download in the same process
takes 649 ms against the first's 1794 ms, because it skips the logon entirely.
Idle sessions are heartbeated by a keeper task, since Steam drops a quiet
session without saying so.

The chain is sugar over a `Pipeline` value, which is what actually travels —
the chain cannot cross a C ABI, so the bindings build the same value and send
its text form:

```text
decode gma
only lua/**
zip /srv/out.zip
```

Line-based rather than JSON because tapline writes JSON and does not parse it,
and a parser here would be one more thing to get wrong on input from outside
the process. A path runs to the end of its line, so there is no quoting.

## Shape

Twelve crates. Everything above the leaves is IO-free and reaches the network
through four traits, so the whole protocol stack is tested against recorded
bytes — and `tokio` is a dependency of one crate, not of anything that links the
library.

```
wire      protobuf codec          crypto   AES/RSA/SHA-1, Steam's compositions
ids       SteamID, EResult        vdf      KeyValues and appmanifest files
chunk     VZ, VSZ, ZIP           fs       path validation
ext       the extension seam     gmad     Garry's Mod addons
pipe      the typed chain
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
cargo test --workspace                # 447 tests, no network
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
