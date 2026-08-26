//! Events, as JSON objects a JS runtime can hand straight to `JSON.parse`.
//!
//! One shape per event, discriminated by `kind`, with `kind` spelled in the
//! lower-snake-case a JS consumer expects rather than the Rust variant name.

use crate::json::{push_str_field, push_u64};
use tapline::{Event, InstallReport, Plan, RetryReason};

/// Encodes one event.
#[must_use]
pub fn encode(event: &Event) -> String {
    let mut out = String::from("{");
    match event {
        Event::Planned { plan } => {
            push_str_field(&mut out, "kind", "planned");
            push_plan(&mut out, plan);
        }
        Event::DepotStarted {
            depot,
            manifest,
            bytes,
        } => {
            push_str_field(&mut out, "kind", "depotStarted");
            push_u64(&mut out, "depot", u64::from(depot.get()));
            // Manifest ids exceed 2^53 and would lose precision as a JS number,
            // so they cross as strings and stay exact.
            push_str_field(&mut out, "manifest", &manifest.get().to_string());
            push_u64(&mut out, "bytes", *bytes);
        }
        Event::DepotCompleted { depot } => {
            push_str_field(&mut out, "kind", "depotCompleted");
            push_u64(&mut out, "depot", u64::from(depot.get()));
        }
        Event::Progress {
            bytes_done,
            bytes_total,
        } => {
            push_str_field(&mut out, "kind", "progress");
            push_u64(&mut out, "bytesDone", *bytes_done);
            push_u64(&mut out, "bytesTotal", *bytes_total);
        }
        Event::FileCompleted { path, bytes } => {
            push_str_field(&mut out, "kind", "fileCompleted");
            push_str_field(&mut out, "path", path);
            push_u64(&mut out, "bytes", *bytes);
        }
        Event::Retrying {
            host,
            reason,
            attempt,
        } => {
            push_str_field(&mut out, "kind", "retrying");
            push_str_field(&mut out, "host", host);
            push_str_field(&mut out, "reason", reason_name(reason));
            push_u64(&mut out, "attempt", u64::from(*attempt));
        }
        Event::Extended {
            extension,
            path,
            produced,
        } => {
            push_str_field(&mut out, "kind", "extended");
            push_str_field(&mut out, "extension", extension);
            push_str_field(&mut out, "path", path);
            push_u64(&mut out, "produced", *produced);
        }
        Event::Verifying { path } => {
            push_str_field(&mut out, "kind", "verifying");
            push_str_field(&mut out, "path", path);
        }
        Event::Completed {
            app,
            downloaded_bytes,
            reused_bytes,
        } => {
            push_str_field(&mut out, "kind", "completed");
            push_u64(&mut out, "app", u64::from(app.get()));
            push_u64(&mut out, "downloadedBytes", *downloaded_bytes);
            push_u64(&mut out, "reusedBytes", *reused_bytes);
        }
        // `Event` is `#[non_exhaustive]` on purpose, and a consumer that sees an
        // event this build does not know about should be told so rather than be
        // handed a silently dropped one.
        other => {
            push_str_field(&mut out, "kind", "unknown");
            push_str_field(&mut out, "debug", &format!("{other:?}"));
        }
    }
    out.push('}');
    out
}

/// Encodes the final report as a `finished` event.
///
/// Not an [`Event`]: the report is what the Rust API returns rather than
/// something it emits, and a JS caller wants it as the last thing on the stream
/// instead of as a second channel to read.
#[must_use]
pub fn encode_report(report: &InstallReport) -> String {
    let mut out = String::from("{");
    push_str_field(&mut out, "kind", "finished");
    push_u64(&mut out, "app", u64::from(report.app.get()));
    push_u64(&mut out, "files", report.files);
    push_u64(&mut out, "bytesWritten", report.bytes_written);
    push_u64(&mut out, "bytesDownloaded", report.bytes_downloaded);
    push_u64(&mut out, "chunksReused", report.chunks_reused);
    push_u64(&mut out, "depotsUnchanged", report.depots_unchanged);
    push_key_array_of_depots(&mut out, report);
    // Never silent about what was left out.
    crate::json::push_key(&mut out, "skipped");
    out.push('[');
    for (index, (path, reason)) in report.skipped.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('{');
        push_str_field(&mut out, "path", path);
        push_str_field(&mut out, "reason", reason);
        out.push('}');
    }
    out.push(']');
    out.push('}');
    out
}

/// Encodes a plan on its own, for the `plan` job.
#[must_use]
pub fn encode_plan(plan: &Plan) -> String {
    let mut out = String::from("{");
    push_str_field(&mut out, "kind", "planned");
    push_plan(&mut out, plan);
    out.push('}');
    out
}

fn push_plan(out: &mut String, plan: &Plan) {
    push_u64(out, "downloadBytes", plan.download_bytes);
    push_u64(out, "reusedBytes", plan.reused_bytes);
    push_u64(out, "totalBytes", plan.total_bytes);
    push_u64(out, "fileCount", plan.file_count);
    push_u64(out, "chunkCount", plan.chunk_count);
}

fn push_key_array_of_depots(out: &mut String, report: &InstallReport) {
    crate::json::push_key(out, "depots");
    out.push('[');
    for (index, depot) in report.depots.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(&depot.get().to_string());
    }
    out.push(']');
}

fn reason_name(reason: &RetryReason) -> &'static str {
    match reason {
        RetryReason::Transport => "transport",
        RetryReason::RateLimited => "rateLimited",
        RetryReason::IntegrityFailure => "integrityFailure",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_ids::{AppId, DepotId, ManifestId};

    #[test]
    fn progress_encodes_as_a_flat_object() {
        let json = encode(&Event::Progress {
            bytes_done: 10,
            bytes_total: 20,
        });
        assert_eq!(
            json,
            r#"{"kind":"progress","bytesDone":10,"bytesTotal":20}"#
        );
    }

    #[test]
    fn a_manifest_id_crosses_as_a_string() {
        // 964598927051546960 is a real GMod manifest id and is larger than
        // Number.MAX_SAFE_INTEGER. As a JSON number JavaScript would round it,
        // and the value would come back wrong without anything erroring.
        let json = encode(&Event::DepotStarted {
            depot: DepotId(4021),
            manifest: ManifestId(964_598_927_051_546_960),
            bytes: 7,
        });
        assert!(
            json.contains(r#""manifest":"964598927051546960""#),
            "manifest id must be a string: {json}"
        );
        assert!(json.contains(r#""depot":4021"#));
    }

    #[test]
    fn a_hostile_path_stays_inside_its_string() {
        let json = encode(&Event::FileCompleted {
            path: "a\",\"kind\":\"finished".to_owned(),
            bytes: 1,
        });
        assert!(json.starts_with(r#"{"kind":"fileCompleted","path":"a\",\"kind\":\"finished""#));
    }

    #[test]
    fn every_named_variant_says_what_it_is() {
        let events = [
            Event::Planned {
                plan: Plan::default(),
            },
            Event::DepotCompleted {
                depot: DepotId(1006),
            },
            Event::Retrying {
                host: "cache1-iad1.steamcontent.com".to_owned(),
                reason: RetryReason::IntegrityFailure,
                attempt: 2,
            },
            Event::Verifying {
                path: "srcds_run".to_owned(),
            },
            Event::Extended {
                extension: "gmad".to_owned(),
                path: "addons/x.gma".to_owned(),
                produced: 348,
            },
            Event::Completed {
                app: AppId(4020),
                downloaded_bytes: 1,
                reused_bytes: 2,
            },
        ];
        for event in &events {
            let json = encode(event);
            assert!(json.starts_with(r#"{"kind":""#), "{json}");
            assert!(!json.contains(r#""kind":"unknown""#), "{json}");
            assert!(json.ends_with('}'));
        }
    }

    #[test]
    fn a_report_lists_its_depots_and_its_skips() {
        let report = InstallReport {
            app: AppId(4020),
            depots: vec![DepotId(1006), DepotId(4021)],
            files: 3,
            skipped: vec![("bad/path".to_owned(), "unsafe".to_owned())],
            ..InstallReport::default()
        };
        let json = encode_report(&report);
        assert!(json.contains(r#""depots":[1006,4021]"#), "{json}");
        assert!(
            json.contains(r#""skipped":[{"path":"bad/path","reason":"unsafe"}]"#),
            "{json}"
        );
    }

    #[test]
    fn an_empty_report_still_produces_valid_arrays() {
        let json = encode_report(&InstallReport::default());
        assert!(json.contains(r#""depots":[]"#), "{json}");
        assert!(json.contains(r#""skipped":[]"#), "{json}");
    }
}
