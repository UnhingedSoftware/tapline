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
await downloadWorkshopItem({ app: 4000, item: 3790437566n, dir: "/srv/gmod" });
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

The native library is found next to the package, or wherever `TAPLINE_LIB`
points — which is what you want while working on tapline itself:

```sh
TAPLINE_LIB=../../target/release/libtapline_ffi.so bun test/smoke.ts
```

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

Add `TAPLINE_LIVE=1` for the tests that talk to Steam and download real content.
Measured on 2026-08-26: 10/10 on Deno 2.9.5, Bun 1.3.14 and Node 26.7.0.
