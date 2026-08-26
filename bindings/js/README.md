# tapline for Deno, Bun and Node

Install Steam apps, dedicated servers and Workshop content from JavaScript. A
`steamcmd` replacement you call instead of spawn.

```ts
import { install } from "tapline";

const report = await install({ app: 4020, dir: "/srv/gmod" });
console.log(`${report.files} files, ${report.bytesDownloaded} bytes`);
```

That installs a Garry's Mod dedicated server that is ready to run — correct
permissions included, which is the difference between a byte-correct tree and a
server that starts.

## One object, four ways to use it

`install`, `plan` and `downloadWorkshopItem` all return a `Job`. It is awaitable,
iterable, cancellable and callback-able, so none of those is a different function
to learn.

```ts
// Await it.
const report = await install({ app: 4020, dir: "/srv/gmod" });

// Watch it. `percent` is worked out for you.
await install({
  app: 4020,
  dir: "/srv/gmod",
  onProgress: (p) => console.log(`${p.percent.toFixed(1)}%`),
});

// Iterate it.
for await (const event of install({ app: 4020, dir: "/srv/gmod" })) {
  if (event.kind === "fileCompleted") console.log(event.path);
}

// Callback, if that suits the surrounding code better.
install({ app: 4020, dir: "/srv/gmod" }).callback((err, report) => {});

// Stop it. What is on disk stays, and the next install resumes from it.
const job = install({ app: 4020, dir: "/srv/gmod" });
setTimeout(() => job.cancel(), 5_000);
```

Ask what an update costs before committing the disk. A "20 GB update" is usually
a few hundred megabytes of changed chunks, and this answers that without
downloading anything:

```ts
const { downloadBytes, reusedBytes, totalBytes } = await plan({
  app: 4020,
  dir: "/srv/gmod",
});
```

Workshop items too. Ids exceed `Number.MAX_SAFE_INTEGER`, so pass a bigint or a
string:

```ts
await downloadWorkshopItem({ app: 4000, item: 104691717n, dir: "/srv/gmod" });
```

### Garry's Mod addons

An addon is a single `.gma`, and a server looks for it in `garrysmod/addons`.
By default items land where steamcmd puts them —
`<dir>/steamapps/workshop/content/4000/<item>/` — which is four directories from
where the server looks. `layout: "flat"` writes into the folder you name:

```ts
await downloadWorkshopItem({
  app: 4000,
  item: 104691717n,                       // PAC3
  dir: "/srv/gmod/garrysmod/addons",
  layout: "flat",
});
// /srv/gmod/garrysmod/addons/104691717.gma
```

A whole collection goes into the same folder, side by side, which is what a
`garrysmod/addons` directory is:

```ts
await Promise.all(
  [104691717n, 3790437566n].map((item) =>
    downloadWorkshopItem({
      app: 4000,
      item,
      dir: "/srv/gmod/garrysmod/addons",
      layout: "flat",
    })
  ),
);
```

They share the download budget, as any concurrent downloads do. The default
stays `"steamcmd"` because that is where the Steam client and wings eggs look,
and moving it would relocate every existing consumer's files.

From the CLI it is `--flat`:

```sh
tapline workshop download 4000 104691717 --dir /srv/gmod/garrysmod/addons --flat
```

### Unpacking and converting addons

A `.gma` is a container. `extensions` runs post-processing on each file as it
lands:

```ts
await downloadWorkshopItem({
  app: 4000,
  item: 104691717n,
  dir: "/srv/gmod/garrysmod/addons",
  layout: "flat",
  extensions: ["gmad", "gmad-zip"],
});
// 104691717.gma   the download
// 104691717/      348 files, unpacked
// 104691717.zip   the same, as a zip
```

| name | what it does |
|---|---|
| `gmad` | unpacks the `.gma` into a directory beside it |
| `gmad-zip` | converts it to a `.zip` beside it |
| `gmad-zip-stored` | the same without deflating — 4× faster, for hosts that compress on the wire |
| `gmad!` / `gmad-zip!` | as above, then delete the `.gma` |

Measured on PAC3 (348 files, 8.69 MB unpacked):

| | time | throughput |
|---|---|---|
| unpack | 19 ms | 429 MB/s |
| convert to zip, stored | 10 ms | 805 MB/s |
| convert to zip, deflated | 39 ms | 213 MB/s |

Deflating is the expensive part, so it runs across every core — it was 175 ms
single-threaded, for byte-identical output. The zip is checked against `unzip -t`
in the test suite, because an archive only this crate can read is not a zip.

These are **names, not functions**. No callback crosses the FFI boundary, and an
extension is Rust compiled into the library rather than code you supply — the
same reason tapline parses `installscript.vdf` and refuses to run it. An unknown
name is an error, not a silent no-op.

They run on a blocking thread after the file is synced, never on the loop that
dispatches downloads. `onEvent` reports each one:

```ts
{ kind: "extended", extension: "gmad", path: "104691717.gma", produced: 348 }
```

From the CLI:

```sh
tapline workshop download 4000 104691717 --dir /srv/gmod/garrysmod/addons \
  --flat --extensions gmad,gmad-zip
```

## Installing

```sh
deno add npm:tapline     # nothing else needed
bun  add tapline         # nothing else needed
npm  install tapline koffi
```

Node has no built-in FFI, so it needs [koffi](https://koffi.dev). Deno and Bun
have one built in and need nothing. If koffi is missing, tapline says so rather
than failing obscurely.

Node also needs **22.18 or newer**. The package ships TypeScript with no build
step, and that is the version where importing it stopped needing a flag. Deno
and Bun have never cared.

### The shared library is not in the package yet

There are no prebuilt binaries on npm, so for now you build it once:

```sh
cargo build --release -p tapline-ffi   # or: npm run build:lib
```

It is then found automatically — beside the package, or in the workspace's
`target/release` — with no configuration. `TAPLINE_LIB` overrides that if the
library lives somewhere else. When it cannot be found, the error lists every
path that was tried, because "library not found" with no indication of where it
looked is the least useful thing a binding can say.

## Why there are no native callbacks

The obvious design is to hand a function pointer to Rust and have the download
call it. It is the wrong one here, and not for style reasons.

Deno needs `Deno.UnsafeCallback.threadSafe` plus manual `ref()`/`unref()` to keep
the isolate alive; Bun's `JSCallback` invoked from a non-JS thread is fragile;
Node has no FFI at all without a third-party package. Worse, all three share one
failure mode — a download thread calling into an isolate that has been torn down
takes the process with it, intermittently, at exit, in your application.

So no function pointer crosses the boundary. Jobs run on tapline's own threads
and push events into a queue, and the binding pulls them. Deno and Node get a
real promise from a nonblocking native call; Bun polls with a zero timeout because
its FFI is synchronous only. All three then present the same promises, iterators
and callbacks, in JavaScript, where that machinery belongs.

## Several downloads at once

Downloads started from one process share a single budget of chunks in flight and
a single connection pool. Nothing to opt into — just start them:

```ts
const [gmod, valheim] = await Promise.all([
  install({ app: 4020, dir: "/srv/gmod" }),
  install({ app: 896660, dir: "/srv/valheim" }),
]);
```

That matters because throughput turns over past ~64 chunks in flight, so N
downloads each taking a full budget is slower than N sharing one. Three
concurrent Valheim installs, 4.40 GB total:

| total budget | wall clock | throughput | spread between finishes |
|---|---|---|---|
| **64 (shared, the default)** | **18.3–21.3 s** | **197–230 MB/s** | 0.9 s |
| 192 (a full budget each) | 24.7 s | 170 MB/s | 6.7 s |

Sharing is both faster and fairer: the three finish within a second of each
other instead of nearly seven seconds apart. Three sharing 64 also beat a single
download at 64 (~184 MB/s) — one download cannot keep 64 requests busy on its
own, and another download's chunks fill the gaps.

When you would rather they finish one at a time — a provisioning tool usually
wants the first server online sooner, not the whole batch marginally earlier —
`installAll` runs them under a limit:

```ts
import { installAll } from "tapline";

await installAll(specs, {
  maxConcurrent: 1,
  onEach: (report, i) => console.log(`server ${i} ready`),
});
```

Measured on three Valheim installs:

| | first ready | all ready |
|---|---|---|
| `maxConcurrent: 3` | 18.0 s | **18.7 s** |
| `maxConcurrent: 1` | **9.5 s** | 27.2 s |

All at once finishes the batch sooner; one at a time gets the first server
running in half the time. Defaults to all at once.

Raise or lower the budget before starting anything:

```ts
import { setTotalConcurrency, concurrency } from "tapline";

await setTotalConcurrency(96);
console.log(await concurrency()); // { total: 96, available: 96 }
```

It is fixed once a download starts, because moving it underneath running
downloads is not something you could reason about.

## Events

Every job emits a stream, discriminated by `kind`:

| kind | when |
|---|---|
| `planned` | always first: the denominator, before the numerator moves |
| `depotStarted`, `depotCompleted` | per depot |
| `progress` | per chunk written — frequent, aggregate before drawing |
| `fileCompleted` | the file is synced to disk, safe to read or execute |
| `retrying` | a fetch failed and is being retried |
| `finished` | the job succeeded; carries the report |
| `error` | the job failed; carries the message |

`retrying` with `reason: "integrityFailure"` is the one worth logging: a CDN or
a caching proxy served bytes that were not what the manifest named, and tapline
refetched rather than writing them.

`manifest` on `depotStarted` is a **string**, not a number. Manifest ids exceed
2^53 and JavaScript would round them silently.

## Testing

One file, three runtimes, because the thing worth testing is that the same code
behaves the same way in each:

```sh
bun            test/smoke.ts
node           test/smoke.ts
deno run --allow-ffi --allow-env --allow-read --unstable-ffi test/smoke.ts
```

No `TAPLINE_LIB` needed: the workspace build is found on its own.

Add `TAPLINE_LIVE=1` for the tests that talk to Steam and download real content.
Measured on 2026-08-26: 10/10 on Deno 2.9.5, Bun 1.3.14 and Node 26.7.0.
