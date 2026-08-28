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
  /** A string: manifest ids exceed Number.MAX_SAFE_INTEGER. */
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
  reason: "transport" | "rateLimited" | "integrityFailure" | "unknown";
  attempt: number;
}

export interface VerifyingEvent {
  kind: "verifying";
  path: string;
}

/** The last event of a successful streamed download. */
export interface StreamedEvent extends StreamReport {
  kind: "streamed";
}

/** One Workshop search result. */
export interface ResultEvent {
  kind: "result";
  app: number;
  /** The item id, as a string: ids exceed Number.MAX_SAFE_INTEGER. */
  item: string;
  title: string;
  description: string;
  /** Size in bytes. */
  size: number;
  /** Last update, as a Unix timestamp. */
  updated: number;
  /** First published, as a Unix timestamp. */
  created: number;
  /** Who published it, as a SteamID64 string. Empty when Steam gives none. */
  creator: string;
  subscriptions: number;
  favorites: number;
  views: number;
  votesUp: number;
  votesDown: number;
  /** Empty when the item has no preview image. */
  previewUrl: string;
  tags: string[];
}

/** How many items a search would match, with none of them fetched. */
export interface CountedEvent {
  kind: "counted";
  total: number;
}

export interface QrEvent {
  kind: "qr";
  /** The URL to render as a QR code. Changes when Steam rotates the code. */
  url: string;
}

export interface LoggedInEvent {
  kind: "loggedIn";
  /** The account that signed in. Its token is now saved. */
  account: string;
}

export interface SearchedEvent {
  kind: "searched";
  total: number;
  returned: number;
  skipped: number;
  /** Pass back as `cursor` for the next page; empty means there is none. */
  nextCursor: string;
}

/** The last event of a successful pipeline run. */
export interface PipedEvent extends PipeReport {
  kind: "piped";
}

/** An extension acted on a file it claimed. */
export interface ExtendedEvent {
  kind: "extended";
  extension: string;
  path: string;
  produced: number;
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
  | ExtendedEvent
  | StreamedEvent
  | ResultEvent
  | QrEvent
  | LoggedInEvent
  | CountedEvent
  | SearchedEvent
  | PipedEvent
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
  app: number;
  /** Where to install. Created if absent. */
  dir: string;
  /** Branch, default `public`. */
  branch?: string;
  /** Chunks in flight; default 64. */
  concurrency?: number;
  os?: TargetOs;
  /** Re-download even when the install record says it is current. */
  validate?: boolean;
  includeDlc?: boolean;
  fileModes?: FileModes;
  /** Post-processing extension names; see {@link WorkshopOptions.extensions}. */
  extensions?: string[];
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
  app: number;
  /** The published file id; a bigint, because these exceed 2^53. */
  item: bigint | number | string;
  dir: string;
  concurrency?: number;
  /** `"steamcmd"` (default) builds the steamapps layout under `dir`; `"flat"` writes into `dir`. */
  layout?: "steamcmd" | "flat";
  /** Extension names, not functions: gmad, gmad-zip, gmad-zip-stored; a trailing `!` deletes the original. */
  extensions?: string[];
  /** Stream the addon as it downloads; implies `layout: "flat"` and ignores `extensions`. */
  stream?: boolean | "extract" | "zip" | "zip-stored";
  onEvent?: (event: TaplineEvent) => void;
  onProgress?: (progress: {
    bytesDone: number;
    bytesTotal: number;
    percent: number;
  }) => void;
}

/** What a pipeline produced. */
export interface PipeReport {
  entries: number;
  /** Bytes fetched from the CDN. */
  bytesDownloaded: number;
  /** Bytes handed to the decoder. */
  bytesStreamed: number;
  /** The most chunks held back at once, waiting on an earlier one. */
  peakBufferedChunks: number;
}

export interface StreamReport {
  files: number;
  /** Bytes fetched from the CDN. */
  bytesDownloaded: number;
  /** Bytes handed to the extractor. */
  bytesStreamed: number;
  chunks: number;
  /** The most chunks held back at once, waiting on an earlier one. */
  peakBufferedChunks: number;
}
