//! Doing what the command line asked for.

use crate::args::{Command, Step};
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
            json,
        } => download(app, dir, branch, validate, json).await,
        Command::Info { app, json } => info(app, json).await,
        Command::WorkshopDownload {
            app,
            item,
            dir,
            json,
        } => workshop(app, item, dir, json).await,
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
                let dir = download_item(session, app, item, install_dir.clone()).await?;
                println!("Success. Downloaded item {item} to \"{}\"", dir.display());
            }

            Step::Quit => break,
        }
    }
    Ok(())
}

/// `app plan`
async fn plan(app: AppId, dir: PathBuf, branch: String, json: bool) -> Result<(), String> {
    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;
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
    json: bool,
) -> Result<(), String> {
    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;
    let options = InstallOptions {
        install_dir: dir,
        branch,
        force: validate,
        ..InstallOptions::default()
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
    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;
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

/// `workshop download`
async fn workshop(
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
    json: bool,
) -> Result<(), String> {
    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;
    let target = download_item(&mut session, app, item, dir).await?;

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

/// Downloads one Workshop item and returns where it landed.
async fn download_item(
    session: &mut Session,
    app: AppId,
    item: PublishedFileId,
    dir: PathBuf,
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
        ..InstallOptions::default()
    };
    session
        .download_workshop_item(&details, &options)
        .await
        .map_err(|error| error.to_string())?;

    let _ = app;
    Ok(tapline::item_dir(&dir, details.app, details.id))
}

/// `login`
async fn login(qr: bool, account: Option<String>) -> Result<(), String> {
    if !qr && account.is_some() {
        // Honest about the gap rather than prompting for a password the rest of
        // the flow does not yet finish.
        return Err(
            "password login is not wired into the CLI yet; use `tapline login --qr`".to_owned(),
        );
    }

    let mut session = Session::anonymous().await.map_err(|e| e.to_string())?;
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
            } => {
                println!("signed in as {account}");
                // Storing is opt-in and off by default; say where it would go
                // rather than writing a credential nobody asked to persist.
                let _ = refresh_token;
                println!(
                    "the refresh token was not saved; \
                     pass a token store to persist it"
                );
                return Ok(());
            }
        }
    }

    Err("timed out waiting for approval".to_owned())
}

/// `whoami`
async fn whoami() -> Result<(), String> {
    let session = Session::anonymous().await.map_err(|e| e.to_string())?;
    println!("anonymous session, cell {}", session.cell_id());
    Ok(())
}

/// Writes one newline-delimited JSON event.
fn emit(value: &serde_json::Value) {
    println!("{value}");
}
