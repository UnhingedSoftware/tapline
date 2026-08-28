/**
 * Loading the shared library, in whichever runtime we happen to be.
 *
 * The three runtimes disagree about everything except that they can call a C
 * function, so this module is the only place that knows which one we are in.
 * Everything above it sees one `Ffi` object.
 *
 * The important difference is how `tapline_job_next` is called. It blocks until
 * an event arrives or its timeout elapses:
 *
 * - Deno marks it `nonblocking`, which runs it on a thread pool and returns a
 *   real promise. No polling, no latency.
 * - Node's koffi has `.async()`, same idea.
 * - Bun's FFI is synchronous only, so it is called with a zero timeout and
 *   polled. That costs a wake-up every few milliseconds during a download and
 *   nothing at all when idle.
 */

/** How the library is reached, regardless of runtime. */
export interface Ffi {
  install(
    app: number,
    dir: string,
    branch: string | null,
    concurrency: number,
    os: number,
    validate: number,
    includeDlc: number,
    fileModes: number,
    extensions: string | null,
  ): bigint;
  plan(
    app: number,
    dir: string,
    branch: string | null,
    os: number,
    includeDlc: number,
  ): bigint;
  workshop(
    app: number,
    item: bigint,
    dir: string,
    concurrency: number,
    flat: number,
    extensions: string | null,
    stream: number,
  ): bigint;
  /** Searches an app's Workshop. */
  search(
    app: number,
    text: string | null,
    searchIn: string | null,
    tags: string | null,
    tagGroups: string | null,
    excludedTags: string | null,
    excludedContent: string | null,
    allTags: number,
    sort: string | null,
    trendDays: number,
    createdSince: number,
    createdUntil: number,
    updatedSince: number,
    updatedUntil: number,
    limit: number,
    cursor: string | null,
    page: number,
    countOnly: number,
  ): bigint;
  /** Signs in with a QR code, emitting the code and its refreshes. */
  qrLogin(timeoutSecs: number): bigint;
  /** Runs a pipeline given in its text form. */
  pipeline(
    app: number,
    item: bigint,
    spec: string,
    concurrency: number,
  ): bigint;
  /** Waits for the next event. Resolves to null when the job is over. */
  next(job: bigint, timeoutMs: number): Promise<string | null>;
  cancel(job: bigint): void;
  free(job: bigint): void;
  version(): string;
  /** The last error on this thread, or "" if there is none. */
  lastError(): string;
  /** Sets the process-wide chunk budget. Must precede the first job. */
  setTotalConcurrency(chunks: number): number;
  /** The process-wide chunk budget. */
  totalConcurrency(): number;
  /** How much of it is free right now. */
  availableConcurrency(): number;
  /** True when `next` genuinely suspends rather than polling. */
  readonly nativeAsync: boolean;
}

/** Return codes, matching the Rust constants. */
export const OK = 0;
export const TIMEOUT = 1;
export const DONE = 2;
export const BUFFER_TOO_SMALL = -1;
export const BAD_ARGUMENT = -2;

/** Most events are well under this; the buffer grows if one is not. */
/// How long to wait between polls on a synchronous FFI.
///
/// Bun's FFI and Node's built-in one are both synchronous, so a blocking call
/// would hold the only thread that can resolve the promise. Zero timeout plus a
/// yield is the honest translation: it never blocks, and costs one wake-up per
/// tick. Deno's is nonblocking and does not need this.
const POLL_MS = 4;

const INITIAL_BUFFER = 4096;

/** NUL-terminated UTF-8, which is what a `const char *` means. */
function cstring(value: string): Uint8Array {
  const bytes = new TextEncoder().encode(value);
  const out = new Uint8Array(bytes.length + 1);
  out.set(bytes);
  return out;
}

/** An environment variable, in whichever runtime this is. */
function getEnv(key: string): string | undefined {
  // deno-lint-ignore no-explicit-any
  const g = globalThis as any;
  if (g.Deno?.env?.get) return g.Deno.env.get(key) ?? undefined;
  if (g.process?.env) return g.process.env[key];
  return undefined;
}

/** The platform, spelled the way Node spells it. */
function getPlatform(): string {
  // deno-lint-ignore no-explicit-any
  const g = globalThis as any;
  if (g.Deno?.build?.os) {
    const os = g.Deno.build.os;
    return os === "windows" ? "win32" : os;
  }
  return g.process?.platform ?? "linux";
}

/** The library's filename on this platform. */
function libraryName(): string {
  const platform = getPlatform();
  if (platform === "darwin") return "libtapline_ffi.dylib";
  if (platform === "win32") return "tapline_ffi.dll";
  return "libtapline_ffi.so";
}

/**
 * Everywhere the shared library might be, best first.
 *
 * Returning candidates rather than one path is deliberate: "library not found"
 * with no indication of where it was looked for is the least useful error a
 * binding can produce, and it is the first thing every user of one hits.
 */
export function libraryCandidates(): string[] {
  const name = libraryName();
  const candidates: string[] = [];

  const override = getEnv("TAPLINE_LIB");
  if (override) candidates.push(override);

  const here = moduleDirectory();
  if (here) {
    // Beside the package, where a prebuilt or locally built copy would sit.
    candidates.push(`${here}/../${name}`);
    // And the workspace target directory, which is where it is while working
    // on tapline itself — the single most common source of this error.
    candidates.push(`${here}/../../../target/release/${name}`);
    candidates.push(`${here}/../../../target/debug/${name}`);
  }

  // Last: let the platform loader search its own paths.
  candidates.push(name);
  return candidates;
}

/** The first candidate that exists, or the bare name as a last resort. */
export async function resolveLibraryPath(): Promise<string> {
  const candidates = libraryCandidates();
  for (const candidate of candidates) {
    if (candidate.includes("/") && (await fileExists(candidate))) return candidate;
  }
  return candidates[candidates.length - 1] ?? libraryName();
}

/** This module's directory, for finding the library relative to it. */
function moduleDirectory(): string | undefined {
  const url = import.meta.url;
  if (!url.startsWith("file:")) return undefined;
  const path = decodeURIComponent(url.slice("file://".length));
  const cut = path.lastIndexOf("/");
  return cut === -1 ? undefined : path.slice(0, cut);
}

/** Whether a path exists. `node:fs` is the one API all three runtimes share. */
async function fileExists(path: string): Promise<boolean> {
  try {
    const fs = await import("node:fs");
    return fs.existsSync(path);
  } catch {
    return false;
  }
}

/** Which runtime this is. */
export function detectRuntime(): "deno" | "bun" | "node" {
  // deno-lint-ignore no-explicit-any
  const g = globalThis as any;
  if (g.Deno?.dlopen) return "deno";
  if (g.Bun) return "bun";
  return "node";
}

/** Opens the library for the current runtime. */
export async function load(path?: string): Promise<Ffi> {
  const resolved = path ?? (await resolveLibraryPath());
  try {
    switch (detectRuntime()) {
      case "deno":
        return await loadDeno(resolved);
      case "bun":
        return await loadBun(resolved);
      default:
        return await loadNode(resolved);
    }
  } catch (cause) {
    // koffi's own message is more useful than anything wrapped around it.
    if (cause instanceof Error && cause.message.includes("koffi")) throw cause;
    throw new Error(
      `could not load the tapline shared library.\n` +
        `Tried:\n${libraryCandidates().map((c) => `  ${c}`).join("\n")}\n` +
        `Build it with \`cargo build --release -p tapline-ffi\`, ` +
        `or point TAPLINE_LIB at it.`,
      { cause },
    );
  }
}

/** Reads a job pointer written through an out-parameter. */
function readJobPointer(
  out: BigUint64Array,
  code: number,
  what: string,
  lastError?: () => string,
): bigint {
  if (code !== OK) {
    // The library already recorded why. Reporting only the number turns
    // "unknown extension \"bogus\"" into "code -2", which tells nobody
    // anything.
    const detail = lastError?.() ?? "";
    throw new Error(detail ? `${what}: ${detail}` : `${what} failed (code ${code})`);
  }
  const job = out[0];
  if (job === undefined || job === 0n) {
    throw new Error(`${what} returned no job`);
  }
  return job;
}

// --- Deno ------------------------------------------------------------------

async function loadDeno(path: string): Promise<Ffi> {
  // deno-lint-ignore no-explicit-any
  const Deno = (globalThis as any).Deno;
  const lib = Deno.dlopen(path, {
    tapline_install: {
      parameters: [
        "u32", "buffer", "buffer", "u32", "u8", "u8", "u8", "u8", "buffer", "buffer",
      ],
      result: "i32",
    },
    tapline_plan: {
      parameters: ["u32", "buffer", "buffer", "u8", "u8", "buffer"],
      result: "i32",
    },
    tapline_workshop_download: {
      parameters: ["u32", "u64", "buffer", "u32", "u8", "buffer", "u8", "buffer"],
      result: "i32",
    },
    tapline_pipeline: {
      parameters: ["u32", "u64", "buffer", "u32", "buffer"],
      result: "i32",
    },
    tapline_qr_login: { parameters: ["u32", "buffer"], result: "i32" },
    tapline_workshop_search: {
      parameters: [
        "u32", "buffer", "buffer", "buffer", "buffer", "buffer", "buffer",
        "u8", "buffer", "u32", "u32", "u32", "u32", "u32", "u32", "buffer", "u32", "u8", "buffer",
      ],
      result: "i32",
    },
    tapline_job_next: {
      parameters: ["pointer", "u32", "buffer", "usize", "buffer"],
      result: "i32",
      // The whole reason this design uses a queue instead of callbacks: Deno
      // turns a blocking C call into a promise for free, on its own threads.
      nonblocking: true,
    },
    tapline_job_cancel: { parameters: ["pointer"], result: "void" },
    tapline_job_free: { parameters: ["pointer"], result: "void" },
    tapline_last_error: {
      parameters: ["buffer", "usize", "buffer"],
      result: "i32",
    },
    tapline_version: { parameters: [], result: "pointer" },
    tapline_set_total_concurrency: { parameters: ["u32"], result: "i32" },
    tapline_total_concurrency: { parameters: [], result: "u32" },
    tapline_available_concurrency: { parameters: [], result: "u32" },
  });

  const ptr = (value: bigint) => Deno.UnsafePointer.create(value);

  const lastError = (): string => {
    const len = new BigUint64Array(1);
    lib.symbols.tapline_last_error(null, 0n, new Uint8Array(len.buffer));
    const needed = Number(len[0] ?? 0n);
    if (needed === 0) return "";
    const buf = new Uint8Array(needed);
    lib.symbols.tapline_last_error(buf, BigInt(needed), new Uint8Array(len.buffer));
    return new TextDecoder().decode(buf);
  };

  return {
    nativeAsync: true,
    lastError,
    install(app, dir, branch, concurrency, os, validate, includeDlc, fileModes, extensions) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_install(
        app,
        cstring(dir),
        branch === null ? null : cstring(branch),
        concurrency,
        os,
        validate,
        includeDlc,
        fileModes,
        extensions === null ? null : cstring(extensions),
        new Uint8Array(out.buffer),
      );
      return readJobPointer(out, code, "install", lastError);
    },
    plan(app, dir, branch, os, includeDlc) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_plan(
        app,
        cstring(dir),
        branch === null ? null : cstring(branch),
        os,
        includeDlc,
        new Uint8Array(out.buffer),
      );
      return readJobPointer(out, code, "plan", lastError);
    },
    workshop(app, item, dir, concurrency, flat, extensions, stream) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_workshop_download(
        app,
        item,
        cstring(dir),
        concurrency,
        flat,
        extensions === null ? null : cstring(extensions),
        stream,
        new Uint8Array(out.buffer),
      );
      return readJobPointer(out, code, "workshop download", lastError);
    },
    pipeline(app, item, spec, concurrency) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_pipeline(
        app,
        item,
        cstring(spec),
        concurrency,
        new Uint8Array(out.buffer),
      );
      return readJobPointer(out, code, "pipeline", lastError);
    },
    qrLogin(timeoutSecs) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_qr_login(timeoutSecs, new Uint8Array(out.buffer));
      return readJobPointer(out, code, "qr login", lastError);
    },
    search(
      app,
      text,
      searchIn,
      tags,
      tagGroups,
      excludedTags,
      excludedContent,
      allTags,
      sort,
      trendDays,
      createdSince,
      createdUntil,
      updatedSince,
      updatedUntil,
      limit,
      cursor,
      page,
      countOnly,
    ) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_workshop_search(
        app,
        text === null ? null : cstring(text),
        searchIn === null ? null : cstring(searchIn),
        tags === null ? null : cstring(tags),
        tagGroups === null ? null : cstring(tagGroups),
        excludedTags === null ? null : cstring(excludedTags),
        excludedContent === null ? null : cstring(excludedContent),
        allTags,
        sort === null ? null : cstring(sort),
        trendDays,
        createdSince,
        createdUntil,
        updatedSince,
        updatedUntil,
        limit,
        cursor === null ? null : cstring(cursor),
        page,
        countOnly,
        new Uint8Array(out.buffer),
      );
      return readJobPointer(out, code, "workshop search", lastError);
    },
    async next(job, timeoutMs) {
      let buffer = new Uint8Array(INITIAL_BUFFER);
      for (;;) {
        const len = new BigUint64Array(1);
        const code = await lib.symbols.tapline_job_next(
          ptr(job),
          timeoutMs,
          buffer,
          BigInt(buffer.length),
          new Uint8Array(len.buffer),
        );
        const needed = Number(len[0] ?? 0n);
        if (code === BUFFER_TOO_SMALL) {
          buffer = new Uint8Array(needed);
          continue;
        }
        if (code === DONE) return null;
        if (code === TIMEOUT) return "";
        if (code !== OK) throw new Error(`tapline_job_next failed (${code})`);
        return new TextDecoder().decode(buffer.subarray(0, needed));
      }
    },
    cancel(job) {
      lib.symbols.tapline_job_cancel(ptr(job));
    },
    free(job) {
      lib.symbols.tapline_job_free(ptr(job));
    },
    version() {
      const raw = lib.symbols.tapline_version();
      return new Deno.UnsafePointerView(raw).getCString();
    },
    setTotalConcurrency: (chunks) => lib.symbols.tapline_set_total_concurrency(chunks),
    totalConcurrency: () => Number(lib.symbols.tapline_total_concurrency()),
    availableConcurrency: () => Number(lib.symbols.tapline_available_concurrency()),
  };
}

// --- Bun -------------------------------------------------------------------

async function loadBun(path: string): Promise<Ffi> {
  const { dlopen, FFIType, ptr, CString } = await import("bun:ffi");
  const lib = dlopen(path, {
    tapline_install: {
      args: [
        FFIType.u32, FFIType.ptr, FFIType.ptr, FFIType.u32,
        FFIType.u8, FFIType.u8, FFIType.u8, FFIType.u8, FFIType.ptr, FFIType.ptr,
      ],
      returns: FFIType.i32,
    },
    tapline_plan: {
      args: [FFIType.u32, FFIType.ptr, FFIType.ptr, FFIType.u8, FFIType.u8, FFIType.ptr],
      returns: FFIType.i32,
    },
    tapline_workshop_download: {
      args: [
        FFIType.u32, FFIType.u64, FFIType.ptr, FFIType.u32,
        FFIType.u8, FFIType.ptr, FFIType.u8, FFIType.ptr,
      ],
      returns: FFIType.i32,
    },
    tapline_pipeline: {
      args: [FFIType.u32, FFIType.u64, FFIType.ptr, FFIType.u32, FFIType.ptr],
      returns: FFIType.i32,
    },
    tapline_qr_login: { args: [FFIType.u32, FFIType.ptr], returns: FFIType.i32 },
    tapline_workshop_search: {
      args: [
        FFIType.u32, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr, FFIType.ptr,
        FFIType.u8, FFIType.ptr, FFIType.u32,
        FFIType.u32, FFIType.u32, FFIType.u32, FFIType.u32,
        FFIType.u32, FFIType.ptr, FFIType.u32, FFIType.u8, FFIType.ptr,
      ],
      returns: FFIType.i32,
    },
    tapline_job_next: {
      args: [FFIType.ptr, FFIType.u32, FFIType.ptr, FFIType.u64, FFIType.ptr],
      returns: FFIType.i32,
    },
    tapline_job_cancel: { args: [FFIType.ptr], returns: FFIType.void },
    tapline_job_free: { args: [FFIType.ptr], returns: FFIType.void },
    tapline_last_error: {
      args: [FFIType.ptr, FFIType.u64, FFIType.ptr],
      returns: FFIType.i32,
    },
    tapline_version: { args: [], returns: FFIType.ptr },
    tapline_set_total_concurrency: { args: [FFIType.u32], returns: FFIType.i32 },
    tapline_total_concurrency: { args: [], returns: FFIType.u32 },
    tapline_available_concurrency: { args: [], returns: FFIType.u32 },
  });


  // Bun wants pointers as numbers and rejects a BigInt outright — the out
  // parameter hands one back as a BigUint64Array element, so it is converted
  // here rather than in the shared code. Exact: Linux user-space addresses are
  // below 2^47, well inside what a double represents without loss.
  const asBunPointer = (job: bigint) => Number(job);

  const lastError = (): string => {
    const len = new BigUint64Array(1);
    lib.symbols.tapline_last_error(null, 0n, ptr(len));
    const needed = Number(len[0] ?? 0n);
    if (needed === 0) return "";
    const buf = new Uint8Array(needed);
    lib.symbols.tapline_last_error(ptr(buf), BigInt(needed), ptr(len));
    return new TextDecoder().decode(buf);
  };

  return {
    nativeAsync: false,
    lastError,
    install(app, dir, branch, concurrency, os, validate, includeDlc, fileModes, extensions) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_install(
        app, ptr(cstring(dir)), branch === null ? null : ptr(cstring(branch)),
        concurrency, os, validate, includeDlc, fileModes,
        extensions === null ? null : ptr(cstring(extensions)), ptr(out),
      );
      return readJobPointer(out, code, "install", lastError);
    },
    plan(app, dir, branch, os, includeDlc) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_plan(
        app, ptr(cstring(dir)), branch === null ? null : ptr(cstring(branch)),
        os, includeDlc, ptr(out),
      );
      return readJobPointer(out, code, "plan", lastError);
    },
    workshop(app, item, dir, concurrency, flat, extensions, stream) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_workshop_download(
        app, item, ptr(cstring(dir)), concurrency, flat,
        extensions === null ? null : ptr(cstring(extensions)), stream, ptr(out),
      );
      return readJobPointer(out, code, "workshop download", lastError);
    },
    pipeline(app, item, spec, concurrency) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_pipeline(
        app, item, ptr(cstring(spec)), concurrency, ptr(out),
      );
      return readJobPointer(out, code, "pipeline", lastError);
    },
    qrLogin(timeoutSecs) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_qr_login(timeoutSecs, ptr(out));
      return readJobPointer(out, code, "qr login", lastError);
    },
    search(
      app,
      text,
      searchIn,
      tags,
      tagGroups,
      excludedTags,
      excludedContent,
      allTags,
      sort,
      trendDays,
      createdSince,
      createdUntil,
      updatedSince,
      updatedUntil,
      limit,
      cursor,
      page,
      countOnly,
    ) {
      const out = new BigUint64Array(1);
      const code = lib.symbols.tapline_workshop_search(
        app,
        text === null ? null : ptr(cstring(text)),
        searchIn === null ? null : ptr(cstring(searchIn)),
        tags === null ? null : ptr(cstring(tags)),
        tagGroups === null ? null : ptr(cstring(tagGroups)),
        excludedTags === null ? null : ptr(cstring(excludedTags)),
        excludedContent === null ? null : ptr(cstring(excludedContent)),
        allTags,
        sort === null ? null : ptr(cstring(sort)),
        trendDays,
        createdSince,
        createdUntil,
        updatedSince,
        updatedUntil,
        limit,
        cursor === null ? null : ptr(cstring(cursor)),
        page,
        countOnly,
        ptr(out),
      );
      return readJobPointer(out, code, "workshop search", lastError);
    },
    async next(job, timeoutMs) {
      let buffer = new Uint8Array(INITIAL_BUFFER);
      const deadline = Date.now() + timeoutMs;
      for (;;) {
        const len = new BigUint64Array(1);
        const code = lib.symbols.tapline_job_next(
          asBunPointer(job), 0, ptr(buffer), BigInt(buffer.length), ptr(len),
        );
        const needed = Number(len[0] ?? 0n);
        if (code === BUFFER_TOO_SMALL) {
          buffer = new Uint8Array(needed);
          continue;
        }
        if (code === DONE) return null;
        if (code === OK) {
          return new TextDecoder().decode(buffer.subarray(0, needed));
        }
        if (code !== TIMEOUT) throw new Error(`tapline_job_next failed (${code})`);
        if (Date.now() >= deadline) return "";
        await new Promise((resolve) => setTimeout(resolve, POLL_MS));
      }
    },
    cancel(job) {
      lib.symbols.tapline_job_cancel(asBunPointer(job));
    },
    free(job) {
      lib.symbols.tapline_job_free(asBunPointer(job));
    },
    version() {
      return new CString(lib.symbols.tapline_version()).toString();
    },
    setTotalConcurrency: (chunks) => lib.symbols.tapline_set_total_concurrency(chunks),
    totalConcurrency: () => Number(lib.symbols.tapline_total_concurrency()),
    availableConcurrency: () => Number(lib.symbols.tapline_available_concurrency()),
  };
}

// --- Node ------------------------------------------------------------------

async function loadNode(path: string): Promise<Ffi> {
  // Node 26.1 has an FFI of its own. It needs --experimental-ffi, so it is not
  // always there even on a new enough Node, and koffi remains the fallback.
  try {
    return await loadNodeBuiltin(path);
  } catch (cause) {
    if (cause instanceof Error && cause.message.includes("tapline")) throw cause;
    // Anything else — no node:ffi, no flag — means try koffi.
  }

  let koffi: typeof import("koffi");
  try {
    koffi = await import("koffi");
  } catch {
    throw new Error(
      "Node needs an FFI: either run with `--experimental-ffi` on Node 26.1 or " +
        "newer, or `npm install koffi`. Deno and Bun need neither.",
    );
  }

  const lib = koffi.load(path);
  const install = lib.func(
    "int tapline_install(uint32_t, const char*, const char*, uint32_t, uint8_t, uint8_t, uint8_t, uint8_t, const char*, _Out_ void**)",
  );
  const planFn = lib.func(
    "int tapline_plan(uint32_t, const char*, const char*, uint8_t, uint8_t, _Out_ void**)",
  );
  const workshop = lib.func(
    "int tapline_workshop_download(uint32_t, uint64_t, const char*, uint32_t, uint8_t, const char*, uint8_t, _Out_ void**)",
  );
  const pipelineFn = lib.func(
    "int tapline_pipeline(uint32_t, uint64_t, const char*, uint32_t, _Out_ void**)",
  );
  const qrLoginFn = lib.func("int tapline_qr_login(uint32_t, _Out_ void**)");
  const searchFn = lib.func(
    "int tapline_workshop_search(uint32_t, const char*, const char*, const char*, const char*, const char*, const char*, uint8_t, const char*, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, uint32_t, const char*, uint32_t, uint8_t, _Out_ void**)",
  );
  const next = lib.func(
    "int tapline_job_next(void*, uint32_t, _Out_ uint8_t*, size_t, _Out_ size_t*)",
  );
  const cancel = lib.func("void tapline_job_cancel(void*)");
  const free = lib.func("void tapline_job_free(void*)");
  const version = lib.func("const char* tapline_version()");
  const lastErrorFn = lib.func("int tapline_last_error(_Out_ uint8_t*, size_t, _Out_ size_t*)");
  const setTotal = lib.func("int tapline_set_total_concurrency(uint32_t)");
  const total = lib.func("uint32_t tapline_total_concurrency()");
  const available = lib.func("uint32_t tapline_available_concurrency()");

  const asPointer = (out: unknown[]): bigint => {
    const value = out[0];
    return typeof value === "bigint" ? value : BigInt(koffi.address(value as never));
  };

  const lastError = (): string => {
    const len = [0];
    lastErrorFn(null, 0, len);
    const needed = Number(len[0] ?? 0);
    if (needed === 0) return "";
    const buf = Buffer.alloc(needed);
    lastErrorFn(buf, needed, len);
    return buf.toString("utf8");
  };

  return {
    nativeAsync: true,
    lastError,
    install(app, dir, branch, concurrency, os, validate, includeDlc, fileModes, extensions) {
      const out: unknown[] = [null];
      const code = install(
        app, dir, branch, concurrency, os, validate, includeDlc, fileModes, extensions, out,
      );
      if (code !== OK) throw new Error(`install: ${lastError() || `code ${code}`}`);
      return asPointer(out);
    },
    plan(app, dir, branch, os, includeDlc) {
      const out: unknown[] = [null];
      const code = planFn(app, dir, branch, os, includeDlc, out);
      if (code !== OK) throw new Error(`plan: ${lastError() || `code ${code}`}`);
      return asPointer(out);
    },
    workshop(app, item, dir, concurrency, flat, extensions, stream) {
      const out: unknown[] = [null];
      const code = workshop(app, item, dir, concurrency, flat, extensions, stream, out);
      if (code !== OK) {
        throw new Error(`workshop download: ${lastError() || `code ${code}`}`);
      }
      return asPointer(out);
    },
    pipeline(app, item, spec, concurrency) {
      const out: unknown[] = [null];
      const code = pipelineFn(app, item, spec, concurrency, out);
      if (code !== OK) throw new Error(`pipeline: ${lastError() || `code ${code}`}`);
      return asPointer(out);
    },
    qrLogin(timeoutSecs) {
      const out: unknown[] = [null];
      const code = qrLoginFn(timeoutSecs, out);
      if (code !== OK) throw new Error(`qr login: ${lastError() || `code ${code}`}`);
      return asPointer(out);
    },
    search(
      app,
      text,
      searchIn,
      tags,
      tagGroups,
      excludedTags,
      excludedContent,
      allTags,
      sort,
      trendDays,
      createdSince,
      createdUntil,
      updatedSince,
      updatedUntil,
      limit,
      cursor,
      page,
      countOnly,
    ) {
      const out: unknown[] = [null];
      const code = searchFn(
        app,
        text,
        searchIn,
        tags,
        tagGroups,
        excludedTags,
        excludedContent,
        allTags,
        sort,
        trendDays,
        createdSince,
        createdUntil,
        updatedSince,
        updatedUntil,
        limit,
        cursor,
        page,
        countOnly,
        out,
      );
      if (code !== OK) {
        throw new Error(`workshop search: ${lastError() || `code ${code}`}`);
      }
      return asPointer(out);
    },
    next(job, timeoutMs) {
      return new Promise((resolve, reject) => {
        const buffer = Buffer.alloc(INITIAL_BUFFER);
        const len = [0];
        next.async(
          job,
          timeoutMs,
          buffer,
          buffer.length,
          len,
          (error: Error | null, code: number) => {
            if (error) return reject(error);
            const needed = Number(len[0] ?? 0);
            if (code === BUFFER_TOO_SMALL) {
              const bigger = Buffer.alloc(needed);
              return next.async(
                job, 0, bigger, bigger.length, len,
                (err2: Error | null, code2: number) => {
                  if (err2) return reject(err2);
                  if (code2 === DONE) return resolve(null);
                  if (code2 === TIMEOUT) return resolve("");
                  if (code2 !== OK) {
                    return reject(new Error(`tapline_job_next failed (${code2})`));
                  }
                  resolve(bigger.subarray(0, Number(len[0] ?? 0)).toString("utf8"));
                },
              );
            }
            if (code === DONE) return resolve(null);
            if (code === TIMEOUT) return resolve("");
            if (code !== OK) return reject(new Error(`tapline_job_next failed (${code})`));
            resolve(buffer.subarray(0, needed).toString("utf8"));
          },
        );
      });
    },
    cancel(job) {
      cancel(job);
    },
    free(job) {
      free(job);
    },
    version() {
      return version();
    },
    setTotalConcurrency: (chunks) => setTotal(chunks),
    totalConcurrency: () => Number(total()),
    availableConcurrency: () => Number(available()),
  };
}

/**
 * Node's own FFI, from 26.1 behind `--experimental-ffi`.
 *
 * Synchronous only — there is no equivalent of koffi's `.async()` — so events
 * are polled with a zero timeout and a sleep between, exactly as Bun is. A
 * blocking call here would hold the event loop for the whole timeout.
 */
async function loadNodeBuiltin(path: string): Promise<Ffi> {
  const ffi = await import("node:ffi");
  const { dlopen, toString: ptrToString, toBuffer } = ffi as unknown as {
    dlopen: (
      path: string,
      // deno-lint-ignore no-explicit-any
      symbols: Record<string, any>,
    ) => { functions: Record<string, (...args: unknown[]) => number | bigint> };
    toString: (pointer: unknown) => string;
    toBuffer: (pointer: unknown, length: number, copy: boolean) => Uint8Array;
  };

  const u32 = "uint32", u8 = "uint8", u64 = "uint64";
  const ptr = "pointer", str = "string", buf = "buffer";
  const { functions } = dlopen(path, {
    tapline_install: {
      arguments: [u32, str, str, u32, u8, u8, u8, u8, str, buf],
      return: "int32",
    },
    tapline_plan: { arguments: [u32, str, str, u8, u8, buf], return: "int32" },
    tapline_workshop_download: {
      arguments: [u32, u64, str, u32, u8, str, u8, buf],
      return: "int32",
    },
    tapline_workshop_search: {
      arguments: [
        u32, str, str, str, str, str, str, u8, str,
        u32, u32, u32, u32, u32, u32, str, u32, u8, buf,
      ],
      return: "int32",
    },
    tapline_pipeline: { arguments: [u32, u64, str, u32, buf], return: "int32" },
    tapline_qr_login: { arguments: [u32, buf], return: "int32" },
    tapline_job_next: {
      arguments: [ptr, u32, buf, "uint64", buf],
      return: "int32",
    },
    tapline_job_cancel: { arguments: [ptr], return: "void" },
    tapline_job_free: { arguments: [ptr], return: "void" },
    tapline_version: { arguments: [], return: ptr },
    tapline_last_error: { arguments: [buf, "uint64", buf], return: "int32" },
    tapline_set_total_concurrency: { arguments: [u32], return: "int32" },
    tapline_total_concurrency: { arguments: [], return: u32 },
    tapline_available_concurrency: { arguments: [], return: u32 },
  });

  const lastError = (): string => {
    const len = new BigUint64Array(1);
    functions.tapline_last_error(null, 0n, new Uint8Array(len.buffer));
    const needed = Number(len[0] ?? 0n);
    if (needed === 0) return "";
    const out = new Uint8Array(needed);
    functions.tapline_last_error(out, BigInt(needed), new Uint8Array(len.buffer));
    return new TextDecoder().decode(out);
  };

  // The job handle comes back through an out pointer; read it as a u64 and
  // hand it back as one, the same shape the other two backends use.
  const jobOut = (): { slot: Uint8Array; read: () => bigint } => {
    const raw = new BigUint64Array(1);
    const slot = new Uint8Array(raw.buffer);
    return { slot, read: () => raw[0] ?? 0n };
  };
  const started = (code: number, what: string, read: () => bigint): bigint => {
    if (code !== OK) throw new Error(`${what}: ${lastError() || `code ${code}`}`);
    return read();
  };

  return {
    nativeAsync: false,
    lastError,
    install(app, dir, branch, concurrency, os, validate, includeDlc, fileModes, extensions) {
      const out = jobOut();
      const code = Number(functions.tapline_install(
        app, dir, branch, concurrency, os, validate, includeDlc, fileModes,
        extensions, out.slot,
      ));
      return started(code, "install", out.read);
    },
    plan(app, dir, branch, os, includeDlc) {
      const out = jobOut();
      const code = Number(functions.tapline_plan(app, dir, branch, os, includeDlc, out.slot));
      return started(code, "plan", out.read);
    },
    workshop(app, item, dir, concurrency, flat, extensions, stream) {
      const out = jobOut();
      const code = Number(functions.tapline_workshop_download(
        app, item, dir, concurrency, flat, extensions, stream, out.slot,
      ));
      return started(code, "workshop download", out.read);
    },
    search(
      app, text, searchIn, tags, tagGroups, excludedTags, excludedContent,
      allTags, sort, trendDays, createdSince, createdUntil, updatedSince,
      updatedUntil, limit, cursor, page, countOnly,
    ) {
      const out = jobOut();
      const code = Number(functions.tapline_workshop_search(
        app, text, searchIn, tags, tagGroups, excludedTags, excludedContent,
        allTags, sort, trendDays, createdSince, createdUntil, updatedSince,
        updatedUntil, limit, cursor, page, countOnly, out.slot,
      ));
      return started(code, "workshop search", out.read);
    },
    qrLogin(timeoutSecs) {
      const out = jobOut();
      const code = Number(functions.tapline_qr_login(timeoutSecs, out.slot));
      return started(code, "qr login", out.read);
    },
    pipeline(app, item, spec, concurrency) {
      const out = jobOut();
      const code = Number(functions.tapline_pipeline(app, item, spec, concurrency, out.slot));
      return started(code, "pipeline", out.read);
    },
    async next(job, timeoutMs) {
      let buffer = new Uint8Array(INITIAL_BUFFER);
      const deadline = Date.now() + timeoutMs;
      for (;;) {
        const len = new BigUint64Array(1);
        // Zero timeout: this call is synchronous, and blocking in it would
        // hold the event loop for the whole wait.
        const code = Number(functions.tapline_job_next(
          job, 0, buffer, BigInt(buffer.length), new Uint8Array(len.buffer),
        ));
        const needed = Number(len[0] ?? 0n);
        if (code === BUFFER_TOO_SMALL) {
          buffer = new Uint8Array(needed);
          continue;
        }
        if (code === DONE) return null;
        if (code === OK) return new TextDecoder().decode(buffer.subarray(0, needed));
        if (code !== TIMEOUT) throw new Error(`tapline_job_next failed (${code})`);
        if (Date.now() >= deadline) return "";
        await new Promise((resolve) => setTimeout(resolve, POLL_MS));
      }
    },
    cancel(job) {
      functions.tapline_job_cancel(job);
    },
    free(job) {
      functions.tapline_job_free(job);
    },
    version() {
      return ptrToString(functions.tapline_version());
    },
    setTotalConcurrency(chunks) {
      return Number(functions.tapline_set_total_concurrency(chunks));
    },
    totalConcurrency() {
      return Number(functions.tapline_total_concurrency());
    },
    availableConcurrency() {
      return Number(functions.tapline_available_concurrency());
    },
    // Only used by the koffi path, which needs it to keep a Buffer alive.
    _toBuffer: toBuffer,
  } as unknown as Ffi;
}
