# tapline

A Rust implementation of Steam's content delivery: CM protocol, PICS, depot
manifests, the SteamPipe CDN chunk pipeline and Workshop/UGC. It replaces
`steamcmd` for installing dedicated servers and Workshop content.

It is two things at once:

- **a library** you link into a Rust service, with a real API instead of a
  subprocess whose stdout you parse;
- **a single static binary** that understands `steamcmd`'s own command grammar,
  so existing scripts keep working.

```sh
# steamcmd's grammar, unchanged
tapline +login anonymous +force_install_dir /srv/tf2 +app_update 232250 validate +quit

# or the native one, with machine-readable output
tapline app plan 232250 --dir /srv/tf2 --json
tapline app download 232250 --dir /srv/tf2 --branch public --validate
```

```rust
let session = Session::anonymous().await?;
let plan = session.app(232250).install_dir("/srv/tf2").plan().await?;
println!("{} to download, {} reused", plan.download_bytes, plan.reused_bytes);
```

## What it does not do

tapline downloads content the signed-in account is entitled to, exactly as the
real client does — depot keys come from Steam, and Steam only hands them out for
owned or anonymous-accessible depots. There is no key dumping, no `keys.txt`, no
unowned-depot access, no DRM removal and no `.lua` manifest sideloading, and
those are not oversights to be filed as feature requests.

It also never executes a depot's `installscript.vdf`. steamcmd does; tapline
parses it and reports it. Installing a game server should not be a
remote-code-execution primitive.

## Status

Early. See `docs/PLAN.md` for the milestone list — the short version is that
nothing works yet.

## Licence

MPL-2.0.
