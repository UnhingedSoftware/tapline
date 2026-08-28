
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
  manifest: string;
  bytes: number;
}

export interface DepotCompletedEvent {
  kind: "depotCompleted";
  depot: number;
}

export interface ProgressEvent {
  kind: "progress";
  bytesDone: number;
  bytesTotal: number;
}

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

export interface StreamedEvent extends StreamReport {
  kind: "streamed";
}

export interface ResultEvent {
  kind: "result";
  app: number;
  item: string;
  title: string;
  description: string;
  size: number;
  updated: number;
  created: number;
  creator: string;
  subscriptions: number;
  favorites: number;
  views: number;
  votesUp: number;
  votesDown: number;
  previewUrl: string;
  tags: string[];
}

export interface CountedEvent {
  kind: "counted";
  total: number;
}

export interface QrEvent {
  kind: "qr";
  url: string;
}

export interface LoggedInEvent {
  kind: "loggedIn";
  account: string;
}

export interface SearchedEvent {
  kind: "searched";
  total: number;
  returned: number;
  skipped: number;
  nextCursor: string;
}

export interface PipedEvent extends PipeReport {
  kind: "piped";
}

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

export interface FinishedEvent {
  kind: "finished";
  app: number;
  files: number;
  bytesWritten: number;
  bytesDownloaded: number;
  chunksReused: number;
  depotsUnchanged: number;
  depots: number[];
  skipped: { path: string; reason: string }[];
}

export interface ErrorEvent {
  kind: "error";
  message: string;
}

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

export type InstallReport = Omit<FinishedEvent, "kind">;

export type PlanReport = Omit<PlannedEvent, "kind">;

export type TargetOs = "host" | "linux" | "windows" | "macos";

export type FileModes =
  | "steamcmd"
  | "manifest";

export interface InstallOptions {
  app: number;
  dir: string;
  branch?: string;
  concurrency?: number;
  os?: TargetOs;
  validate?: boolean;
  includeDlc?: boolean;
  fileModes?: FileModes;
  extensions?: string[];
  onEvent?: (event: TaplineEvent) => void;
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
  item: bigint | number | string;
  dir: string;
  concurrency?: number;
  layout?: "steamcmd" | "flat";
  extensions?: string[];
  stream?: boolean | "extract" | "zip" | "zip-stored";
  onEvent?: (event: TaplineEvent) => void;
  onProgress?: (progress: {
    bytesDone: number;
    bytesTotal: number;
    percent: number;
  }) => void;
}

export interface PipeReport {
  entries: number;
  bytesDownloaded: number;
  bytesStreamed: number;
  peakBufferedChunks: number;
}

export interface StreamReport {
  files: number;
  bytesDownloaded: number;
  bytesStreamed: number;
  chunks: number;
  peakBufferedChunks: number;
}
