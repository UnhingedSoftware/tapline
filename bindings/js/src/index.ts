/**
 * tapline — install Steam apps and Workshop content from Deno, Bun or Node.
 *
 * ```ts
 * import { install } from "tapline";
 *
 * // Await it.
 * const report = await install({ app: 4020, dir: "/srv/gmod" });
 *
 * // Or watch it.
 * await install({
 *   app: 4020,
 *   dir: "/srv/gmod",
 *   onProgress: (p) => console.log(`${p.percent.toFixed(1)}%`),
 * });
 *
 * // Or iterate it.
 * for await (const event of install({ app: 4020, dir: "/srv/gmod" })) {
 *   if (event.kind === "fileCompleted") console.log(event.path);
 * }
 *
 * // Or stop it.
 * const job = install({ app: 4020, dir: "/srv/gmod" });
 * setTimeout(() => job.cancel(), 5_000);
 * ```
 *
 * The same object is awaitable, iterable and cancellable, so none of those is a
 * different function to learn.
 */

import { type Ffi, load } from "./ffi.ts";
import type {
  InstallOptions,
  InstallReport,
  PipeReport,
  ResultEvent,
  SearchedEvent,
  PlanOptions,
  PlanReport,
  StreamReport,
  TaplineEvent,
  TargetOs,
  WorkshopOptions,
} from "./types.ts";

export * from "./types.ts";

/** How long a single wait for an event blocks before looping. */
const POLL_TIMEOUT_MS = 250;

let cached: Promise<Ffi> | undefined;

/** Opens the shared library, once per process. */
function library(): Promise<Ffi> {
  cached ??= load();
  return cached;
}

/** The version of the native library. */
export async function version(): Promise<string> {
  return (await library()).version();
}

/**
 * Sets how many chunks may be in flight across *all* downloads in this process.
 *
 * Downloads share one budget rather than taking one each. Two installs at 64
 * chunks each is measurably slower than two splitting 64, because throughput
 * turns over past 64 — and sharing also lets one download use connections the
 * other already warmed.
 *
 * Must be called before the first job starts; after that the budget is fixed,
 * because moving it underneath running downloads is not something a caller can
 * reason about. Throws if it is already in use.
 */
export async function setTotalConcurrency(chunks: number): Promise<void> {
  const code = (await library()).setTotalConcurrency(chunks);
  if (code !== 0) {
    throw new Error(
      "the concurrency budget is already in use; set it before starting any download",
    );
  }
}

/** How the shared budget currently stands. */
export async function concurrency(): Promise<{
  total: number;
  available: number;
}> {
  const ffi = await library();
  return { total: ffi.totalConcurrency(), available: ffi.availableConcurrency() };
}

function osCode(os: TargetOs | undefined): number {
  switch (os) {
    case "linux":
      return 1;
    case "windows":
      return 2;
    case "macos":
      return 3;
    default:
      return 0;
  }
}

/**
 * A running job.
 *
 * Awaitable for the result, async-iterable for the events, and cancellable at
 * any point. Iterating and awaiting the same job both work — the events are
 * buffered for the iterator either way, so attaching one late does not lose
 * what already happened.
 */
export class Job<T> implements PromiseLike<T>, AsyncIterable<TaplineEvent> {
  #ffi: Ffi | undefined;
  #pointer: bigint | undefined;
  #cancelled = false;
  #done = false;
  #buffered: TaplineEvent[] = [];
  #waiters: (() => void)[] = [];
  #result: Promise<T>;

  constructor(
    start: (ffi: Ffi) => bigint,
    finish: (events: TaplineEvent[]) => T,
    onEvent?: (event: TaplineEvent) => void,
  ) {
    this.#result = this.#run(start, finish, onEvent);
    // A job starts working the moment it is constructed, so a caller who only
    // wants the events — or who cancels one and walks away — would otherwise
    // leave a rejected promise nobody handled. In Node that is a process-level
    // crash by default, in someone else's server, for a job they deliberately
    // stopped. Marking it handled here does not hide anything: `then`,
    // `callback` and the iterator all still receive the failure, because
    // attaching a handler to a promise does not consume its rejection.
    this.#result.catch(() => {});
  }

  async #run(
    start: (ffi: Ffi) => bigint,
    finish: (events: TaplineEvent[]) => T,
    onEvent?: (event: TaplineEvent) => void,
  ): Promise<T> {
    const ffi = await library();
    this.#ffi = ffi;

    if (this.#cancelled) {
      throw new Error("the job was cancelled before it started");
    }

    const pointer = start(ffi);
    this.#pointer = pointer;
    const collected: TaplineEvent[] = [];

    try {
      for (;;) {
        const text = await ffi.next(pointer, POLL_TIMEOUT_MS);
        if (text === null) break;
        // An empty string is a timeout, not an event: nothing happened yet.
        if (text === "") {
          if (this.#cancelled) break;
          continue;
        }

        let event: TaplineEvent;
        try {
          event = JSON.parse(text) as TaplineEvent;
        } catch (cause) {
          throw new Error(`tapline emitted invalid JSON: ${text}`, { cause });
        }

        collected.push(event);
        this.#buffered.push(event);
        this.#wake();
        // A throwing listener must not strand the job or leak the library
        // handle, so it surfaces as the job's failure rather than an unhandled
        // rejection somewhere else.
        onEvent?.(event);

        if (event.kind === "error") {
          throw new Error(event.message);
        }
      }

      if (this.#cancelled) {
        throw new Error("the job was cancelled");
      }
      return finish(collected);
    } finally {
      this.#done = true;
      this.#wake();
      ffi.free(pointer);
      this.#pointer = undefined;
    }
  }

  #wake(): void {
    const waiters = this.#waiters;
    this.#waiters = [];
    for (const wake of waiters) wake();
  }

  /**
   * Stops the job.
   *
   * Whatever is already on disk stays there, and a later install resumes from
   * it rather than starting over.
   */
  cancel(): void {
    this.#cancelled = true;
    if (this.#ffi && this.#pointer !== undefined) {
      this.#ffi.cancel(this.#pointer);
    }
    this.#wake();
  }

  /** Whether {@link cancel} has been called. */
  get cancelled(): boolean {
    return this.#cancelled;
  }

  then<A = T, B = never>(
    onFulfilled?: ((value: T) => A | PromiseLike<A>) | null,
    onRejected?: ((reason: unknown) => B | PromiseLike<B>) | null,
  ): PromiseLike<A | B> {
    return this.#result.then(onFulfilled, onRejected);
  }

  /** Node-style callback, for code that prefers one. */
  callback(done: (error: Error | null, value?: T) => void): this {
    this.#result.then(
      (value) => done(null, value),
      (error: unknown) =>
        done(error instanceof Error ? error : new Error(String(error))),
    );
    return this;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<TaplineEvent> {
    // Swallowed here on purpose: a caller iterating events sees the `error`
    // event go past and can act on it, and the same failure is still delivered
    // to anyone awaiting the job. Rethrowing here as well would produce an
    // unhandled rejection for the awaiter that never got there.
    this.#result.catch(() => {});
    for (;;) {
      while (this.#buffered.length > 0) {
        const event = this.#buffered.shift();
        if (event) yield event;
      }
      if (this.#done) return;
      await new Promise<void>((resolve) => this.#waiters.push(resolve));
    }
  }
}

function progressBridge(
  onEvent: ((event: TaplineEvent) => void) | undefined,
  onProgress: InstallOptions["onProgress"],
): ((event: TaplineEvent) => void) | undefined {
  if (!onProgress) return onEvent;
  return (event) => {
    onEvent?.(event);
    if (event.kind === "progress") {
      onProgress({
        bytesDone: event.bytesDone,
        bytesTotal: event.bytesTotal,
        // Guarded: a depot of zero bytes is legal and would otherwise produce
        // NaN in someone's progress bar.
        percent:
          event.bytesTotal === 0
            ? 100
            : (event.bytesDone / event.bytesTotal) * 100,
      });
    }
  };
}

function lastOfKind<K extends TaplineEvent["kind"]>(
  events: TaplineEvent[],
  kind: K,
): Extract<TaplineEvent, { kind: K }> {
  for (let i = events.length - 1; i >= 0; i -= 1) {
    const event = events[i];
    if (event?.kind === kind) {
      return event as Extract<TaplineEvent, { kind: K }>;
    }
  }
  throw new Error(`the job ended without a ${kind} event`);
}

/** Installs or updates an app. */
export function install(options: InstallOptions): Job<InstallReport> {
  return new Job<InstallReport>(
    (ffi) =>
      ffi.install(
        options.app,
        options.dir,
        options.branch ?? null,
        options.concurrency ?? 0,
        osCode(options.os),
        options.validate ? 1 : 0,
        options.includeDlc ? 1 : 0,
        options.fileModes === "manifest" ? 1 : 0,
        options.extensions?.length ? options.extensions.join(",") : null,
      ),
    (events) => {
      const { kind: _kind, ...report } = lastOfKind(events, "finished");
      return report;
    },
    progressBridge(options.onEvent, options.onProgress),
  );
}

/**
 * Works out what an install would cost, without fetching any content.
 *
 * Worth calling before committing disk: an update that looks like 20 GB is
 * often 200 MB of changed chunks, and this answers that without downloading.
 */
export function plan(options: PlanOptions): Job<PlanReport> {
  return new Job<PlanReport>(
    (ffi) =>
      ffi.plan(
        options.app,
        options.dir,
        options.branch ?? null,
        osCode(options.os),
        options.includeDlc ? 1 : 0,
      ),
    (events) => {
      const { kind: _kind, ...report } = lastOfKind(events, "planned");
      return report;
    },
  );
}

/**
 * Runs many installs, at most `maxConcurrent` at a time.
 *
 * Which number to pick is a real trade-off, not a tuning detail, because the
 * chunk budget is shared. Three concurrent Valheim installs measured against
 * three sequential ones:
 *
 * | | first ready | all ready |
 * |---|---|---|
 * | all three at once | 18.0 s | **18.7 s** |
 * | one at a time | **9.5 s** | 27.2 s |
 *
 * Running them together finishes the batch sooner. Running them one at a time
 * gets the first server online in less than half the time. A provisioning tool
 * usually wants the second; a nightly sync wants the first.
 *
 * Defaults to all at once, which is the throughput-optimal choice and what
 * `Promise.all` over `install` would already do.
 *
 * Results come back in the order the specs were given, not the order they
 * finished.
 */
export function installAll(
  specs: InstallOptions[],
  options: {
    /** How many may run at once. Defaults to all of them. */
    maxConcurrent?: number;
    /** Called as each one finishes, in completion order. */
    onEach?: (report: InstallReport, index: number) => void;
  } = {},
): Batch {
  return new Batch(specs, options.maxConcurrent ?? specs.length, options.onEach);
}

/**
 * A set of installs running under a concurrency limit.
 *
 * Awaitable for all the reports, and cancellable — which cancels the ones
 * running and never starts the ones still queued.
 */
export class Batch implements PromiseLike<InstallReport[]> {
  #jobs: Job<InstallReport>[] = [];
  #cancelled = false;
  #result: Promise<InstallReport[]>;

  constructor(
    specs: InstallOptions[],
    maxConcurrent: number,
    onEach?: (report: InstallReport, index: number) => void,
  ) {
    this.#result = this.#run(specs, Math.max(1, maxConcurrent), onEach);
    // Same reason as Job: a batch nobody awaits must not crash the process.
    this.#result.catch(() => {});
  }

  async #run(
    specs: InstallOptions[],
    maxConcurrent: number,
    onEach?: (report: InstallReport, index: number) => void,
  ): Promise<InstallReport[]> {
    const reports = new Array<InstallReport>(specs.length);
    let next = 0;

    const worker = async (): Promise<void> => {
      for (;;) {
        const index = next;
        next += 1;
        if (index >= specs.length) return;
        if (this.#cancelled) throw new Error("the batch was cancelled");

        const spec = specs[index];
        if (!spec) return;
        const job = install(spec);
        this.#jobs.push(job);
        const report = await job;
        reports[index] = report;
        onEach?.(report, index);
      }
    };

    const workers = Array.from(
      { length: Math.min(maxConcurrent, specs.length) },
      () => worker(),
    );
    await Promise.all(workers);
    return reports;
  }

  /** Cancels everything running, and everything not yet started. */
  cancel(): void {
    this.#cancelled = true;
    for (const job of this.#jobs) job.cancel();
  }

  /** Whether {@link cancel} has been called. */
  get cancelled(): boolean {
    return this.#cancelled;
  }

  then<A = InstallReport[], B = never>(
    onFulfilled?: ((value: InstallReport[]) => A | PromiseLike<A>) | null,
    onRejected?: ((reason: unknown) => B | PromiseLike<B>) | null,
  ): PromiseLike<A | B> {
    return this.#result.then(onFulfilled, onRejected);
  }

  /** Node-style callback, matching {@link Job.callback}. */
  callback(done: (error: Error | null, value?: InstallReport[]) => void): this {
    this.#result.then(
      (value) => done(null, value),
      (error: unknown) =>
        done(error instanceof Error ? error : new Error(String(error))),
    );
    return this;
  }
}

/** Downloads one Workshop item. */
export function downloadWorkshopItem(
  options: WorkshopOptions & { stream: NonNullable<WorkshopOptions["stream"]> },
): Job<StreamReport>;
export function downloadWorkshopItem(
  options: WorkshopOptions,
): Job<InstallReport>;
export function downloadWorkshopItem(
  options: WorkshopOptions,
): Job<InstallReport | StreamReport> {
  const streamMode = ((): number => {
    switch (options.stream) {
      case undefined:
      case false:
        return 0;
      case "zip":
        return 2;
      case "zip-stored":
        return 3;
      default:
        return 1;
    }
  })();
  const streaming = streamMode !== 0;
  return new Job<InstallReport | StreamReport>(
    (ffi) =>
      ffi.workshop(
        options.app,
        BigInt(options.item),
        options.dir,
        options.concurrency ?? 0,
        // Streaming writes into the directory given; there is no archive to
        // build a steamcmd path around.
        streaming || options.layout === "flat" ? 1 : 0,
        options.extensions?.length ? options.extensions.join(",") : null,
        streamMode,
      ),
    (events) => {
      if (streaming) {
        const { kind: _kind, ...report } = lastOfKind(events, "streamed");
        return report;
      }
      const { kind: _kind, ...report } = lastOfKind(events, "finished");
      return report;
    },
    progressBridge(options.onEvent, options.onProgress),
  );
}


/** A pipeline destination. One only — see {@link Decoded.dir}. */
type Sink =
  | { readonly directive: "dir"; readonly path: string }
  | { readonly directive: "zip"; readonly path: string }
  | { readonly directive: "zip-stored"; readonly path: string };

/** What a chain has accumulated. Immutable; every step returns a new one. */
interface Spec {
  readonly app: number;
  readonly item: bigint;
  readonly format: string;
  readonly filters: readonly string[];
  readonly picks: readonly string[];
  readonly concurrency: number;
  readonly onEvent?: (event: TaplineEvent) => void;
  readonly onProgress?: WorkshopOptions["onProgress"];
}

/** Renders a spec as the text form the C ABI takes. */
function toText(spec: Spec, sink: Sink): string {
  const lines = [`decode ${spec.format}`];
  for (const filter of spec.filters) lines.push(`only ${filter}`);
  for (const pick of spec.picks) lines.push(`pick ${pick}`);
  lines.push(`${sink.directive} ${sink.path}`);
  return `${lines.join("\n")}\n`;
}

/**
 * A Workshop item that has been given a format, and can now be narrowed and
 * pointed somewhere.
 */
class Decoded {
  readonly #spec: Spec;

  constructor(spec: Spec) {
    this.#spec = spec;
  }

  /**
   * Takes only entries matching a glob. Repeatable; the matches are a union.
   *
   * Selecting makes the download itself selective — the chunks holding the
   * entries you did not ask for are never fetched, rather than fetched and
   * discarded. A pattern matching nothing is a legitimate answer.
   */
  only(pattern: string): Decoded {
    return new Decoded({
      ...this.#spec,
      filters: [...this.#spec.filters, pattern],
    });
  }

  /**
   * Takes one exact path, whatever the globs say.
   *
   * Unlike {@link Decoded.only}, a path that is not in the archive is an error:
   * you are asserting something about the archive, and running anyway would
   * produce an empty result that looks like success.
   */
  pick(path: string): Decoded {
    return new Decoded({ ...this.#spec, picks: [...this.#spec.picks, path] });
  }

  /** Reports progress while it runs. */
  onProgress(handler: NonNullable<WorkshopOptions["onProgress"]>): Decoded {
    return new Decoded({ ...this.#spec, onProgress: handler });
  }

  /** Receives every event while it runs. */
  onEvent(handler: (event: TaplineEvent) => void): Decoded {
    return new Decoded({ ...this.#spec, onEvent: handler });
  }

  /** Unpacks into a directory. */
  dir(path: string): Job<PipeReport> {
    return this.#run({ directive: "dir", path });
  }

  /** Writes a zip, deflating entries that get smaller for it. */
  zip(path: string): Job<PipeReport> {
    return this.#run({ directive: "zip", path });
  }

  /** Writes a zip without deflating: bigger, and faster to produce. */
  zipStored(path: string): Job<PipeReport> {
    return this.#run({ directive: "zip-stored", path });
  }

  /** The text form this chain compiles to. Exposed for debugging and tests. */
  text(sink: Sink["directive"], path: string): string {
    return toText(this.#spec, { directive: sink, path } as Sink);
  }

  #run(sink: Sink): Job<PipeReport> {
    const spec = this.#spec;
    const text = toText(spec, sink);
    return new Job<PipeReport>(
      (ffi) => ffi.pipeline(spec.app, spec.item, text, spec.concurrency),
      (events) => {
        const { kind: _kind, ...report } = lastOfKind(events, "piped");
        return report;
      },
      progressBridge(spec.onEvent, spec.onProgress),
    );
  }
}

/** A Workshop item, before it has been given a meaning. */
class Source {
  readonly #spec: Spec;

  constructor(spec: Spec) {
    this.#spec = spec;
  }

  /** Reads it as a Garry's Mod addon. */
  gma(): Decoded {
    return new Decoded({ ...this.#spec, format: "gma" });
  }

  /** Reads it as a ZIP. */
  zip(): Decoded {
    return new Decoded({ ...this.#spec, format: "zip" });
  }

  /** Reads it as a named format. */
  decode(format: string): Decoded {
    return new Decoded({ ...this.#spec, format });
  }

  /** How many chunks to hold while reordering. 0 takes the default. */
  window(chunks: number): Source {
    return new Source({ ...this.#spec, concurrency: chunks });
  }
}

/**
 * Starts a pipeline over one Workshop item.
 *
 * The chain mirrors the Rust one and compiles to the same text form, which is
 * what actually crosses the C ABI:
 *
 * ```ts
 * const report = await workshop(4000, 104691717)
 *   .gma()
 *   .only("lua/**")
 *   .zip("/srv/out.zip");
 * ```
 *
 * A stream has one direction, so there is one destination and it ends the
 * chain. Writing the same download to two places would mean buffering for
 * whichever sink is behind, which is a different operation with different
 * costs — not a flag.
 */
export function workshop(app: number, item: number | bigint): Source {
  return new Source({
    app,
    item: BigInt(item),
    format: "gma",
    filters: [],
    picks: [],
    concurrency: 0,
  });
}

/** What to search an app's Workshop for. */
export interface SearchOptions {
  /** Which app's Workshop. */
  app: number;
  /** Free text to match. */
  text?: string;
  /** Tags an item must carry. */
  tags?: string[];
  /** Tags that exclude an item. */
  excludeTags?: string[];
  /** Require every tag rather than any of them. */
  allTags?: boolean;
  /**
   * How to order results: `vote`, `recent`, `updated`, `trend`, `subscribed`
   * or `text`. `text` needs {@link SearchOptions.text} — without it Steam
   * returns an arbitrary order that looks like a ranking, so it is refused.
   */
  sort?: "vote" | "recent" | "updated" | "trend" | "subscribed" | "text";
  /** How many to return. Capped at 100, because Steam silently returns fewer. */
  limit?: number;
  /** {@link SearchPage.nextCursor} from a previous page. */
  cursor?: string;
  onEvent?: (event: TaplineEvent) => void;
}

/** A page of search results. */
export interface SearchPage {
  items: Omit<ResultEvent, "kind">[];
  /** How many the whole search matched, usually far more than one page. */
  total: number;
  /** Items Steam returned that could not be described. */
  skipped: number;
  /** Pass to {@link SearchOptions.cursor} for the next page, or null at the end. */
  nextCursor: string | null;
}

/**
 * Searches an app's Workshop.
 *
 * Works on an anonymous session — no key, no login:
 *
 * ```ts
 * const page = await searchWorkshop({ app: 4000, text: "stargate", sort: "text" });
 * for (const found of page.items) console.log(found.item, found.title);
 * ```
 *
 * Each result carries everything {@link downloadWorkshopItem} needs, so a
 * search feeds a download with no second lookup:
 *
 * ```ts
 * await downloadWorkshopItem({ app: 4000, item: page.items[0].item, dir });
 * ```
 *
 * Paging is a cursor rather than a page number, because offsets repeat items
 * past about a thousand results. Walk it until `nextCursor` is null.
 */
export function searchWorkshop(options: SearchOptions): Job<SearchPage> {
  return new Job<SearchPage>(
    (ffi) =>
      ffi.search(
        options.app,
        options.text ?? null,
        options.tags?.length ? options.tags.join(",") : null,
        options.excludeTags?.length ? options.excludeTags.join(",") : null,
        options.allTags ? 1 : 0,
        options.sort ?? null,
        options.limit ?? 0,
        options.cursor ?? null,
      ),
    (events) => {
      const summary = lastOfKind(events, "searched") as SearchedEvent;
      const items = events
        .filter((event): event is ResultEvent => event.kind === "result")
        .map(({ kind: _kind, ...rest }) => rest);
      return {
        items,
        total: summary.total,
        skipped: summary.skipped,
        // Empty is "no next page"; null says that plainly rather than making
        // every caller check for "".
        nextCursor: summary.nextCursor === "" ? null : summary.nextCursor,
      };
    },
    options.onEvent ? progressBridge(options.onEvent, undefined) : undefined,
  );
}
