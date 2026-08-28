
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

export async function version(): Promise<string> {
  return (await library()).version();
}

export async function setTotalConcurrency(chunks: number): Promise<void> {
  const code = (await library()).setTotalConcurrency(chunks);
  if (code !== 0) {
    throw new Error(
      "the concurrency budget is already in use; set it before starting any download",
    );
  }
}

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

  cancel(): void {
    this.#cancelled = true;
    if (this.#ffi && this.#pointer !== undefined) {
      this.#ffi.cancel(this.#pointer);
    }
    this.#wake();
  }

  get cancelled(): boolean {
    return this.#cancelled;
  }

  then<A = T, B = never>(
    onFulfilled?: ((value: T) => A | PromiseLike<A>) | null,
    onRejected?: ((reason: unknown) => B | PromiseLike<B>) | null,
  ): PromiseLike<A | B> {
    return this.#result.then(onFulfilled, onRejected);
  }

  callback(done: (error: Error | null, value?: T) => void): this {
    this.#result.then(
      (value) => done(null, value),
      (error: unknown) =>
        done(error instanceof Error ? error : new Error(String(error))),
    );
    return this;
  }

  async *[Symbol.asyncIterator](): AsyncIterator<TaplineEvent> {
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

export function installAll(
  specs: InstallOptions[],
  options: {
    maxConcurrent?: number;
    onEach?: (report: InstallReport, index: number) => void;
  } = {},
): Batch {
  return new Batch(specs, options.maxConcurrent ?? specs.length, options.onEach);
}

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

  cancel(): void {
    this.#cancelled = true;
    for (const job of this.#jobs) job.cancel();
  }

  get cancelled(): boolean {
    return this.#cancelled;
  }

  then<A = InstallReport[], B = never>(
    onFulfilled?: ((value: InstallReport[]) => A | PromiseLike<A>) | null,
    onRejected?: ((reason: unknown) => B | PromiseLike<B>) | null,
  ): PromiseLike<A | B> {
    return this.#result.then(onFulfilled, onRejected);
  }

  callback(done: (error: Error | null, value?: InstallReport[]) => void): this {
    this.#result.then(
      (value) => done(null, value),
      (error: unknown) =>
        done(error instanceof Error ? error : new Error(String(error))),
    );
    return this;
  }
}

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

  only(pattern: string): Decoded {
    return new Decoded({
      ...this.#spec,
      filters: [...this.#spec.filters, pattern],
    });
  }

  pick(path: string): Decoded {
    return new Decoded({ ...this.#spec, picks: [...this.#spec.picks, path] });
  }

  onProgress(handler: NonNullable<WorkshopOptions["onProgress"]>): Decoded {
    return new Decoded({ ...this.#spec, onProgress: handler });
  }

  onEvent(handler: (event: TaplineEvent) => void): Decoded {
    return new Decoded({ ...this.#spec, onEvent: handler });
  }

  dir(path: string): Job<PipeReport> {
    return this.#run({ directive: "dir", path });
  }

  zip(path: string): Job<PipeReport> {
    return this.#run({ directive: "zip", path });
  }

  zipStored(path: string): Job<PipeReport> {
    return this.#run({ directive: "zip-stored", path });
  }

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

  gma(): Decoded {
    return new Decoded({ ...this.#spec, format: "gma" });
  }

  zip(): Decoded {
    return new Decoded({ ...this.#spec, format: "zip" });
  }

  decode(format: string): Decoded {
    return new Decoded({ ...this.#spec, format });
  }

  window(chunks: number): Source {
    return new Source({ ...this.#spec, concurrency: chunks });
  }
}

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

export interface SearchOptions {
  app: number;
  text?: string;
  searchIn?: "all" | "title" | "description";
  tags?: string[];
  tagGroups?: string[][];
  excludeTags?: string[];
  excludeContent?: ("nudity" | "violence" | "adult-only" | "gratuitous" | "mature")[];
  allTags?: boolean;
  sort?: "vote" | "recent" | "updated" | "trend" | "subscribed" | "text";
  days?: number;
  created?: { since?: number; until?: number };
  updated?: { since?: number; until?: number };
  limit?: number;
  cursor?: string;
  page?: number;
  onEvent?: (event: TaplineEvent) => void;
}

export interface SearchPage {
  items: Omit<ResultEvent, "kind">[];
  total: number;
  skipped: number;
  nextCursor: string | null;
}

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
        nextCursor: summary.nextCursor === "" ? null : summary.nextCursor,
      };
    },
    options.onEvent ? progressBridge(options.onEvent, undefined) : undefined,
  );
}

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

export interface QrLoginOptions {
  onCode: (url: string) => void;
  timeoutSeconds?: number;
}

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
