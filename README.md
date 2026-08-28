# tapline

A Rust implementation of Steam's content delivery — CM protocol, PICS, depot
manifests, the SteamPipe CDN pipeline and Workshop/UGC. It replaces `steamcmd`
for installing dedicated servers and Workshop content.

Two things at once: **a library** you link into a Rust service, and **a single
static binary** that also understands `steamcmd`'s own command grammar, so
existing scripts keep working.

- Installs are byte-for-byte identical to steamcmd's, permissions included.
- One binary. No 32-bit runtime, no Steam client, no CA bundle. 5 MB on
  `scratch`.
- Machine-readable output everywhere (`--json`), and a real API instead of a
  subprocess whose stdout you parse.

## Install

```sh
cargo build --release          # target/release/tapline
docker build -t tapline .      # or a 5 MB scratch image
```

## Command line

### Apps and dedicated servers

```sh
# What would an install or update cost? Fetches no content.
tapline app plan 896660 --dir /srv/valheim --json

# Install or update. Downloads nothing if already current.
tapline app download 896660 --dir /srv/valheim
tapline app download 896660 --dir /srv/valheim --validate --concurrency 24

# Depots, branches and sizes.
tapline app info 896660
```

steamcmd's grammar works unchanged:

```sh
tapline +login anonymous +force_install_dir /srv/tf2 +app_update 232250 +quit
```

### Workshop

```sh
# Search. No key and no login needed.
tapline workshop search 4000 --text stargate --sort text --limit 5
tapline workshop search 4000 --tag Weapon --sort subscribed

# Describe items by id.
tapline workshop info 104691717 3790437566

# Download one, into the folder a server actually reads.
tapline workshop download 4000 104691717 --dir /srv/gmod/garrysmod/addons --flat
```

`--sort` is `vote`, `recent`, `updated`, `trend`, `subscribed` or `text` (which
needs `--text`). Paging is a cursor: pass the `next_cursor` a page prints back
as `--cursor`.

**Take part of an item**, fetching only the chunks it lives in — an 8.4 MB addon
filtered to `lua/**` moves 816 KB instead of 3.17 MB:

```sh
tapline workshop download 4000 104691717 --dir out --only 'lua/**'
tapline workshop download 4000 104691717 --dir out --pick lua/autorun/init.lua
tapline workshop download 4000 104691717 --dir out --only 'lua/**' --stream zip
```

**Unpack or convert as it lands**, without keeping the `.gma`:

```sh
tapline workshop download 4000 104691717 --dir addons --flat --extensions gmad,gmad-zip
tapline workshop download 4000 104691717 --dir addons --stream    # never writes the .gma
```

| extension | does |
|---|---|
| `gmad` | unpacks the `.gma` beside it |
| `gmad-zip` | converts it to a `.zip` |
| `gmad-zip-stored` | the same without deflating — 4× faster |
| `gmad!`, `gmad-zip!` | as above, then delete the `.gma` |

### Signing in

Anonymous is the default and covers every dedicated server and Workshop search.
Content an account *owns* needs a login; the token is saved, so it happens once.

```sh
tapline login --qr                                    # approve in the mobile app
tapline login --username NAME --password PASS
tapline login --username NAME --password-stdin < pass.txt
TAPLINE_PASSWORD=... tapline login --username NAME
tapline whoami
```

Steam Guard codes come from `TAPLINE_GUARD_CODE` or a prompt. `--password` puts
the password in your shell history and in every `ps` listing while it runs;
`--password-stdin` does not.

`whoami` also reports the Steam accounts and library paths this machine's Steam
client is configured with. tapline reads that *identity* to save you typing a
name — it cannot reuse the client's session, because a modern client keeps its
token in its own encrypted store.

When content needs an account, the error says so:

```
Steam refused a key for depot 734: access denied. This depot is not anonymously
accessible, so it needs an account that owns it — run `tapline login`
```

## Library

```rust
use tapline::{AppId, BrowseQuery, Session};

// Signs in if a token is saved, anonymous otherwise.
let mut session = Session::automatic(None).await?;

let plan = session.plan(AppId(896_660), &options).await?;
println!("{} to download, {} reused", plan.download_bytes, plan.reused_bytes);

session.install(AppId(896_660), &options).await?;

// Search, then download a result directly — no second lookup.
let page = session
    .browse_workshop(&BrowseQuery { app: AppId(4000), ..Default::default() })
    .await?;
session.download_workshop_item(&page.items[0].item, &options).await?;
```

Signing in, when content needs it:

```rust
// Third argument is the Steam Guard code, when the account has one.
let token = session.sign_in("username", "password", None).await?;
tapline_auth::TokenStore::default_file().save(&token)?;   // reused automatically after this
```

`install_observed` and `download_workshop_item_observed` take a callback and
report progress, retries and per-file completion. Errors are concrete enums;
`InstallError::needs_login()` says when signing in would help.

## JavaScript

Deno, Bun and Node, over a C ABI. See `bindings/js/README.md`.

```ts
import { install, searchWorkshop, downloadWorkshopItem } from "tapline";

await install({ app: 4020, dir: "/srv/gmod",
                onProgress: (p) => console.log(`${p.percent.toFixed(1)}%`) });

const page = await searchWorkshop({ app: 4000, text: "stargate", sort: "text" });
await downloadWorkshopItem({ app: 4000, item: page.items[0].item, dir: "/srv/gmod" });
```

Every job is awaitable, async-iterable, cancellable and callback-able. Item ids
are strings — they exceed `Number.MAX_SAFE_INTEGER`.

## In a container

```sh
docker run --rm --memory=96m -v /srv/valheim:/data tapline \
  app download 896660 --dir /data
```

Give it 96 MB rather than the 80 MB it survives on: the same install takes 36 s
at 80m and 19 s at 96m, because a cgroup limit counts page cache and a tight one
starves the writeback path.

## Numbers

Measured on a 2.5 Gb link with a 1.9 GB/s disk. Wire bytes, not installed bytes.

| | steamcmd | tapline |
|---|---|---|
| Valheim DS (1.47 GB) | 19.0 s | **8.2 s** |
| Garry's Mod DS (3.54 GB) | 29.5 s | **19.1 s** |

A download holds about **73 MB**, whatever its size — 6.8 GB costs what 1.5 GB
does, because chunks are written straight to their offset as they arrive.
`--concurrency` trades the two: peak RSS is roughly `15 + 1.1 × N` MB, and 48 is
the fewest chunks in flight that still reach full speed. `--concurrency 16`
holds a download to ~40 MB for about 35% of the speed.

An update that finds nothing changed re-checks 6.8 GB across 2,329 files in
2.1 s and downloads nothing.

## Safety

Workshop content is attacker-influenced: anyone can publish an item, and its
manifest names the paths tapline will create.

- Every path resolves against a rooted directory handle. `..`, absolute paths
  and escaping symlinks fail the install rather than being skipped.
- Every chunk's SHA-1 is verified before it reaches the disk.
- Decompression is bounded by the manifest's own sizes.
- `installscript.vdf` is parsed and reported, never executed.
- Passwords are RSA-encrypted with Steam's key and never written to disk. Only
  the refresh token persists.
- `unsafe_code = "forbid"` across the workspace; `unwrap`, `expect`, `panic` and
  indexing are denied outside tests.

## Building

```sh
cargo test --workspace
cargo clippy --workspace --all-targets
cargo build --release
```

Network tests are `#[ignore]`d, so the default suite is offline. Run them with
`-- --ignored`; the steamcmd differential needs `TAPLINE_STEAMCMD_DIR` pointing
at a steamcmd install of the same app.

## Licence

**AGPL-3.0-only.** You may use, modify and redistribute tapline freely. If you
run a modified version as a network service, §13 requires you to offer its
source to the people using that service — linking it into your own program makes
that program a combined work under the same terms.

If those terms do not suit your project, **commercial licences are available**:
the copyright holder is not bound by the AGPL and can grant other terms. Open an
issue.

**Contributions** need a copyright assignment or a licence grant broad enough to
keep dual licensing possible. Without one, a contribution can only ever be used
under the AGPL — including by the maintainer. Nothing is accepted without it, so
that the offer above stays honest.

tapline downloads only content the signed-in account is entitled to, exactly as
the real client does — no key dumping, no unowned depots, no DRM removal.
