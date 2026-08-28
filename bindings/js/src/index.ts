/** tapline — install Steam apps and Workshop content from Deno, Bun or Node. */

import { type Ffi, load } from "./ffi.ts";
import type {
  CountedEvent,
  InstallOptions,
  LoggedInEvent,
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

const POLL_TIMEOUT_MS = 250;

let cached: Promise<Ffi> | undefined;

function library(): Promise<Ffi> {
  cached ??= load();
  return cached;
}

/** The version of the native library. */
export async function version(): Promise<string> {
  return (await library()).version();
}

/** Sets the process-wide chunk budget; must be called before the first job. */
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

/** A running job: awaitable, async-iterable, and cancellable. */
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
    // Mark handled: an unawaited or cancelled job must not crash the process.
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
        // A throwing listener surfaces as the job's failure.
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

  /** Stops the job; whatever is already on disk stays there. */
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
    // Swallowed on purpose: the failure still reaches anyone awaiting the job.
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
        // A zero-byte depot is legal and would otherwise produce NaN.
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

/** Works out what an install would cost, without fetching any content. */
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

/** Runs many installs, at most `maxConcurrent` at a time. */
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

/** A set of installs running under a concurrency limit. */
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
        // Streaming implies the flat layout.
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


type Sink =
  | { readonly directive: "dir"; readonly path: string }
  | { readonly directive: "zip"; readonly path: string }
  | { readonly directive: "zip-stored"; readonly path: string };

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

function toText(spec: Spec, sink: Sink): string {
  const lines = [`decode ${spec.format}`];
  for (const filter of spec.filters) lines.push(`only ${filter}`);
  for (const pick of spec.picks) lines.push(`pick ${pick}`);
  lines.push(`${sink.directive} ${sink.path}`);
  return `${lines.join("\n")}\n`;
}

class Decoded {
  readonly #spec: Spec;

  constructor(spec: Spec) {
    this.#spec = spec;
  }

  /** Takes only entries matching a glob; repeatable, a union. */
  only(pattern: string): Decoded {
    return new Decoded({
      ...this.#spec,
      filters: [...this.#spec.filters, pattern],
    });
  }

  /** Takes one exact path; missing it is an error, unlike a glob. */
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

/** Starts a pipeline over one Workshop item. */
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
  app: number;
  text?: string;
  /** Where `text` is matched; Steam searches title and description by default. */
  searchIn?: "all" | "title" | "description";
  tags?: string[];
  /** Steam's sidebar groups: one tag required from each group. */
  tagGroups?: string[][];
  excludeTags?: string[];
  /** Steam's own content labels to leave out. */
  excludeContent?: ("nudity" | "violence" | "adult-only" | "gratuitous" | "mature")[];
  /** Require every tag rather than any of them. Applies to `tags` only. */
  allTags?: boolean;
  /** How to order results; `"text"` needs `text` to be set. */
  sort?: "vote" | "recent" | "updated" | "trend" | "subscribed" | "text";
  /** Days a `trend` ranking covers; refused with any other sort. */
  days?: number;
  /** Only items first published within this window. Unix seconds. */
  created?: { since?: number; until?: number };
  /** Only items last updated within this window. Unix seconds. */
  updated?: { since?: number; until?: number };
  /** How many to return. Capped at 100, because Steam silently returns fewer. */
  limit?: number;
  /** {@link SearchPage.nextCursor} from a previous page. */
  cursor?: string;
  /** Jump straight to a numbered page, 1-based; cannot be combined with `cursor`. */
  page?: number;
  onEvent?: (event: TaplineEvent) => void;
}

/** A page of search results. */
export interface SearchPage {
  items: Omit<ResultEvent, "kind">[];
  total: number;
  skipped: number;
  /** Pass to {@link SearchOptions.cursor} for the next page, or null at the end. */
  nextCursor: string | null;
}

/** Searches an app's Workshop; page with `cursor` until `nextCursor` is null. */
export function searchWorkshop(options: SearchOptions): Job<SearchPage> {
  return new Job<SearchPage>(
    (ffi) =>
      ffi.search(
        options.app,
        options.text ?? null,
        options.searchIn ?? null,
        options.tags?.length ? options.tags.join(",") : null,
        options.tagGroups?.length
          ? options.tagGroups.map((group) => group.join(",")).join(";")
          : null,
        options.excludeTags?.length ? options.excludeTags.join(",") : null,
        options.excludeContent?.length ? options.excludeContent.join(",") : null,
        options.allTags ? 1 : 0,
        options.sort ?? null,
        options.days ?? 0,
        options.created?.since ?? 0,
        options.created?.until ?? 0,
        options.updated?.since ?? 0,
        options.updated?.until ?? 0,
        options.limit ?? 0,
        options.cursor ?? null,
        options.page ?? 0,
        0,
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
        // Empty means no next page; null says that plainly.
        nextCursor: summary.nextCursor === "" ? null : summary.nextCursor,
      };
    },
    options.onEvent ? progressBridge(options.onEvent, undefined) : undefined,
  );
}

/** Counts what a search would match, fetching none of it. */
export function countWorkshop(options: SearchOptions): Job<number> {
  return new Job<number>(
    (ffi) =>
      ffi.search(
        options.app,
        options.text ?? null,
        options.searchIn ?? null,
        options.tags?.length ? options.tags.join(",") : null,
        options.tagGroups?.length
          ? options.tagGroups.map((group) => group.join(",")).join(";")
          : null,
        options.excludeTags?.length ? options.excludeTags.join(",") : null,
        options.excludeContent?.length ? options.excludeContent.join(",") : null,
        options.allTags ? 1 : 0,
        options.sort ?? null,
        options.days ?? 0,
        options.created?.since ?? 0,
        options.created?.until ?? 0,
        options.updated?.since ?? 0,
        options.updated?.until ?? 0,
        0,
        null,
        0,
        1,
      ),
    (events) => (lastOfKind(events, "counted") as CountedEvent).total,
    options.onEvent ? progressBridge(options.onEvent, undefined) : undefined,
  );
}

/** How a QR login reports itself. */
export interface QrLoginOptions {
  /** Called with the QR URL, again each time Steam rotates it. */
  onCode: (url: string) => void;
  /** How long to wait for approval, in seconds. Defaults to 300. */
  timeoutSeconds?: number;
}

/** Signs in with a QR code, handling the refresh; the token is saved. */
export function qrLogin(options: QrLoginOptions): Job<{ account: string }> {
  const onEvent = (event: TaplineEvent) => {
    if (event.kind === "qr") options.onCode(event.url);
  };
  return new Job<{ account: string }>(
    (ffi) => ffi.qrLogin(options.timeoutSeconds ?? 0),
    (events) => {
      const done = lastOfKind(events, "loggedIn") as LoggedInEvent;
      return { account: done.account };
    },
    onEvent,
  );
}
