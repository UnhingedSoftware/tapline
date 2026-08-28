mod event;
mod json;

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;
use tapline::{
    FileModes, InstallOptions, Os, PublishedFileId, Session, SessionPool, Shared, WorkshopLayout,
};
use tapline_ids::AppId;

pub const TAPLINE_OK: i32 = 0;
pub const TAPLINE_TIMEOUT: i32 = 1;
pub const TAPLINE_DONE: i32 = 2;
pub const TAPLINE_BUFFER_TOO_SMALL: i32 = -1;
pub const TAPLINE_BAD_ARGUMENT: i32 = -2;

#[derive(Debug, Clone, Copy)]
pub struct TaplineOptions {
    pub concurrency: u32,
    pub branch: *const c_char,
    pub os: u8,
    pub validate: u8,
    pub include_dlc: u8,
    pub file_modes: u8,
}

pub struct TaplineJob {
    events: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>,
    held: std::sync::Mutex<Option<String>>,
    handle: tokio::task::JoinHandle<()>,
}

thread_local! {
    static LAST_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_error(message: impl Into<String>) {
    let message = message.into();
    LAST_ERROR.with(|slot| slot.replace(message));
}

fn runtime() -> Option<&'static tokio::runtime::Runtime> {
    static RUNTIME: OnceLock<Option<tokio::runtime::Runtime>> = OnceLock::new();
    RUNTIME
        .get_or_init(|| {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .thread_name("tapline-ffi")
                .build()
                .ok()
        })
        .as_ref()
}

static TOTAL_CONCURRENCY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn shared() -> Option<&'static std::sync::Arc<Shared>> {
    pool().map(|pool| pool.budget())
}

fn pool() -> Option<&'static std::sync::Arc<SessionPool>> {
    static POOL: OnceLock<std::sync::Arc<SessionPool>> = OnceLock::new();
    Some(POOL.get_or_init(|| {
        let configured = TOTAL_CONCURRENCY.load(std::sync::atomic::Ordering::Relaxed) as usize;
        let budget = if configured == 0 {
            Shared::new(InstallOptions::default().concurrency)
        } else {
            Shared::new(configured)
        };
        std::sync::Arc::new(SessionPool::with_shared(budget))
    }))
}

#[unsafe(no_mangle)]
pub extern "C" fn tapline_set_total_concurrency(chunks: u32) -> i32 {
    TOTAL_CONCURRENCY.store(chunks, std::sync::atomic::Ordering::Relaxed);
    if STARTED.load(std::sync::atomic::Ordering::Relaxed) {
        set_error("the concurrency budget is already in use and cannot be resized");
        return TAPLINE_BAD_ARGUMENT;
    }
    TAPLINE_OK
}

static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

#[unsafe(no_mangle)]
pub extern "C" fn tapline_total_concurrency() -> u32 {
    shared().map_or(0, |shared| shared.concurrency() as u32)
}

#[unsafe(no_mangle)]
pub extern "C" fn tapline_available_concurrency() -> u32 {
    shared().map_or(0, |shared| shared.available() as u32)
}

fn build_extensions(
    names: Option<&str>,
) -> Result<Vec<std::sync::Arc<dyn tapline::Extension>>, String> {
    let Some(names) = names else {
        return Ok(Vec::new());
    };
    let mut out: Vec<std::sync::Arc<dyn tapline::Extension>> = Vec::new();
    for name in names.split(',').map(str::trim).filter(|n| !n.is_empty()) {
        match name {
            "gmad" => out.push(std::sync::Arc::new(tapline_gmad::Extract::new())),
            "gmad!" => out.push(std::sync::Arc::new(
                tapline_gmad::Extract::new().removing_original(),
            )),
            "gmad-zip" => out.push(std::sync::Arc::new(tapline_gmad::ToZip::new())),
            "gmad-zip!" => out.push(std::sync::Arc::new(
                tapline_gmad::ToZip::new().removing_original(),
            )),
            "gmad-zip-stored" => out.push(std::sync::Arc::new(tapline_gmad::ToZip::new().stored())),
            other => {
                return Err(format!(
                    "unknown extension {other:?}; known: gmad, gmad!, gmad-zip, gmad-zip!, \
                     gmad-zip-stored (a trailing ! deletes the original)"
                ));
            }
        }
    }
    Ok(out)
}

unsafe fn read_str<'a>(pointer: *const c_char) -> Option<&'a str> {
    if pointer.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(pointer) }.to_str().ok()
}

impl TaplineOptions {
    unsafe fn into_install_options(self, dir: &str) -> InstallOptions {
        let defaults = InstallOptions::default();
        InstallOptions {
            install_dir: std::path::PathBuf::from(dir),
            os: match self.os {
                1 => Os::Linux,
                2 => Os::Windows,
                3 => Os::MacOs,
                _ => Os::host(),
            },
            branch: unsafe { read_str(self.branch) }
                .unwrap_or("public")
                .to_owned(),
            include_dlc: self.include_dlc != 0,
            force: self.validate != 0,
            concurrency: if self.concurrency == 0 {
                defaults.concurrency
            } else {
                self.concurrency as usize
            },
            file_modes: if self.file_modes == 1 {
                FileModes::Manifest
            } else {
                FileModes::SteamCmd
            },
            ..defaults
        }
    }
}

fn spawn_job<F, Fut>(build: F, out: *mut *mut TaplineJob) -> i32
where
    F: FnOnce(tokio::sync::mpsc::UnboundedSender<String>) -> Fut + Send + 'static,
    Fut: std::future::Future<Output = ()> + Send + 'static,
{
    if out.is_null() {
        set_error("the job pointer was null");
        return TAPLINE_BAD_ARGUMENT;
    }
    let Some(runtime) = runtime() else {
        set_error("the tapline runtime could not be started");
        return TAPLINE_BAD_ARGUMENT;
    };

    STARTED.store(true, std::sync::atomic::Ordering::Relaxed);

    let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
    let handle = runtime.spawn(build(sender));

    let job = Box::new(TaplineJob {
        events: std::sync::Mutex::new(receiver),
        held: std::sync::Mutex::new(None),
        handle,
    });
    unsafe { *out = Box::into_raw(job) };
    TAPLINE_OK
}

fn send_error(sender: &tokio::sync::mpsc::UnboundedSender<String>, message: &str) {
    let mut out = String::from("{");
    json::push_str_field(&mut out, "kind", "error");
    json::push_str_field(&mut out, "message", message);
    out.push('}');
    let _ = sender.send(out);
}

/// # Safety
/// `dir` must be a valid C string and `out` a writable pointer.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn tapline_install(
    app_id: u32,
    dir: *const c_char,
    branch: *const c_char,
    concurrency: u32,
    os: u8,
    validate: u8,
    include_dlc: u8,
    file_modes: u8,
    extensions: *const c_char,
    out: *mut *mut TaplineJob,
) -> i32 {
    let Some(dir) = (unsafe { read_str(dir) }) else {
        set_error("the install directory was null or not UTF-8");
        return TAPLINE_BAD_ARGUMENT;
    };
    let options = TaplineOptions {
        concurrency,
        branch,
        os,
        validate,
        include_dlc,
        file_modes,
    };
    let install = unsafe { options.into_install_options(dir) };

    let extensions = match build_extensions(unsafe { read_str(extensions) }) {
        Ok(extensions) => extensions,
        Err(message) => {
            set_error(message);
            return TAPLINE_BAD_ARGUMENT;
        }
    };

    let pool_handle = pool().cloned();
    spawn_job(
        move |sender| async move {
            let Some(pool_handle) = pool_handle else {
                send_error(&sender, "the tapline runtime could not be started");
                return;
            };
            match pool_handle.acquire().await {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(mut session) => {
                    for extension in extensions {
                        session.register(extension);
                    }
                    let forward = sender.clone();
                    let result = session
                        .install_observed(AppId(app_id), &install, &mut |event| {
                            let _ = forward.send(event::encode(&event));
                        })
                        .await;
                    match result {
                        Ok(report) => {
                            let _ = sender.send(event::encode_report(&report));
                        }
                        Err(error) => send_error(&sender, &error.to_string()),
                    }
                }
            }
        },
        out,
    )
}

/// # Safety
/// Same as [`tapline_install`].
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_plan(
    app_id: u32,
    dir: *const c_char,
    branch: *const c_char,
    os: u8,
    include_dlc: u8,
    out: *mut *mut TaplineJob,
) -> i32 {
    let Some(dir) = (unsafe { read_str(dir) }) else {
        set_error("the install directory was null or not UTF-8");
        return TAPLINE_BAD_ARGUMENT;
    };
    let options = TaplineOptions {
        branch,
        os,
        include_dlc,
        ..DEFAULTS
    };
    let install = unsafe { options.into_install_options(dir) };

    let pool_handle = pool().cloned();
    spawn_job(
        move |sender| async move {
            let Some(pool_handle) = pool_handle else {
                send_error(&sender, "the tapline runtime could not be started");
                return;
            };
            match pool_handle.acquire().await {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(mut session) => match session.plan(AppId(app_id), &install).await {
                    Ok(plan) => {
                        let _ = sender.send(event::encode_plan(&plan));
                    }
                    Err(error) => send_error(&sender, &error.to_string()),
                },
            }
        },
        out,
    )
}

/// # Safety
/// Same as [`tapline_install`].
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn tapline_workshop_download(
    app_id: u32,
    item_id: u64,
    dir: *const c_char,
    concurrency: u32,
    flat: u8,
    extensions: *const c_char,
    stream: u8,
    out: *mut *mut TaplineJob,
) -> i32 {
    let Some(dir) = (unsafe { read_str(dir) }) else {
        set_error("the download directory was null or not UTF-8");
        return TAPLINE_BAD_ARGUMENT;
    };
    let options = TaplineOptions {
        concurrency,
        ..DEFAULTS
    };
    let mut install = unsafe { options.into_install_options(dir) };
    install.workshop_layout = if flat == 0 {
        WorkshopLayout::SteamCmd
    } else {
        WorkshopLayout::Flat
    };

    let extensions = match build_extensions(unsafe { read_str(extensions) }) {
        Ok(extensions) => extensions,
        Err(message) => {
            set_error(message);
            return TAPLINE_BAD_ARGUMENT;
        }
    };
    let _ = app_id;

    let pool_handle = pool().cloned();
    spawn_job(
        move |sender| async move {
            let Some(pool_handle) = pool_handle else {
                send_error(&sender, "the tapline runtime could not be started");
                return;
            };
            match pool_handle.acquire().await {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(mut session) => {
                    for extension in extensions {
                        session.register(extension);
                    }
                    let item = PublishedFileId(item_id);
                    let described = match session.workshop_details(&[item]).await {
                        Ok(described) => described,
                        Err(error) => {
                            send_error(&sender, &error.to_string());
                            return;
                        }
                    };
                    let details = match described.into_iter().next() {
                        Some(Ok(details)) => details,
                        Some(Err(error)) => {
                            send_error(&sender, &error.to_string());
                            return;
                        }
                        None => {
                            send_error(&sender, &format!("Steam said nothing about item {item}"));
                            return;
                        }
                    };
                    if stream != 0 {
                        stream_addon(&mut session, &details, &install, stream, &sender).await;
                        return;
                    }

                    let forward = sender.clone();
                    let result = session
                        .download_workshop_item_observed(&details, &install, &mut |event| {
                            let _ = forward.send(event::encode(&event));
                        })
                        .await;
                    match result {
                        Ok(report) => {
                            let _ = sender.send(event::encode_report(&report));
                        }
                        Err(error) => send_error(&sender, &error.to_string()),
                    }
                }
            }
        },
        out,
    )
}

/// # Safety
/// Every pointer must be a valid C string or null; `out` writable.
#[unsafe(no_mangle)]
#[allow(clippy::too_many_arguments)]
pub unsafe extern "C" fn tapline_workshop_search(
    app_id: u32,
    text: *const c_char,
    search_in: *const c_char,
    tags: *const c_char,
    tag_groups: *const c_char,
    excluded_tags: *const c_char,
    excluded_content: *const c_char,
    all_tags: u8,
    sort: *const c_char,
    trend_days: u32,
    created_since: u32,
    created_until: u32,
    updated_since: u32,
    updated_until: u32,
    limit: u32,
    cursor: *const c_char,
    page: u32,
    count_only: u8,
    out: *mut *mut TaplineJob,
) -> i32 {
    fn split(list: Option<&str>) -> Vec<String> {
        list.map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|part| !part.is_empty())
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
    }

    fn window(since: u32, until: u32) -> Option<tapline::TimeRange> {
        (since != 0 || until != 0).then(|| tapline::TimeRange {
            start: (since != 0).then_some(since),
            end: (until != 0).then_some(until),
        })
    }

    fn groups(raw: Option<&str>) -> Vec<Vec<String>> {
        raw.map(|raw| {
            raw.split(';')
                .map(|group| split(Some(group)))
                .filter(|group: &Vec<String>| !group.is_empty())
                .collect()
        })
        .unwrap_or_default()
    }

    let defaults = tapline::BrowseQuery::default();
    let mut excluded_descriptors = Vec::new();
    for name in split(unsafe { read_str(excluded_content) }) {
        match tapline::ContentDescriptor::parse(&name) {
            Some(descriptor) => excluded_descriptors.push(descriptor),
            None => {
                set_error(format!(
                    "unknown content label {name:?}; known: {}",
                    tapline::ContentDescriptor::NAMES.join(", ")
                ));
                return TAPLINE_BAD_ARGUMENT;
            }
        }
    }
    let search_in = match unsafe { read_str(search_in) } {
        None => tapline::TextTarget::default(),
        Some(name) => match tapline::TextTarget::parse(name) {
            Some(target) => target,
            None => {
                set_error(format!(
                    "unknown search target {name:?}; known: {}",
                    tapline::TextTarget::NAMES.join(", ")
                ));
                return TAPLINE_BAD_ARGUMENT;
            }
        },
    };
    let sort = match unsafe { read_str(sort) } {
        None => defaults.sort,
        Some(name) => match tapline::BrowseSort::parse(name) {
            Some(sort) => sort,
            None => {
                set_error(format!(
                    "unknown sort {name:?}; known: {}",
                    tapline::BrowseSort::NAMES.join(", ")
                ));
                return TAPLINE_BAD_ARGUMENT;
            }
        },
    };

    let query = tapline::BrowseQuery {
        app: AppId(app_id),
        text: unsafe { read_str(text) }.map(str::to_owned),
        search_in,
        required_tags: split(unsafe { read_str(tags) }),
        tag_groups: groups(unsafe { read_str(tag_groups) }),
        excluded_tags: split(unsafe { read_str(excluded_tags) }),
        excluded_descriptors,
        created: window(created_since, created_until),
        updated: window(updated_since, updated_until),
        match_all_tags: all_tags != 0,
        sort,
        trend_days: if trend_days == 0 {
            None
        } else {
            Some(trend_days)
        },
        per_page: if limit == 0 { defaults.per_page } else { limit },
        cursor: unsafe { read_str(cursor) }.map(str::to_owned),
        page: (page != 0).then_some(page),
    };
    if let Err(error) = query.validate() {
        set_error(error.to_string());
        return TAPLINE_BAD_ARGUMENT;
    }

    let pool_handle = pool().cloned();
    spawn_job(
        move |sender| async move {
            let Some(pool_handle) = pool_handle else {
                send_error(&sender, "the tapline runtime could not be started");
                return;
            };
            match pool_handle.acquire().await {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(mut session) if count_only != 0 => match session.count_workshop(&query).await {
                    Err(error) => send_error(&sender, &error.to_string()),
                    Ok(total) => {
                        let mut out = String::from("{");
                        json::push_str_field(&mut out, "kind", "counted");
                        json::push_u64(&mut out, "total", u64::from(total));
                        out.push('}');
                        let _ = sender.send(out);
                    }
                },
                Ok(mut session) => match session.browse_workshop(&query).await {
                    Err(error) => send_error(&sender, &error.to_string()),
                    Ok(page) => {
                        for found in &page.items {
                            let mut out = String::from("{");
                            json::push_str_field(&mut out, "kind", "result");
                            json::push_u64(&mut out, "app", u64::from(found.item.app.get()));
                            json::push_str_field(
                                &mut out,
                                "item",
                                &found.item.id.get().to_string(),
                            );
                            json::push_str_field(&mut out, "title", &found.item.title);
                            json::push_str_field(&mut out, "description", &found.description);
                            json::push_u64(&mut out, "size", found.item.size);
                            json::push_u64(&mut out, "updated", u64::from(found.item.updated));
                            json::push_u64(&mut out, "created", u64::from(found.created));
                            json::push_str_field(
                                &mut out,
                                "creator",
                                &found.creator.map(|id| id.to_string()).unwrap_or_default(),
                            );
                            json::push_u64(&mut out, "subscriptions", found.subscriptions);
                            json::push_u64(&mut out, "favorites", found.favorites);
                            json::push_u64(&mut out, "views", found.views);
                            json::push_u64(&mut out, "votesUp", found.votes_up);
                            json::push_u64(&mut out, "votesDown", found.votes_down);
                            json::push_str_field(
                                &mut out,
                                "previewUrl",
                                found.preview_url.as_deref().unwrap_or(""),
                            );
                            json::push_key(&mut out, "tags");
                            out.push('[');
                            for (index, tag) in found.tags.iter().enumerate() {
                                if index > 0 {
                                    out.push(',');
                                }
                                json::push_string(&mut out, tag);
                            }
                            out.push(']');
                            out.push('}');
                            let _ = sender.send(out);
                        }

                        let mut out = String::from("{");
                        json::push_str_field(&mut out, "kind", "searched");
                        json::push_u64(&mut out, "total", u64::from(page.total));
                        json::push_u64(&mut out, "returned", page.items.len() as u64);
                        json::push_u64(&mut out, "skipped", page.skipped.len() as u64);
                        json::push_str_field(
                            &mut out,
                            "nextCursor",
                            page.next_cursor.as_deref().unwrap_or(""),
                        );
                        out.push('}');
                        let _ = sender.send(out);
                    }
                },
            }
        },
        out,
    )
}

/// # Safety
/// `spec` must be a valid C string or null; `out` writable.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_pipeline(
    app_id: u32,
    item_id: u64,
    spec: *const c_char,
    concurrency: u32,
    out: *mut *mut TaplineJob,
) -> i32 {
    let Some(spec) = (unsafe { read_str(spec) }) else {
        set_error("the pipeline was null or not UTF-8");
        return TAPLINE_BAD_ARGUMENT;
    };

    let pipeline = match tapline_pipe::Pipeline::parse(spec) {
        Ok(pipeline) => pipeline,
        Err(error) => {
            set_error(error.to_string());
            return TAPLINE_BAD_ARGUMENT;
        }
    };
    if let Err(error) = pipeline.validate() {
        set_error(error.to_string());
        return TAPLINE_BAD_ARGUMENT;
    }

    let window = if concurrency == 0 {
        tapline::Window::default()
    } else {
        tapline::Window::new(concurrency as usize)
    };

    let pool_handle = pool().cloned();
    spawn_job(
        move |sender| async move {
            let Some(pool_handle) = pool_handle else {
                send_error(&sender, "the tapline runtime could not be started");
                return;
            };
            match pool_handle.acquire().await {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(mut session) => {
                    let forward = sender.clone();
                    let result = tapline_pipe::run_pipeline(
                        &mut session,
                        AppId(app_id),
                        PublishedFileId(item_id),
                        window,
                        &pipeline,
                        &mut |event| {
                            let _ = forward.send(event::encode(&event));
                        },
                    )
                    .await;

                    match result {
                        Err(error) => send_error(&sender, &error.to_string()),
                        Ok(outcome) => {
                            let mut out = String::from("{");
                            json::push_str_field(&mut out, "kind", "piped");
                            json::push_u64(&mut out, "entries", outcome.entries as u64);
                            json::push_u64(&mut out, "bytesDownloaded", outcome.bytes_downloaded);
                            json::push_u64(&mut out, "bytesStreamed", outcome.bytes_streamed);
                            json::push_u64(
                                &mut out,
                                "peakBufferedChunks",
                                outcome.peak_buffered as u64,
                            );
                            out.push('}');
                            let _ = sender.send(out);
                        }
                    }
                }
            }
        },
        out,
    )
}

async fn stream_addon(
    session: &mut Session,
    details: &tapline::WorkshopItem,
    options: &InstallOptions,
    mode: u8,
    sender: &tokio::sync::mpsc::UnboundedSender<String>,
) {
    if let Err(error) = std::fs::create_dir_all(&options.install_dir) {
        send_error(sender, &error.to_string());
        return;
    }

    let zip_path = options.install_dir.join(format!("{}.zip", details.id));
    let target = match mode {
        2 => tapline_gmad::StreamTarget::Zip(&zip_path),
        3 => tapline_gmad::StreamTarget::ZipStored(&zip_path),
        _ => tapline_gmad::StreamTarget::Directory(&options.install_dir),
    };
    let mut extractor = match tapline_gmad::StreamWriter::new(target) {
        Ok(writer) => writer,
        Err(error) => {
            send_error(sender, &error.to_string());
            return;
        }
    };

    let forward = sender.clone();
    let result = session
        .stream_workshop_item(
            details,
            tapline::Window::default(),
            &mut |bytes| {
                extractor
                    .push(bytes)
                    .map_err(|error| tapline::InstallError::Io(error.to_string()))
            },
            &mut |event| {
                let _ = forward.send(event::encode(&event));
            },
        )
        .await;

    match result {
        Err(error) => send_error(sender, &error.to_string()),
        Ok(report) => match extractor.finish() {
            Err(error) => send_error(sender, &error.to_string()),
            Ok(produced) => {
                let mut out = String::from("{");
                json::push_str_field(&mut out, "kind", "streamed");
                json::push_u64(&mut out, "files", produced.entries as u64);
                json::push_u64(&mut out, "bytesDownloaded", report.bytes_downloaded);
                json::push_u64(&mut out, "bytesStreamed", report.bytes_streamed);
                json::push_u64(&mut out, "chunks", report.chunks);
                json::push_u64(&mut out, "peakBufferedChunks", report.peak_buffered as u64);
                out.push('}');
                let _ = sender.send(out);
            }
        },
    }
}

const DEFAULTS: TaplineOptions = TaplineOptions {
    concurrency: 0,
    branch: std::ptr::null(),
    os: 0,
    validate: 0,
    include_dlc: 0,
    file_modes: 0,
};

/// # Safety
/// `job` must be a live job pointer; `buf` writable for `cap` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_job_next(
    job: *mut TaplineJob,
    timeout_ms: u32,
    buf: *mut u8,
    cap: usize,
    out_len: *mut usize,
) -> i32 {
    let Some(job) = (unsafe { job.as_ref() }) else {
        set_error("the job pointer was null");
        return TAPLINE_BAD_ARGUMENT;
    };
    let Some(runtime) = runtime() else {
        set_error("the tapline runtime is not running");
        return TAPLINE_BAD_ARGUMENT;
    };

    let held = job.held.lock().ok().and_then(|mut slot| slot.take());
    let message = match held {
        Some(message) => Some(message),
        None => {
            let Ok(mut receiver) = job.events.lock() else {
                set_error("the job's event queue was poisoned");
                return TAPLINE_BAD_ARGUMENT;
            };
            if timeout_ms == 0 {
                match receiver.try_recv() {
                    Ok(message) => Some(message),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty) => return TAPLINE_TIMEOUT,
                    Err(tokio::sync::mpsc::error::TryRecvError::Disconnected) => None,
                }
            } else {
                let wait = std::time::Duration::from_millis(u64::from(timeout_ms));
                match runtime.block_on(async { tokio::time::timeout(wait, receiver.recv()).await })
                {
                    Ok(Some(message)) => Some(message),
                    Ok(None) => None,
                    Err(_elapsed) => return TAPLINE_TIMEOUT,
                }
            }
        }
    };

    let Some(message) = message else {
        return TAPLINE_DONE;
    };

    if !out_len.is_null() {
        unsafe { *out_len = message.len() };
    }
    if buf.is_null() || cap < message.len() {
        if let Ok(mut slot) = job.held.lock() {
            *slot = Some(message);
        }
        return TAPLINE_BUFFER_TOO_SMALL;
    }

    unsafe { std::ptr::copy_nonoverlapping(message.as_ptr(), buf, message.len()) };
    TAPLINE_OK
}

/// # Safety
/// `job` must be a live job pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_job_cancel(job: *mut TaplineJob) {
    if let Some(job) = unsafe { job.as_ref() } {
        job.handle.abort();
    }
}

/// # Safety
/// `job` must not be used afterwards; null is allowed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_job_free(job: *mut TaplineJob) {
    if job.is_null() {
        return;
    }
    let job = unsafe { Box::from_raw(job) };
    job.handle.abort();
    drop(job);
}

/// # Safety
/// `buf` must be writable for `cap` bytes, or null to query the length.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_last_error(buf: *mut u8, cap: usize, out_len: *mut usize) -> i32 {
    LAST_ERROR.with(|slot| {
        let message = slot.borrow();
        if !out_len.is_null() {
            unsafe { *out_len = message.len() };
        }
        if buf.is_null() || cap < message.len() {
            return TAPLINE_BUFFER_TOO_SMALL;
        }
        unsafe { std::ptr::copy_nonoverlapping(message.as_ptr(), buf, message.len()) };
        TAPLINE_OK
    })
}

/// # Safety
/// `out` must be a valid pointer to write the job handle to.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_qr_login(timeout_secs: u32, out: *mut *mut TaplineJob) -> i32 {
    let timeout = std::time::Duration::from_secs(if timeout_secs == 0 {
        300
    } else {
        u64::from(timeout_secs)
    });

    spawn_job(
        move |sender| async move {
            let mut session = match Session::anonymous().await {
                Ok(session) => session,
                Err(error) => {
                    send_error(&sender, &error.to_string());
                    return;
                }
            };

            let codes = sender.clone();
            let result = session
                .qr_login(timeout, &mut |url| {
                    let mut out = String::from("{");
                    json::push_str_field(&mut out, "kind", "qr");
                    json::push_str_field(&mut out, "url", url);
                    out.push('}');
                    let _ = codes.send(out);
                })
                .await;

            match result {
                Err(error) => send_error(&sender, &error.to_string()),
                Ok(token) => {
                    if let Err(error) = tapline::TokenStore::default_file().save(&token) {
                        send_error(&sender, &error.to_string());
                        return;
                    }
                    let mut out = String::from("{");
                    json::push_str_field(&mut out, "kind", "loggedIn");
                    json::push_str_field(&mut out, "account", &token.account);
                    out.push('}');
                    let _ = sender.send(out);
                }
            }
        },
        out,
    )
}

#[unsafe(no_mangle)]
pub extern "C" fn tapline_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), "\0").as_ptr().cast()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_version_is_a_readable_c_string() {
        let raw = tapline_version();
        let text = unsafe { CStr::from_ptr(raw) }.to_str();
        assert_eq!(text, Ok(env!("CARGO_PKG_VERSION")));
    }

    #[test]
    fn zeroed_options_mean_the_rust_defaults() {
        let options = DEFAULTS;
        let converted = unsafe { options.into_install_options("/tmp/x") };
        let defaults = InstallOptions::default();
        assert_eq!(converted.concurrency, defaults.concurrency);
        assert_eq!(converted.branch, "public");
        assert_eq!(converted.file_modes, defaults.file_modes);
        assert_eq!(converted.os, Os::host());
        assert!(!converted.force);
        assert!(!converted.include_dlc);
    }

    #[test]
    fn options_are_carried_across_the_boundary() {
        let branch = std::ffi::CString::new("beta").expect("no NUL");
        let options = TaplineOptions {
            concurrency: 8,
            branch: branch.as_ptr(),
            os: 2,
            validate: 1,
            include_dlc: 1,
            file_modes: 1,
        };
        let converted = unsafe { options.into_install_options("/srv/x") };
        assert_eq!(converted.concurrency, 8);
        assert_eq!(converted.branch, "beta");
        assert_eq!(converted.os, Os::Windows);
        assert!(converted.force);
        assert!(converted.include_dlc);
        assert_eq!(converted.file_modes, FileModes::Manifest);
        assert_eq!(converted.install_dir, std::path::PathBuf::from("/srv/x"));
    }

    #[test]
    fn a_null_directory_is_refused_rather_than_guessed() {
        let mut job: *mut TaplineJob = std::ptr::null_mut();
        let code = unsafe {
            tapline_install(
                4020,
                std::ptr::null(),
                std::ptr::null(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null(),
                &raw mut job,
            )
        };
        assert_eq!(code, TAPLINE_BAD_ARGUMENT);
        assert!(job.is_null());
    }

    #[test]
    fn a_null_job_pointer_is_refused() {
        let dir = std::ffi::CString::new("/tmp/x").expect("no NUL");
        let code = unsafe {
            tapline_install(
                4020,
                dir.as_ptr(),
                std::ptr::null(),
                0,
                0,
                0,
                0,
                0,
                std::ptr::null(),
                std::ptr::null_mut(),
            )
        };
        assert_eq!(code, TAPLINE_BAD_ARGUMENT);
    }

    #[test]
    fn the_last_error_is_readable_and_reports_its_length() {
        set_error("something went wrong");
        let mut needed = 0_usize;
        let code = unsafe { tapline_last_error(std::ptr::null_mut(), 0, &raw mut needed) };
        assert_eq!(code, TAPLINE_BUFFER_TOO_SMALL);
        assert_eq!(needed, "something went wrong".len());

        let mut buf = vec![0_u8; needed];
        let code = unsafe { tapline_last_error(buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_OK);
        assert_eq!(
            String::from_utf8(buf).as_deref(),
            Ok("something went wrong")
        );
    }

    #[test]
    fn freeing_a_null_job_is_allowed() {
        unsafe { tapline_job_free(std::ptr::null_mut()) };
        unsafe { tapline_job_cancel(std::ptr::null_mut()) };
    }

    #[test]
    fn a_blocking_wait_does_not_panic_outside_a_runtime() {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel::<String>();
        let Some(runtime) = runtime() else {
            return;
        };
        let handle = runtime.spawn(async {});
        let job = Box::into_raw(Box::new(TaplineJob {
            events: std::sync::Mutex::new(receiver),
            held: std::sync::Mutex::new(None),
            handle,
        }));

        let mut needed = 0_usize;
        let mut buf = [0_u8; 64];
        let code =
            unsafe { tapline_job_next(job, 20, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_TIMEOUT);

        let _ = sender.send(String::from(r#"{"kind":"progress"}"#));
        let code =
            unsafe { tapline_job_next(job, 500, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_OK);

        unsafe { tapline_job_free(job) };
    }

    #[test]
    fn an_event_too_big_for_the_buffer_is_kept_not_dropped() {
        let (sender, receiver) = tokio::sync::mpsc::unbounded_channel();
        let Some(runtime) = runtime() else {
            return;
        };
        let handle = runtime.spawn(async {});
        let job = Box::into_raw(Box::new(TaplineJob {
            events: std::sync::Mutex::new(receiver),
            held: std::sync::Mutex::new(None),
            handle,
        }));
        let _ = sender.send(String::from(r#"{"kind":"progress"}"#));

        let mut needed = 0_usize;
        let mut tiny = [0_u8; 2];
        let code =
            unsafe { tapline_job_next(job, 0, tiny.as_mut_ptr(), tiny.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_BUFFER_TOO_SMALL);
        assert_eq!(needed, r#"{"kind":"progress"}"#.len());

        let mut buf = vec![0_u8; needed];
        let code =
            unsafe { tapline_job_next(job, 0, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_OK);
        assert_eq!(
            std::str::from_utf8(&buf).ok(),
            Some(r#"{"kind":"progress"}"#)
        );

        let code =
            unsafe { tapline_job_next(job, 0, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_TIMEOUT);

        drop(sender);
        let code =
            unsafe { tapline_job_next(job, 0, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_DONE);

        unsafe { tapline_job_free(job) };
    }
}
