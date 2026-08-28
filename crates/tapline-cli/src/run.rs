//! Doing what the command line asked for.

use crate::args::{Command, Moment, SearchFilters, Step};
use std::path::PathBuf;
use tapline::{AppId, InstallOptions, Os, PublishedFileId, Session};

/// Runs one command.
pub async fn execute(command: Command) -> Result<(), String> {
    match command {
        Command::Script(steps) => run_script(steps).await,
        Command::Plan {
            app,
            dir,
            branch,
            json,
        } => plan(app, dir, branch, json).await,
        Command::Download {
            app,
            dir,
            branch,
            validate,
            concurrency,
            json,
        } => download(app, dir, branch, validate, concurrency, json).await,
        Command::Info { app, json } => info(app, json).await,
        Command::WorkshopDownload {
            flat,
            extensions,
            stream,
            only,
            pick,
            decode,
            app,
            item,
            dir,
            json,
        } => {
            workshop(
                app,
                item,
                dir,
                flat,
                extensions,
                stream,
                Selection { only, pick, decode },
                json,
            )
            .await
        }
        Command::WorkshopSearch { filters, json } => search(filters, json).await,
        Command::WorkshopInfo { items, json } => workshop_info(items, json).await,
        Command::Login { qr, account } => login(qr, account).await,
        Command::WhoAmI => whoami().await,
        Command::Help | Command::Version => Ok(()),
    }
}

/// Runs a steamcmd-style script.
///
/// Steps are evaluated in order and later ones use state the earlier ones set,
/// which is how steamcmd behaves and what every script relies on: the
/// `force_install_dir` before an `app_update` is the directory it installs to.
async fn run_script(steps: Vec<Step>) -> Result<(), String> {
    let mut install_dir = PathBuf::from(".");
    let mut session: Option<Session> = None;

    for step in steps {
        match step {
            Step::Login { account } => {
                if let Some(account) = account {
                    // Stated rather than silently ignored: a script asking for
                    // a named account and getting an anonymous session would
                    // install the wrong thing, or nothing, with no explanation.
                    return Err(format!(
                        "logging in as {account} is not supported from a script yet; \
                         run `tapline login` first, or use `+login anonymous`"
                    ));
                }
                session = Some(
                    Session::anonymous()
                        .await
                        .map_err(|error| error.to_string())?,
                );
            }

            Step::InstallDir(dir) => install_dir = dir,

            Step::AppUpdate {
                app,
                validate,
                branch,
            } => {
                let session = session
                    .as_mut()
                    .ok_or("+app_update needs a +login before it")?;

                let options = InstallOptions {
                    install_dir: install_dir.clone(),
                    os: Os::host(),
                    branch: branch.unwrap_or_else(|| "public".to_owned()),
                    force: validate,
                    ..InstallOptions::default()
                };

                let report = session
                    .install(app, &options)
                    .await
                    .map_err(|error| error.to_string())?;

                println!(
                    "Success! App '{app}' fully installed. \
                     ({} files, {} bytes downloaded)",
                    report.files, report.bytes_downloaded
                );
            }

            Step::AppInfo(app) => {
                let session = session
                    .as_mut()
                    .ok_or("+app_info_print needs a +login before it")?;
                print_info(session, app, false).await?;
            }

            Step::WorkshopDownload { app, item } => {
                let session = session
                    .as_mut()
                    .ok_or("+workshop_download_item needs a +login before it")?;
                // The steamcmd grammar has no flag for this, and steamcmd's own
                // layout is what a script using that grammar expects.
                let dir = download_item(session, app, item, install_dir.clone(), false).await?;
                println!("Success. Downloaded item {item} to \"{}\"", dir.display());
            }

            Step::Quit => break,
        }
    }
    Ok(())
}

/// `app plan`
async fn plan(app: AppId, dir: PathBuf, branch: String, json: bool) -> Result<(), String> {
    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    let options = InstallOptions {
        install_dir: dir,
        branch,
        ..InstallOptions::default()
    };

    let plan = session
        .plan(app, &options)
        .await
        .map_err(|error| error.to_string())?;

    if json {
        emit(&serde_json::json!({
            "event": "planned",
            "app": app.get(),
            "files": plan.file_count,
            "chunks": plan.chunk_count,
            "total_bytes": plan.total_bytes,
            "download_bytes": plan.download_bytes,
            "reused_bytes": plan.reused_bytes,
        }));
    } else {
        println!("{} files, {} chunks", plan.file_count, plan.chunk_count);
        println!("{} bytes on disk when complete", plan.total_bytes);
        println!("{} bytes to download", plan.download_bytes);
        println!("{} bytes already present", plan.reused_bytes);
    }
    Ok(())
}

/// `app download`
async fn download(
    app: AppId,
    dir: PathBuf,
    branch: String,
    validate: bool,
    concurrency: Option<usize>,
    json: bool,
) -> Result<(), String> {
    let defaults = InstallOptions::default();
    let concurrency = concurrency.unwrap_or(defaults.concurrency);

    // The budget is per process and a download draws from it as well as from
    // its own limit, so a session built with the default budget caps
    // `--concurrency` at the default however high the flag is set. This process
    // runs one download, so the two are the same number.
    let mut session = Session::anonymous_shared(tapline::Shared::new(concurrency))
        .await
        .map_err(|e| e.to_string())?;
    let options = InstallOptions {
        install_dir: dir,
        branch,
        force: validate,
        concurrency,
        ..defaults
    };

    let started = std::time::Instant::now();
    let report = session
        .install(app, &options)
        .await
        .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();

    if json {
        emit(&serde_json::json!({
            "event": "completed",
            "app": app.get(),
            "files": report.files,
            "bytes_written": report.bytes_written,
            "bytes_downloaded": report.bytes_downloaded,
            "chunks_reused": report.chunks_reused,
            "depots_unchanged": report.depots_unchanged,
            "seconds": elapsed.as_secs_f64(),
        }));
    } else {
        println!(
            "{} files, {} bytes written, {} downloaded in {:.1}s",
            report.files,
            report.bytes_written,
            report.bytes_downloaded,
            elapsed.as_secs_f64()
        );
        // Never silent about what was left out.
        for (path, reason) in &report.skipped {
            println!("skipped {path}: {reason}");
        }
    }
    Ok(())
}

/// `app info`
async fn info(app: AppId, json: bool) -> Result<(), String> {
    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    print_info(&mut session, app, json).await
}

async fn print_info(session: &mut Session, app: AppId, json: bool) -> Result<(), String> {
    let info = session
        .app_info(app)
        .await
        .map_err(|error| error.to_string())?;

    let filter = tapline::DepotFilter {
        os: Os::host(),
        branch: "public".to_owned(),
        include_dlc: false,
    };
    let depots = info.depots(&filter);

    if json {
        emit(&serde_json::json!({
            "event": "app_info",
            "app": app.get(),
            "name": info.name(),
            "type": info.app_type(),
            "branches": info.branches().iter().map(|branch| serde_json::json!({
                "name": branch.name,
                "build_id": branch.build_id,
                "password_required": branch.password_required,
            })).collect::<Vec<_>>(),
            "depots": depots.iter().map(|depot| serde_json::json!({
                "id": depot.id.get(),
                "manifest": depot.manifest.get(),
                "size": depot.size,
                "download_size": depot.download_size,
            })).collect::<Vec<_>>(),
        }));
    } else {
        println!("{} ({:?})", app, info.name().unwrap_or("unknown"));
        println!("type: {}", info.app_type().unwrap_or("unknown"));
        println!("branches:");
        for branch in info.branches() {
            println!(
                "  {:<24} build {}{}",
                branch.name,
                branch.build_id.unwrap_or(0),
                if branch.password_required {
                    " (password)"
                } else {
                    ""
                }
            );
        }
        println!("depots for {}:", Os::host().token());
        for depot in &depots {
            println!(
                "  {:<8} manifest {:<20} {:>14} bytes",
                depot.id, depot.manifest, depot.size
            );
        }
        println!("install size: {} bytes", info.install_size(&filter));
    }
    Ok(())
}

/// `workshop search`
#[allow(clippy::too_many_arguments)]
async fn search(filters: SearchFilters, json: bool) -> Result<(), String> {
    let defaults = tapline::BrowseQuery::default();
    // Resolved before connecting: an unknown sort should cost a message rather
    // than a login and a round trip.
    let sort = match filters.sort.as_deref() {
        None => defaults.sort,
        Some(name) => tapline::BrowseSort::parse(name).ok_or_else(|| {
            format!(
                "unknown --sort {name:?}; known: {}",
                tapline::BrowseSort::NAMES.join(", ")
            )
        })?,
    };
    // Resolved before connecting, like the sort: a misspelt label should cost
    // a message rather than a round trip that quietly filters nothing.
    let mut excluded_descriptors = Vec::new();
    for name in &filters.exclude_content {
        let descriptor = tapline::ContentDescriptor::parse(name).ok_or_else(|| {
            format!(
                "unknown --exclude-content {name:?}; known: {}",
                tapline::ContentDescriptor::NAMES.join(", ")
            )
        })?;
        excluded_descriptors.push(descriptor);
    }

    // One clock reading for the whole query, so --created-since 1d and
    // --updated-since 1d mean the same instant rather than two a millisecond
    // apart.
    let now = u32::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_or(0, |since| since.as_secs()),
    )
    .unwrap_or(u32::MAX);
    let window = |since: Option<Moment>, until: Option<Moment>| {
        (since.is_some() || until.is_some()).then(|| tapline::TimeRange {
            start: since.map(|moment| moment.resolve(now)),
            end: until.map(|moment| moment.resolve(now)),
        })
    };

    let search_in = match filters.search_in.as_deref() {
        None => tapline::TextTarget::default(),
        Some(name) => tapline::TextTarget::parse(name).ok_or_else(|| {
            format!(
                "unknown --search-in {name:?}; known: {}",
                tapline::TextTarget::NAMES.join(", ")
            )
        })?,
    };

    let query = tapline::BrowseQuery {
        app: filters.app,
        text: filters.text,
        search_in,
        required_tags: filters.tags,
        tag_groups: filters.tag_groups,
        excluded_tags: filters.exclude_tags,
        excluded_descriptors,
        match_all_tags: filters.all_tags,
        sort,
        created: window(filters.created_since, filters.created_until),
        updated: window(filters.updated_since, filters.updated_until),
        trend_days: filters.days,
        per_page: filters.limit.unwrap_or(defaults.per_page),
        cursor: filters.cursor,
    };
    query.validate().map_err(|error| error.to_string())?;

    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;

    if filters.count {
        let total = session
            .count_workshop(&query)
            .await
            .map_err(|error| error.to_string())?;
        if json {
            emit(&serde_json::json!({ "event": "counted", "total": total }));
        } else {
            println!("{total}");
        }
        return Ok(());
    }

    let page = session
        .browse_workshop(&query)
        .await
        .map_err(|error| error.to_string())?;

    if json {
        for found in &page.items {
            emit(&serde_json::json!({
                "event": "result",
                "app": found.item.app.get(),
                // A string, because item ids exceed what JSON numbers hold
                // exactly and a rounded id downloads the wrong thing.
                "item": found.item.id.get().to_string(),
                "title": found.item.title,
                "size": found.item.size,
                "updated": found.item.updated,
                "subscriptions": found.subscriptions,
                "favorites": found.favorites,
                "tags": found.tags,
                "description": found.description,
                "preview_url": found.preview_url,
            }));
        }
        emit(&serde_json::json!({
            "event": "searched",
            "total": page.total,
            "returned": page.items.len(),
            "next_cursor": page.next_cursor,
            "skipped": page.skipped.len(),
        }));
    } else {
        for found in &page.items {
            println!(
                "{:>12}  {:>9}  {:>8} subs  {}",
                found.item.id.get(),
                human_bytes(found.item.size),
                found.subscriptions,
                found.item.title
            );
        }
        println!(
            "{} of {} matches{}",
            page.items.len(),
            page.total,
            match &page.next_cursor {
                Some(cursor) => format!("; next page: --cursor {cursor}"),
                None => String::new(),
            }
        );
        // Never silent about what was left out.
        for (id, why) in &page.skipped {
            eprintln!("skipped {id}: {why}");
        }
    }
    Ok(())
}

/// Renders a byte count the way a person reads it.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    match UNITS.get(unit) {
        // The loop cannot walk past the last unit, so this is unreachable by
        // construction — but a panic in a formatter is a poor way to find out.
        None | Some(&"B") => format!("{bytes} B"),
        Some(name) => format!("{value:.1} {name}"),
    }
}

/// `workshop info`
async fn workshop_info(items: Vec<PublishedFileId>, json: bool) -> Result<(), String> {
    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    let described = session
        .workshop_details(&items)
        .await
        .map_err(|error| error.to_string())?;

    let mut failed = 0_usize;
    for outcome in described {
        match outcome {
            Ok(item) => {
                if json {
                    emit(&serde_json::json!({
                        "event": "item",
                        "app": item.app.get(),
                        "item": item.id.get().to_string(),
                        "title": item.title,
                        "size": item.size,
                        "updated": item.updated,
                        "content": match &item.content {
                            tapline::WorkshopContent::SteamPipe { depot, manifest } =>
                                serde_json::json!({
                                    "kind": "steampipe",
                                    "depot": depot.get(),
                                    "manifest": manifest.get().to_string(),
                                }),
                            tapline::WorkshopContent::Legacy { url, filename } =>
                                serde_json::json!({
                                    "kind": "legacy",
                                    "url": url,
                                    "filename": filename,
                                }),
                        },
                    }));
                } else {
                    println!(
                        "{:>12}  {:>9}  app {}  {}",
                        item.id.get(),
                        human_bytes(item.size),
                        item.app.get(),
                        item.title
                    );
                }
            }
            Err(error) => {
                failed += 1;
                if json {
                    emit(&serde_json::json!({"event": "error", "message": error.to_string()}));
                } else {
                    eprintln!("{error}");
                }
            }
        }
    }

    // A lookup where every id failed is a failure, not a silent empty result.
    if failed > 0 && failed == items.len() {
        return Err(format!("none of the {failed} items could be described"));
    }
    Ok(())
}

/// `workshop download`
#[allow(clippy::too_many_arguments)]
/// What a caller asked to take out of the archive, and how to read it.
struct Selection {
    only: Vec<String>,
    pick: Vec<String>,
    decode: Option<String>,
}

impl Selection {
    /// Whether anything narrows what is taken.
    fn is_selective(&self) -> bool {
        !self.only.is_empty() || !self.pick.is_empty()
    }
}

#[allow(clippy::too_many_arguments)]
async fn workshop(
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
    flat: bool,
    extensions: Vec<String>,
    stream: Option<String>,
    selection: Selection,
    json: bool,
) -> Result<(), String> {
    // A selection is what makes this a pipeline rather than a download: only
    // the chunks holding the selected entries are fetched. A named format is
    // the same, since reading it at all means decoding it.
    if selection.is_selective() || selection.decode.is_some() {
        let target = stream.as_deref().unwrap_or("dir");
        return pipe_workshop(app, item, dir, target, &selection, json).await;
    }
    if let Some(target) = stream {
        return stream_workshop(app, item, dir, &target, json).await;
    }
    // Resolved before connecting. A typo should cost a message, not a login and
    // a round trip to Steam first.
    let resolved: Vec<_> = extensions
        .iter()
        .map(|name| extension_by_name(name))
        .collect::<Result<_, _>>()?;

    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    for extension in resolved {
        session.register(extension);
    }
    let target = download_item(&mut session, app, item, dir, flat).await?;

    if json {
        emit(&serde_json::json!({
            "event": "workshop_completed",
            "app": app.get(),
            "item": item.get(),
            "path": target.display().to_string(),
        }));
    } else {
        println!("downloaded item {item} to {}", target.display());
    }
    Ok(())
}

/// `workshop download --only/--pick`
///
/// Runs the item through a pipeline instead of downloading it whole. The saving
/// is on the wire, not just on disk: a selection is resolved against the
/// archive's index first, and only the chunks the selected entries live in are
/// fetched.
async fn pipe_workshop(
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
    target: &str,
    selection: &Selection,
    json: bool,
) -> Result<(), String> {
    let format = selection.decode.as_deref().unwrap_or("gma");

    // A zip target names a file; a directory target names a directory. Same
    // convention as --stream, so the two flags do not disagree about where
    // things land.
    let zip_path = dir.join(format!("{item}.zip"));
    let sink = match target {
        "zip" => tapline_pipe::Sink::Zip {
            path: zip_path.display().to_string(),
            compress: true,
        },
        "zip-stored" => tapline_pipe::Sink::Zip {
            path: zip_path.display().to_string(),
            compress: false,
        },
        _ => tapline_pipe::Sink::Directory(dir.display().to_string()),
    };

    let pipeline = tapline_pipe::Pipeline {
        format: format.to_owned(),
        filters: selection.only.clone(),
        picks: selection.pick.clone(),
        sink: Some(sink),
    };
    // Before connecting: an unknown format should cost a message rather than a
    // login and a round trip to Steam.
    pipeline.validate().map_err(|error| error.to_string())?;

    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;

    let started = std::time::Instant::now();
    let outcome = tapline_pipe::run_pipeline(
        &mut session,
        app,
        item,
        tapline::Window::default(),
        &pipeline,
        &mut |_event| {},
    )
    .await
    .map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();

    if json {
        emit(&serde_json::json!({
            "event": "piped",
            "app": app.get(),
            "item": item.get(),
            "path": dir.display().to_string(),
            "format": format,
            "target": target,
            "entries": outcome.entries,
            "bytes_downloaded": outcome.bytes_downloaded,
            "bytes_streamed": outcome.bytes_streamed,
            "peak_buffered_chunks": outcome.peak_buffered,
            "seconds": elapsed.as_secs_f64(),
        }));
    } else {
        println!(
            "took {} entries from item {item} into {} as {target}: \
             {} bytes downloaded, {:.2}s",
            outcome.entries,
            dir.display(),
            outcome.bytes_downloaded,
            elapsed.as_secs_f64()
        );
    }
    Ok(())
}

/// `workshop download --stream`
///
/// Unpacks the addon as it downloads. The `.gma` is never written: GMAD's
/// header and index come first and its contents follow in index order, so each
/// file can be written the moment its bytes land.
async fn stream_workshop(
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
    target: &str,
    json: bool,
) -> Result<(), String> {
    let _ = app;
    let mut session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    let details = session
        .workshop_details(&[item])
        .await
        .map_err(|error| error.to_string())?
        .into_iter()
        .next()
        .ok_or_else(|| format!("Steam said nothing about item {item}"))?
        .map_err(|error| error.to_string())?;

    // A zip target names a file; a directory target names a directory.
    let zip_path = dir.join(format!("{item}.zip"));
    let chosen = match target {
        "zip" => tapline_gmad::StreamTarget::Zip(&zip_path),
        "zip-stored" => tapline_gmad::StreamTarget::ZipStored(&zip_path),
        _ => tapline_gmad::StreamTarget::Directory(&dir),
    };
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    let mut extractor =
        tapline_gmad::StreamWriter::new(chosen).map_err(|error| error.to_string())?;

    let started = std::time::Instant::now();
    let report = session
        .stream_workshop_item(
            &details,
            tapline::Window::default(),
            &mut |bytes| {
                extractor
                    .push(bytes)
                    .map_err(|error| tapline::InstallError::Io(error.to_string()))
            },
            &mut |_event| {},
        )
        .await
        .map_err(|error| error.to_string())?;

    let produced = extractor.finish().map_err(|error| error.to_string())?;
    let elapsed = started.elapsed();

    if json {
        emit(&serde_json::json!({
            "event": "streamed",
            "item": item.get(),
            "path": dir.display().to_string(),
            "target": target,
            "files": produced.entries,
            "bytes_downloaded": report.bytes_downloaded,
            "bytes_streamed": report.bytes_streamed,
            "peak_buffered_chunks": report.peak_buffered,
            "seconds": elapsed.as_secs_f64(),
        }));
    } else {
        println!(
            "streamed item {item} into {} as {target}: {} files, {} bytes, {} chunks, \
             peak {} buffered, {:.2}s",
            dir.display(),
            produced.entries,
            report.bytes_streamed,
            report.chunks,
            report.peak_buffered,
            elapsed.as_secs_f64()
        );
    }
    Ok(())
}

/// Looks up a built-in extension.
///
/// An unknown name is refused rather than ignored: a caller who asked for
/// unpacking and got a directory of untouched `.gma` files has been told
/// nothing about why.
fn extension_by_name(name: &str) -> Result<std::sync::Arc<dyn tapline::Extension>, String> {
    match name {
        "gmad" => Ok(std::sync::Arc::new(tapline_gmad::Extract::new())),
        "gmad!" => Ok(std::sync::Arc::new(
            tapline_gmad::Extract::new().removing_original(),
        )),
        "gmad-zip" => Ok(std::sync::Arc::new(tapline_gmad::ToZip::new())),
        "gmad-zip!" => Ok(std::sync::Arc::new(
            tapline_gmad::ToZip::new().removing_original(),
        )),
        "gmad-zip-stored" => Ok(std::sync::Arc::new(tapline_gmad::ToZip::new().stored())),
        other => Err(format!(
            "unknown extension {other:?}; known: gmad, gmad!, gmad-zip, gmad-zip!, \
             gmad-zip-stored (a trailing ! deletes the original)"
        )),
    }
}

/// Downloads one Workshop item and returns where it landed.
async fn download_item(
    session: &mut Session,
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
    flat: bool,
) -> Result<PathBuf, String> {
    let described = session
        .workshop_details(&[item])
        .await
        .map_err(|error| error.to_string())?;

    let details = described
        .into_iter()
        .next()
        .ok_or_else(|| format!("Steam said nothing about item {item}"))?
        .map_err(|error| error.to_string())?;

    let options = InstallOptions {
        install_dir: dir.clone(),
        workshop_layout: if flat {
            tapline::WorkshopLayout::Flat
        } else {
            tapline::WorkshopLayout::SteamCmd
        },
        ..InstallOptions::default()
    };
    session
        .download_workshop_item(&details, &options)
        .await
        .map_err(|error| error.to_string())?;

    let _ = app;
    Ok(tapline::target_dir(&options, details.app, details.id))
}

/// `login`
async fn login(qr: bool, account: Option<String>) -> Result<(), String> {
    // A machine with Steam on it already knows who you are. Say so, so the
    // account name in the QR prompt is not a surprise.
    let local = tapline_auth::most_recent();
    if account.is_none()
        && let Some(found) = &local
    {
        if found.persona.is_empty() {
            println!(
                "this machine's Steam client last signed in as {}",
                found.account
            );
        } else {
            println!(
                "this machine's Steam client last signed in as {} ({})",
                found.account, found.persona
            );
        }
    }

    // Anonymous on purpose: signing in is what this command is for, and a
    // session that quietly reused an old token would hide a failed login.
    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;

    if let Some(name) = account.filter(|_| !qr) {
        return password_login(&mut session, &name).await;
    }
    let pending = session
        .begin_qr_login()
        .await
        .map_err(|error| error.to_string())?;

    println!("{}", pending.instruction());
    println!(
        "waiting for approval, polling every {:.0}s...",
        pending.interval
    );

    let interval = std::time::Duration::from_secs_f32(pending.interval.max(1.0));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(300);
    let mut current = pending;

    while std::time::Instant::now() < deadline {
        tokio::time::sleep(interval).await;

        match session
            .poll_login(&current)
            .await
            .map_err(|error| error.to_string())?
        {
            tapline::PollOutcome::Pending { had_interaction } => {
                if had_interaction {
                    println!("approval started...");
                }
            }
            tapline::PollOutcome::Moved {
                client_id,
                challenge_url,
            } => {
                // Steam refreshed the code. Polling the old one waits forever.
                current.client_id = client_id;
                current.challenge_url = challenge_url;
                println!("{}", current.instruction());
            }
            tapline::PollOutcome::Complete {
                account,
                refresh_token,
                ..
            } => return finish_login(&account, &refresh_token),
        }
    }

    Err("timed out waiting for approval".to_owned())
}

/// `whoami`
/// Signs in with an account name and a password typed at the terminal.
async fn password_login(session: &mut Session, account: &str) -> Result<(), String> {
    let key = session
        .password_key(account)
        .await
        .map_err(|error| error.to_string())?;

    // Read straight from the terminal with echo off, and never from an
    // argument: a password on the command line is in the shell history and in
    // every `ps` listing on the machine.
    let password = read_hidden(&format!("password for {account}: "))?;
    if password.is_empty() {
        return Err("no password given".to_owned());
    }

    let mut pending = session
        .begin_password_login(account, password, &key)
        .await
        .map_err(|error| error.to_string())?;

    // A code, when Steam wants one. The confirmations are alternatives, so
    // take the first that a typed code satisfies.
    if let Some(kind) = pending
        .confirmations
        .iter()
        .copied()
        .find(|kind| kind.needs_a_code())
    {
        let code = read_line(&format!("Steam Guard code ({kind}): "))?;
        if code.is_empty() {
            return Err("no Steam Guard code given".to_owned());
        }
        session
            .submit_guard_code(&pending, code.trim(), kind)
            .await
            .map_err(|error| error.to_string())?;
    } else if !pending.confirmations.is_empty() {
        println!("{}", pending.instruction());
    }

    let interval = std::time::Duration::from_secs_f32(pending.interval.max(1.0));
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
    while std::time::Instant::now() < deadline {
        match session
            .poll_login(&pending)
            .await
            .map_err(|error| error.to_string())?
        {
            tapline::PollOutcome::Complete {
                account,
                refresh_token,
                ..
            } => return finish_login(&account, &refresh_token),
            tapline::PollOutcome::Moved { client_id, .. } => pending.client_id = client_id,
            tapline::PollOutcome::Pending { .. } => {}
        }
        tokio::time::sleep(interval).await;
    }
    Err("timed out waiting for the login to complete".to_owned())
}

/// Saves the token and says where it went.
///
/// Saved by default, because a login that has to be repeated every run is not
/// a login. The file is the store's own, under the user's config directory.
fn finish_login(account: &str, refresh_token: &str) -> Result<(), String> {
    let store = tapline_auth::TokenStore::default_file();
    let token = tapline_auth::StoredToken {
        account: account.to_owned(),
        refresh_token: refresh_token.to_owned(),
    };
    store.save(&token).map_err(|error| error.to_string())?;
    println!("signed in as {account}; token saved, future commands will use it");
    Ok(())
}

/// Reads a line from the terminal.
fn read_line(prompt: &str) -> Result<String, String> {
    use std::io::Write;
    print!("{prompt}");
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut line = String::new();
    std::io::stdin()
        .read_line(&mut line)
        .map_err(|e| e.to_string())?;
    Ok(line.trim().to_owned())
}

/// Reads a line without echoing it.
///
/// Turns the terminal's echo off around the read rather than pulling in a
/// dependency for it. If the terminal cannot be reconfigured — a pipe, a CI
/// job — it refuses instead of echoing the password, because printing a
/// password to a log is worse than failing.
fn read_hidden(prompt: &str) -> Result<String, String> {
    use std::io::Write;
    use std::process::Command;

    if !std::io::IsTerminal::is_terminal(&std::io::stdin()) {
        return Err(
            "a password has to be typed at a terminal; this is not one. \
             Use `tapline login --qr`, which needs no password"
                .to_owned(),
        );
    }

    // `stty` rather than a crate: it is present wherever a terminal is, and
    // the alternative is a dependency for two syscalls.
    let off = Command::new("stty").arg("-echo").status();
    print!("{prompt}");
    std::io::stdout().flush().map_err(|e| e.to_string())?;

    let mut line = String::new();
    let read = std::io::stdin().read_line(&mut line);

    if off.map(|status| status.success()).unwrap_or(false) {
        let _ = Command::new("stty").arg("echo").status();
    }
    println!();
    read.map_err(|e| e.to_string())?;
    Ok(line.trim().to_owned())
}

async fn whoami() -> Result<(), String> {
    let session = Session::automatic(None).await.map_err(|e| e.to_string())?;
    match session.account() {
        Some(account) => println!("signed in as {account}, cell {}", session.cell_id()),
        None => println!("anonymous session, cell {}", session.cell_id()),
    }

    // The local Steam client's accounts are a different thing from tapline's
    // session, and saying which is which is the whole point of printing both.
    let accounts = tapline_auth::discover();
    if accounts.is_empty() {
        println!("no local Steam client found");
    } else {
        for found in &accounts {
            println!(
                "local Steam account: {}{}{}",
                found.account,
                if found.persona.is_empty() {
                    String::new()
                } else {
                    format!(" ({})", found.persona)
                },
                if found.most_recent {
                    ", most recent"
                } else {
                    ""
                }
            );
        }
    }

    for library in tapline_auth::libraries() {
        println!("Steam library: {}", library.display());
    }
    Ok(())
}

/// Writes one newline-delimited JSON event.
fn emit(value: &serde_json::Value) {
    println!("{value}");
}
