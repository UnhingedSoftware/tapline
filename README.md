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

# smaller footprint, ~25 MB instead of ~40, at 25-39% of the speed
tapline app download 896660 --dir /srv/valheim --concurrency 10
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

| app | downloaded | steamcmd | tapline | wire |
|---|---|---|---|---|
| Valheim DS (896660) | 1.47 GB | 19.0 s | **8.2 s** | 1.43 Gb/s |
| Garry's Mod DS (4020) | 3.54 GB | 29.5 s | **19.1 s** | 1.48 Gb/s |

"Downloaded" is compressed bytes over the wire, which is the only unit worth
quoting for throughput. Earlier versions of this file quoted installed bytes per
second — 184 MB/s and similar — which is the same runs inflated by a 1.2x
decompression ratio and made the link look closer to saturated than it was.

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

There was no link ceiling at 120 MB/s; 16 was leaving a third of the throughput
unused, and the sentence explaining that it could not go faster was reasoning
from two coincidentally similar numbers.

That table used to have rows for 128 and 256 showing the curve turning over
gently. They were wrong: a download draws on the process-wide budget as well as
its own limit, and the budget was left at the default of 64, so both rows were
really 64 measured twice more. Raising a limit that another limit already caps
is an easy experiment to run and a hard one to notice, and it produced two wrong
tables here before it was found. Measured properly, 128 does not turn over
gently — it collapses to 29.3 s, because more requests go out than the link and
the CDN will carry and they queue until they time out.

The default is **48**: the fewest chunks in flight that reach full speed, and
not one more. Past that the extra requests cost more than they carry — 64 is
slower than 48 on both workloads *and* costs 15–18 MB more — so this is not a
case of trading all the memory you have for all the speed there is. See
[Memory](#memory).

## Finding Workshop items

tapline could always download an item once you knew its number; getting that
number meant a browser. `PublishedFile.QueryFiles` is the same search the
Workshop website runs, over the CM session already open — no WebAPI key and no
login:

```sh
tapline workshop search 4000 --text stargate --sort text --limit 5
tapline workshop search 4000 --tag Weapon --sort subscribed
tapline workshop info 104691717 3790437566
```

```
  3037205213     2.0 GB      4341 subs  Stargate (CAP)
   177663377    61.0 MB    434585 subs  ttt_stargate
   133391119    63.1 MB     70138 subs  rp_stargate
5 of 885 matches; next page: --cursor AoIIQ0gRxHvXp+MF
```

A result carries everything a download needs, so search feeds download with no
second lookup — in Rust, from the CLI, and from JavaScript:

```rust
let page = session.browse_workshop(&BrowseQuery { app: AppId(4000), ..default }).await?;
session.download_workshop_item(&page.items[0].item, &options).await?;
```

Paging is Steam's cursor rather than a page number, because offsets start
repeating items past about a thousand results. Pass `nextCursor` back as
`--cursor` and stop when it is empty.

Two queries are refused rather than sent, because Steam answers both with
something that looks right: a search with no app id (an unusable mixture of
every app's Workshop) and a text sort with no text (an arbitrary order that
looks like a ranking).

## Which Steam account this machine uses

`tapline whoami` reports the local Steam client's accounts and library paths
alongside its own session, and `tapline login` names the account the client last
used instead of making you remember it.

It reads the **identity**, not the session. A modern Steam client keeps its
refresh token in its own encrypted store: `loginusers.vdf` has the SteamID, the
account name and the auto-login flags and no token, `config.vdf` has no auth key
in 49 KB, and there are no `ssfn` files. Getting at a live session would mean
driving the client's undocumented localhost IPC or linking Valve's SDK — the
first lifts a credential out of another running process, the second is the one
thing this project does not do at any layer. So tapline signs in once itself and
keeps its own token.

## Several downloads at once

A process that installs more than one app shares one budget of chunks in flight
and one connection pool, through `Shared`. It is the default; concurrent
`install()` calls need nothing.

The reason is the curve above. Throughput falls off past 64, so N downloads each
taking a full 64 is slower than N splitting one 64. Three concurrent Valheim
installs, 4.40 GB total:

| total budget | wall clock | throughput | spread between finishes |
|---|---|---|---|
| **64 (shared)** | **18.3–21.3 s** | **197–230 MB/s** | 0.9 s |
| 96 | 18.5–19.3 s | 218–227 MB/s | — |
| 128 | 22.9 s | 184 MB/s | — |
| 192 (a full budget each) | 24.7 s | 170 MB/s | 6.7 s |

Faster and fairer: the three finish within a second of each other rather than
nearly seven seconds apart. 64 and 96 are inside each other's run-to-run
variance — 64 produced both 197 and 230 MB/s — so the gain is from sharing, not
from picking a different number.

The shipped budget is 48, the same as one download's default, because a budget
is chunks in flight and chunks in flight are what memory is made of. A process
doing several installs at once that has memory to spare should raise it — the
sharing is worth more than the number:

```rust
let shared = Shared::new(64);                            // ~83 MB, three installs
let a = Session::anonymous_shared(shared.clone()).await?;
let b = Session::anonymous_shared(shared).await?;
```

Worth noting against the single-download table above: three downloads sharing 64
beat *one* download at 64, which peaks near 184 MB/s. One download cannot keep
64 requests busy, because it stalls on its own per-file and per-depot ordering,
and another download's chunks fill those gaps.

## Where the ceiling actually is

A single install tops out around **1.45 Gb/s**, and this machine can pull
**2.03 Gb/s** from Steam in total — measured by running 1, 2, 3 and 4 installs at
once (1.39, 1.73, 1.95, 2.03), which flattens out well below the 2.5 Gb link. So
one install gets about 70% of what is there, and the missing 30% needs a second
download stream to reach, not a bigger number anywhere.

Where a request's time goes, measured over 2,406 of them: **24.9 ms waiting for
headers, 72.6 ms reading the body, 97.5 ms total**. Multiply that out and the
average number in flight is 27, not the 48 the budget allows — a slot is held for
the whole chunk, so it is not fetching while its chunk decodes. Releasing the
slot the moment the bytes are down raises the average to 31 and changes
throughput not at all: each request simply gets slower. That is what a per-stream
cap looks like from the inside, and it is why the list below is mostly things
that are *not* the constraint.

| change | result |
|---|---|
| blocking pool 4 → 16 threads | **1.02 → 1.40 Gb/s** — chunk decode was starved |
| more chunks in flight (96, 192) | slower: 1.28, 1.21 Gb/s |
| more CDN hosts (40, 60, 80) | slower: 1.02, 1.04, 1.04 Gb/s despite 89 IPs |
| fewer CDN hosts (4, 8, 12) | no change within noise |
| bigger idle connection pool (32/256, 64/512) | no change beyond noise |
| sticky host affinity per slot | **40–55% slower** |
| sharding one install across 2, 4, 8 connection pools | no change: 1.28-1.46 Gb/s |
| releasing the chunk slot after the fetch instead of after the write | no change |
| a second, third, fourth concurrent install | **1.39 → 1.73 → 1.95 → 2.03 Gb/s** |

The decode row is the one that mattered, and it was a self-inflicted wound: the
pool was capped at four threads in an earlier commit here, on a measurement
taken while `--concurrency` was itself silently capped at 8. Four decode threads
really were enough for eight chunks in flight. At 48 they cost 40% of the link.

CPU sits at 3.6–4.1 cores of 32, the disk writes at 1.9 GB/s, per-file setup is
0.03 s of an 8 s install, and 96% of requests reuse a pooled connection (2,313
against 93 fresh). None of those is close to being the constraint.

What is left, and is not yet explained: several installs beat one, while every
way of making a single install *look* like several — more slots, more pools,
more hosts, earlier slot release — does not. The next thing worth testing is
sticky host assignment per slot, which was tried once and lost badly; that
measurement was taken while chunk decode was capped at four threads, so it
deserves a rerun for the same reason the concurrency tables did.

The sticky experiment is the informative failure. Chunks round-robin across the
host list, so a host's pooled connection can go cold between visits and pay a
fresh TLS handshake; pinning each in-flight slot to one host should have kept
every socket warm. It lost every single run, at 20 hosts and at 64. The reason
is the thing round-robin was quietly doing all along: hosts are not equally
fast, and dynamic assignment lets the quick ones absorb more work while a
pinned slot waits on whichever host it drew. That also explains the wide-host
result without any appeal to handshakes — Steam returns hosts best-first, so
asking for 40 or 60 means spending an equal share of requests on the worst ones.

Steam does apply a volume limit: pulling ~100 GB inside an hour got throughput
cut roughly in half for a while. That is worth knowing before reading any
benchmark taken on a hot cache, and it is not the same thing as a per-client
rate cap — this text used to claim the latter on the strength of the former.

An update that finds nothing changed re-checks 6.8 GB across 2,329 files in
**2.1 s** and downloads 0 bytes.

The number worth quoting is the one before the concurrency work: the Valheim
install took **238 seconds** when chunks were fetched one at a time. Most of
that was not the network. `#[tokio::test]` defaults to a current-thread runtime,
so sixteen "concurrent" tasks were sixteen tasks taking turns on one thread,
each blocking the others while it decompressed a megabyte of LZMA. Moving the
decode to `spawn_blocking` and using a real runtime took it to 18 s in the test
harness and 11.7 s in a release build.

## Memory

A download holds about **65 MB**, whatever it is downloading — the memory a
default costs is predictable from one number, and that number is picked to reach
full speed and stop there. Measured on this machine, release build, defaults:

| | peak RSS | wall |
|---|---|---|
| idle logged-in session | 7.8 MB | — |
| `app info` (metadata only) | 7.6 MB | 1.0 s |
| Workshop item, 8.4 MB, streamed to a zip | 29.8 MB | 1.9 s |
| Valheim dedicated server, 1.5 GB | 73.1 MB | 8.2 s |
| Garry's Mod dedicated server, 6.8 GB | 71.4 MB | 19.1 s |

The last two rows are the point: 6.8 GB costs what 1.5 GB does, and the bigger
install costs slightly *less*. Nothing in a download scales with the content.
Chunks are written straight to their offset in the target file as they arrive
and pass their hash — there is no per-file buffer to fill — and files are closed
as they finish. The only thing that grows is the number of chunks in flight, and
that is a fixed budget. Peak RSS is close to `15 + 1.1 × concurrency` MB from 4
chunks in flight up to 128, which is what makes a ceiling worth promising rather
than something that happened to hold on the apps that got tested.

Two things produce that number.

**The concurrency default is where memory and speed argue.** A slot costs about
1.1 MB and it is a floor, not waste: a chunk's plaintext has to be complete
before its SHA-1 can be checked, and nothing is written before that check. So
buying speed means buying memory — up to the point where it stops buying speed:

| in flight | Valheim 1.5 GB | GMod 6.8 GB | peak RSS |
|---|---|---|---|
| 16 | 11.5 s | 29.6 s | 40 MB |
| 32 | 9.3 s | 21.4 s | 60 MB |
| **48 (default)** | **8.5 s** | **19.3 s** | **73 MB** |
| 64 | 8.0 s | 19.7 s | 84 MB |

The curve flattens at 48: GMod is at its best there and 64 is slightly worse,
while Valheim gains 6% for another 11 MB. `--concurrency 16` holds a download to
~40 MB if the footprint matters more than the 35–55% it costs.

Measure this with interleaved repeats or not at all — a single sweep cannot tell
a 2% difference from the link having a bad minute, and an earlier attempt at
this table put 16 ahead of 24 on noise alone.

**glibc is pinned at startup, or it hoards.** A download decrypts and
decompresses a megabyte per chunk and frees it again, thousands of times over.
glibc reads that as evidence that blocks that size are worth keeping, raises its
dynamic mmap threshold, and stops returning them to the kernel — so peak memory
becomes what the allocator decided to retain rather than what the download had
in flight. The same Valheim install measures **80.2 MB** with the allocator left
alone against **25 MB** pinned, and drifts about 20% between identical runs
because the threshold settles somewhere different each time.

The knobs are environment variables glibc only reads at startup, so `main` calls
`tapline::retune()`, which re-executes the process once with them set. It costs
one `execve`. `TAPLINE_NO_MALLOC_TUNING=1` turns it off.

**Embedding tapline?** `retune()` replaces the running process, which is right
for a binary and catastrophic for a library or a plugin, so nothing calls it for
you. Set the variables on the process instead, before it starts —
`tapline::tuning::ENVIRONMENT` is the list, so a launcher does not have to copy
the numbers out of the documentation:

```sh
MALLOC_ARENA_MAX=2 MALLOC_TRIM_THRESHOLD_=131072 MALLOC_MMAP_THRESHOLD_=131072
```

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
`Dockerfile` builds a **5.1 MB image on `scratch`** — one static binary and
nothing else, not even a shell. (It was 1.6 MB before the zip, pipeline and
extension crates; the number here is measured per release, not aspirational.)

```sh
docker build -t tapline .
docker run --rm -v /srv/gmod:/data tapline \
  +login anonymous +force_install_dir /data +app_update 4020 +quit
```

The whole surface works in there, filtering included — verified by running it
against Steam and comparing every file against a full extraction, 195 of 348
entries byte-for-byte correct:

```sh
docker run --rm -v /srv/gmod:/data tapline \
  workshop download 4000 104691717 --dir /data --only 'lua/**'
```

It is also the cheapest place tapline runs, because the image is a musl build
and musl's allocator has none of the retention behaviour glibc has to be talked
out of — `retune()` is compiled out there, and there is nothing to fix. A full
1.5 GB Valheim install against a hard cgroup limit:

| `--memory` | default (48 in flight) | `--concurrency 10` |
|---|---|---|
| 32m | fails | 88 s |
| 48m | fails | 58 s |
| 64m | fails | — |
| 80m | 36 s | — |
| 96m | **19 s** | — |

```sh
docker run --rm --memory=96m -v /srv/valheim:/data tapline app download 896660 --dir /data
```

**Give it 96m, not the 80m it survives on.** The floor and the working figure
are different numbers: the same install takes 36 s at 80m and 19 s at 96m. A
cgroup limit counts page cache, so a tight one starves the writeback path and
the download goes slow well before it goes OOM. Squeezing a container to the
point where it still passes is how you end up with something that works in
testing and crawls in production.

The same applies to the cheap column: `--concurrency 10` fits in 32m, and takes
88 s doing it against 12 s unconstrained.

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

One thing does not carry across the boundary: `retune()` replaces the running
process, so the library never calls it for you and the FFI must not — restarting
someone's Node server to save 50 MB is not a trade tapline gets to make. A host
that wants the memory profile sets the variables before it starts:

```sh
MALLOC_ARENA_MAX=2 MALLOC_TRIM_THRESHOLD_=131072 MALLOC_MMAP_THRESHOLD_=131072 node app.js
```

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
decoder and nothing else changes.

### Reading an archive without downloading it

A depot file is stored as content-addressed chunks, each carrying the offset its
plaintext belongs at, so tapline can fetch a *range* of a file. That makes two
things possible.

Two formats implement it. A GMAD says its index is in the first 64 KiB; a ZIP
says the **last** 66 KiB, and everything downstream is identical:

```rust
workshop(app, item).gma().only("lua/**").dir("/out").run().await?;
workshop(app, item).zip().pick("readme.txt").dir("/out").run().await?;
```

**Listing.** What is in this archive, without paying for the archive:

```rust
let listing = workshop(4000, 104_691_717).gma().list().await?;
```

Measured on PAC3: **348 entries known after reading 65 KB of 8.7 MB** — the
first chunk, whatever the archive's size, because GMAD's index is at the front.
A format says where its index lives (`IndexLocation::Head(n)` or `Tail(n)`), so
a ZIP would ask for its last 64 KB and nothing else would change.

**Filters that stop paying for what they discard.** Read front to back, a filter
still pulls every byte across the wire and drops the ones it did not want. Read
by range, the chunks holding unselected entries are never asked for:

| | entries | fetched |
|---|---|---|
| whole archive | 348 | 3.17 MB |
| `only("lua/**")` | 195 | **816 KB** |
| `pick_all([three files])` | 3 | **204 KB** |

`only(..)` takes a glob; `pick(..)` takes an exact name, so a file called
`weapons/ak[47].lua` can be asked for by its own name rather than by something
that happens to match it. They differ in one more way: a pattern matching
nothing is a legitimate answer, while a **named** file that is not in the
archive is an error — naming it was a claim about the archive, and an empty
result would look like success.

An earlier version of this document claimed a container with its index at the
end could not be streamed. That is true of a socket and false here — tapline
does not read a byte stream, it fetches chunks by offset, and reading a ZIP's
central directory first is an ordinary read. The ZIP reader exists now and does
exactly that.

The ZIP support is deliberately narrow: no ZIP64, no encryption, no methods
beyond stored and deflate. Each is refused by name rather than guessed at,
because a reader that half-supports a format writes files that are wrong rather
than missing.

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
pipe      the typed chain        zip      ZIP, read by range
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
cargo test --workspace                # 483 tests, no network
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
