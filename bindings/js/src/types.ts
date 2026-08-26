/** The events a job emits, as they arrive from the C ABI. */

/** What an install will cost. Emitted once, first, before anything downloads. */
export interface PlannedEvent {
  kind: "planned";
  downloadBytes: number;
  reusedBytes: number;
  totalBytes: number;
  fileCount: number;
  chunkCount: number;
}

export interface DepotStartedEvent {
  kind: "depotStarted";
  depot: number;
  /**
   * A string, not a number. Manifest ids exceed `Number.MAX_SAFE_INTEGER`, so
   * as a JSON number JavaScript would silently round them.
   */
  manifest: string;
  bytes: number;
}

export interface DepotCompletedEvent {
  kind: "depotCompleted";
  depot: number;
}

/** Emitted per chunk written, so it is frequent. Aggregate before drawing. */
export interface ProgressEvent {
  kind: "progress";
  bytesDone: number;
  bytesTotal: number;
}

/** The file is on disk and synced — safe to read, hash or execute. */
export interface FileCompletedEvent {
  kind: "fileCompleted";
  path: string;
  bytes: number;
}

export interface RetryingEvent {
  kind: "retrying";
  host: string;
  /**
   * `integrityFailure` is the one worth logging: a CDN or caching proxy served
   * bytes that were not what the manifest named, and tapline refetched instead
   * of writing them.
   */
  reason: "transport" | "rateLimited" | "integrityFailure" | "unknown";
  attempt: number;
}

export interface VerifyingEvent {
  kind: "verifying";
  path: string;
}

export interface CompletedEvent {
  kind: "completed";
  app: number;
  downloadedBytes: number;
  reusedBytes: number;
}

/** The last event of a successful job. */
export interface FinishedEvent {
  kind: "finished";
  app: number;
  files: number;
  bytesWritten: number;
  bytesDownloaded: number;
  chunksReused: number;
  depotsUnchanged: number;
  depots: number[];
  /** Files deliberately not installed, and why. Never silently empty. */
  skipped: { path: string; reason: string }[];
}

/** The last event of a failed job. */
export interface ErrorEvent {
  kind: "error";
  message: string;
}

/** An event this build of the bindings does not know about. */
export interface UnknownEvent {
  kind: "unknown";
  debug: string;
}

export type TaplineEvent =
  | PlannedEvent
  | DepotStartedEvent
  | DepotCompletedEvent
  | ProgressEvent
  | FileCompletedEvent
  | RetryingEvent
  | VerifyingEvent
  | CompletedEvent
  | FinishedEvent
  | ErrorEvent
  | UnknownEvent;

/** What an install produced. Resolved from a job's promise. */
export type InstallReport = Omit<FinishedEvent, "kind">;

/** What a plan produced. */
export type PlanReport = Omit<PlannedEvent, "kind">;

/** Which platform's depots to install. Defaults to the host. */
export type TargetOs = "host" | "linux" | "windows" | "macos";

/** How installed files are chmod'd. */
export type FileModes =
  /** 0755 on everything, which is what steamcmd does. The default. */
  | "steamcmd"
  /** 0755 only where the manifest says executable. */
  | "manifest";

export interface InstallOptions {
  /** The app id. Garry's Mod Dedicated Server is 4020. */
  app: number;
  /** Where to install. Created if absent. */
  dir: string;
  /** Branch, default `public`. */
  branch?: string;
  /** Chunks in flight. Default 64, which is where throughput turns over. */
  concurrency?: number;
  os?: TargetOs;
  /** Re-download even when the install record says it is current. */
  validate?: boolean;
  includeDlc?: boolean;
  fileModes?: FileModes;
  /** Called for every event, including progress. */
  onEvent?: (event: TaplineEvent) => void;
  /** Called for progress only, with a `percent` worked out for you. */
  onProgress?: (progress: {
    bytesDone: number;
    bytesTotal: number;
    percent: number;
  }) => void;
}

export interface PlanOptions {
  app: number;
  dir: string;
  branch?: string;
  os?: TargetOs;
  includeDlc?: boolean;
}

export interface WorkshopOptions {
  /** The app the item belongs to. */
  app: number;
  /** The published file id. A bigint, because these exceed 2^53. */
  item: bigint | number | string;
  dir: string;
  concurrency?: number;
  /**
   * Where the item's files land.
   *
   * `"steamcmd"` (the default) builds
   * `<dir>/steamapps/workshop/content/<app>/<item>/` underneath `dir`, which is
   * what steamcmd does and where the Steam client and wings eggs look.
   *
   * `"flat"` writes them into `dir` itself. That is what you want when `dir` is
   * already the right folder — a Garry's Mod addon belongs in
   * `garrysmod/addons`, and under the steamcmd layout it would land four
   * directories below where the server looks for it.
   */
  layout?: "steamcmd" | "flat";
  onEvent?: (event: TaplineEvent) => void;
  onProgress?: (progress: {
    bytesDone: number;
    bytesTotal: number;
    percent: number;
  }) => void;
}
