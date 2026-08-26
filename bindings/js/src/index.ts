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
  PlanOptions,
  PlanReport,
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

/** Downloads one Workshop item. */
export function downloadWorkshopItem(
  options: WorkshopOptions,
): Job<InstallReport> {
  return new Job<InstallReport>(
    (ffi) =>
      ffi.workshop(
        options.app,
        BigInt(options.item),
        options.dir,
        options.concurrency ?? 0,
      ),
    (events) => {
      const { kind: _kind, ...report } = lastOfKind(events, "finished");
      return report;
    },
    progressBridge(options.onEvent, options.onProgress),
  );
}
