# tapline for Deno, Bun and Node

Install Steam apps, dedicated servers and Workshop content from JavaScript. A
`steamcmd` replacement you call instead of spawn.

```ts
import { install } from "tapline";

const report = await install({ app: 4020, dir: "/srv/gmod" });
console.log(`${report.files} files, ${report.bytesDownloaded} bytes`);
```

That installs a Garry's Mod dedicated server ready to run — correct permissions
included, which is the difference between a byte-correct tree and a server that
starts.

## Installing

```sh
deno add npm:tapline     # nothing else needed
bun  add tapline         # nothing else needed
npm  install tapline koffi
```

Node has no built-in FFI, so it needs [koffi](https://koffi.dev), and **22.18 or
newer** — the package ships TypeScript with no build step. Deno and Bun need
neither.

There are no prebuilt binaries on npm yet, so build the shared library once:

```sh
cargo build --release -p tapline-ffi   # or: npm run build:lib
```

It is then found automatically, beside the package or in the workspace's
`target/release`. `TAPLINE_LIB` overrides that.

## One object, four ways to use it

Every call returns a `Job`. It is awaitable, iterable, cancellable and
callback-able, so none of those is a different function to learn.

```ts
const report = await install({ app: 4020, dir: "/srv/gmod" });

await install({ app: 4020, dir: "/srv/gmod",
                onProgress: (p) => console.log(`${p.percent.toFixed(1)}%`) });

for await (const event of install({ app: 4020, dir: "/srv/gmod" })) {
  if (event.kind === "fileCompleted") console.log(event.path);
}

install({ app: 4020, dir: "/srv/gmod" }).callback((err, report) => {});

// Stop it. What is on disk stays, and the next install resumes from it.
const job = install({ app: 4020, dir: "/srv/gmod" });
setTimeout(() => job.cancel(), 5_000);
```

Ask what an update costs before committing the disk — a "20 GB update" is
usually a few hundred megabytes of changed chunks:

```ts
const { downloadBytes, reusedBytes } = await plan({ app: 4020, dir: "/srv/gmod" });
```

## Finding items

```ts
import { searchWorkshop, downloadWorkshopItem } from "tapline";

const page = await searchWorkshop({ app: 4000, text: "stargate", sort: "text" });
for (const found of page.items) console.log(found.item, found.title, found.subscriptions);

// A result is what a download takes — no second lookup.
await downloadWorkshopItem({ app: 4000, item: page.items[0].item, dir: "/srv/gmod" });
```

No key and no login needed.

| option | does |
|---|---|
| `text` | free text to match |
| `tags` / `excludeTags` | filter by tag; `allTags` requires every one |
| `sort` | `vote`, `recent`, `updated`, `trend`, `subscribed`, `text` |
| `limit` | results per page, capped at 100 |
| `cursor` | `nextCursor` from a previous page |

**Item ids are strings.** They exceed `Number.MAX_SAFE_INTEGER`, and a rounded
id is a different item. Paging is a cursor, not a page number — walk it until
`nextCursor` is `null`:

```ts
let cursor: string | null = null;
do {
  const page = await searchWorkshop({ app: 4000, tags: ["Weapon"], limit: 50,
                                      cursor: cursor ?? undefined });
  handle(page.items);
  cursor = page.nextCursor;
} while (cursor);
```

## Workshop items

Ids exceed `Number.MAX_SAFE_INTEGER`, so pass a bigint or a string:

```ts
await downloadWorkshopItem({ app: 4000, item: 104691717n, dir: "/srv/gmod" });
```

A Garry's Mod server looks for addons in `garrysmod/addons`, not where steamcmd
puts them. `layout: "flat"` writes into the folder you name:

```ts
await downloadWorkshopItem({
  app: 4000, item: 104691717n, dir: "/srv/gmod/garrysmod/addons", layout: "flat",
});
// /srv/gmod/garrysmod/addons/104691717.gma
```

### Unpacking and converting

`extensions` post-processes each file as it lands:

```ts
await downloadWorkshopItem({
  app: 4000, item: 104691717n, dir: "/srv/gmod/garrysmod/addons",
  layout: "flat", extensions: ["gmad", "gmad-zip"],
});
```

| name | does |
|---|---|
| `gmad` | unpacks the `.gma` beside it |
| `gmad-zip` | converts it to a `.zip` |
| `gmad-zip-stored` | the same without deflating — 4× faster |
| `gmad!`, `gmad-zip!` | as above, then delete the `.gma` |

These are **names, not functions** — an extension is Rust compiled into the
library, not code you supply. An unknown name is an error, not a silent no-op.

`stream: true` writes each file as its bytes arrive and never stores the `.gma`
at all. It implies the flat layout and ignores `extensions`, since the archive
they would act on never exists.

## Pipelines

When you want *part* of an item, or it converted as it lands:

```ts
import { workshop } from "tapline";

const report = await workshop(4000, 104691717)
  .gma()                       // read it as a Garry's Mod addon
  .only("lua/**")              // take the Lua, leave the models
  .zip("/srv/addons/out.zip"); // one destination, and it ends the chain
```

**A selection makes the download selective**: the chunks holding entries you did
not ask for are never requested. Measured on a real 8.4 MB addon, `only("lua/**")`
fetched **816 KB of 3.17 MB**.

| step | does |
|---|---|
| `.gma()` / `.zip()` / `.decode(name)` | how to read the download |
| `.only(glob)` | take matching entries; repeatable |
| `.pick(path)` | take one exact path; missing it is an **error** |
| `.onProgress(fn)` / `.onEvent(fn)` | watch it run |
| `.dir(p)` / `.zip(p)` / `.zipStored(p)` | where it goes — ends the chain |

`only` and `pick` differ deliberately: a pattern matching nothing is a
legitimate answer, a named file that is not there means you are wrong about the
archive. Every step returns a new chain, so a half-built one is safe to reuse.

## Several downloads at once

Downloads in one process share a single budget of chunks in flight and one
connection pool. Concurrent calls need nothing:

```ts
await Promise.all(items.map((item) =>
  downloadWorkshopItem({ app: 4000, item, dir: "/srv/gmod/garrysmod/addons", layout: "flat" })));
```

Raise the budget before starting anything:

```ts
import { setTotalConcurrency } from "tapline";
await setTotalConcurrency(96);
```

It is chunks in flight, so it also bounds memory: peak RSS is about
`15 + 1.1 × total` MB regardless of download size. The default of 48 costs
around 65 MB and is where a single download stops getting faster.

On glibc, add these to whatever starts your process or it will retain roughly
twice that:

```sh
MALLOC_ARENA_MAX=2 MALLOC_TRIM_THRESHOLD_=131072 MALLOC_MMAP_THRESHOLD_=131072 node app.js
```

## Events

Every job emits a stream, discriminated by `kind`: `planned`, `depotStarted`,
`progress`, `fileCompleted`, `retrying`, `verifying`, `extended`, `result`,
`searched`, `streamed`, `piped`, `finished`, `error`.

## Why there are no native callbacks

No function pointer crosses the FFI boundary. All three runtimes share one
failure mode — a download thread calling into an isolate that has been torn down
takes the process with it, intermittently, at exit, in your application.

So jobs run on tapline's own threads and push events into a queue that the
binding pulls. The promises, iterators and callbacks are built in JavaScript,
where that machinery belongs.

## Testing

```sh
cargo build --release -p tapline-ffi
TAPLINE_LIB=../../target/release/libtapline_ffi.so bun test/smoke.ts
TAPLINE_LIVE=1 TAPLINE_LIB=... deno run -A test/smoke.ts   # includes real downloads
```

The same file runs on all three runtimes.
