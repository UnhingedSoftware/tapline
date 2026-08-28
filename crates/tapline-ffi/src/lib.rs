//! The C ABI for tapline.
//!
//! One shared library that Deno, Bun and Node all load, plus a TypeScript
//! wrapper per runtime in `bindings/js`.
//!
//! # Why no callbacks cross this boundary
//!
//! The obvious shape for "call me with progress" is a function pointer. It is
//! the wrong one here. Deno needs `Deno.UnsafeCallback.threadSafe` plus manual
//! `ref()`/`unref()` to keep the isolate alive; Bun's `JSCallback` invoked from
//! a non-JS thread is fragile; Node has no built-in FFI at all. Worse, all
//! three share the same failure mode — a download thread calling into an
//! isolate that has been torn down takes the whole process with it, and it
//! happens at exit, intermittently, in someone else's application.
//!
//! So nothing here takes a function pointer. A job runs on tapline's own
//! runtime and pushes events into a queue; the caller pulls them with
//! [`tapline_job_next`], which is an ordinary blocking call with a timeout.
//! Deno marks it `nonblocking` and gets a real `Promise`; Node's koffi has
//! `.async()`; Bun polls it with a zero timeout. The JS wrapper turns that one
//! primitive into promises, async iterators and `onProgress` callbacks, which
//! is where that sort of thing belongs.
//!
//! # Memory
//!
//! The caller owns every buffer. Nothing here allocates something the caller
//! must free except the opaque [`TaplineJob`], which [`tapline_job_free`]
//! releases. Strings are written into caller-provided buffers, so there is no
//! `tapline_string_free` to forget.

mod event;
mod json;

use std::ffi::{CStr, c_char};
use std::sync::OnceLock;
use tapline::{
    FileModes, InstallOptions, Os, PublishedFileId, Session, SessionPool, Shared, WorkshopLayout,
};
use tapline_ids::AppId;

/// An event was written to the buffer.
pub const TAPLINE_OK: i32 = 0;
/// The timeout elapsed with no event. Not an error: call again.
pub const TAPLINE_TIMEOUT: i32 = 1;
/// The job is over and no further events will arrive.
pub const TAPLINE_DONE: i32 = 2;
/// The buffer was too small. The required length was written to `out_len` and
/// the event is kept, so calling again with a bigger buffer returns it.
pub const TAPLINE_BUFFER_TOO_SMALL: i32 = -1;
/// An argument was unusable — a null pointer, or a string that is not UTF-8.
pub const TAPLINE_BAD_ARGUMENT: i32 = -2;

/// Install options, as loose scalars rather than a struct.
///
/// A `#[repr(C)]` struct would be tidier to read and worse to use. Every one of
/// the three target runtimes would have to reproduce Rust's field order and
/// padding by hand — 4 bytes, 4 of padding, an 8-byte pointer, then four
/// bytes — and getting it wrong does not raise an error. It reads the branch
/// pointer from the wrong offset and dereferences whatever is there. Scalar
/// parameters cannot be misaligned.
///
/// Zero means "the default" everywhere, so a caller passing nothing but the app
/// id and directory gets exactly `InstallOptions::default()`.
#[derive(Debug, Clone, Copy)]
pub struct TaplineOptions {
    /// Chunks in flight. 0 uses tapline's default.
    pub concurrency: u32,
    /// Branch name, or null for `public`.
    pub branch: *const c_char,
    /// 0 host, 1 linux, 2 windows, 3 macos. Anything else is the host.
    pub os: u8,
    /// Non-zero re-downloads even when the install record says it is current.
    pub validate: u8,
    /// Non-zero includes DLC depots.
    pub include_dlc: u8,
    /// 0 matches steamcmd's permissions, 1 uses the manifest's.
    pub file_modes: u8,
}

/// A running job. Opaque.
pub struct TaplineJob {
    /// Events, as JSON, in the order they happened.
    events: std::sync::Mutex<tokio::sync::mpsc::UnboundedReceiver<String>>,
    /// An event that did not fit the caller's buffer, kept so it is not lost.
    held: std::sync::Mutex<Option<String>>,
    /// Lets [`tapline_job_cancel`] stop the work.
    handle: tokio::task::JoinHandle<()>,
}

thread_local! {
    /// The last error on this thread, for [`tapline_last_error`].
    static LAST_ERROR: std::cell::RefCell<String> = const { std::cell::RefCell::new(String::new()) };
}

fn set_error(message: impl Into<String>) {
    let message = message.into();
    LAST_ERROR.with(|slot| slot.replace(message));
}

/// The runtime every job runs on.
///
/// One for the process, built on first use. A JS host already has its own event
/// loop and must not be asked to host ours, so tapline brings its own threads
/// and never blocks the caller's except inside [`tapline_job_next`], where the
/// caller asked for it.
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

/// The chunk budget every job in this process draws from.
///
/// Set once, on first use. Two downloads started from JavaScript are two jobs
/// in one process, and without this they would take a full budget each — which
/// is measurably slower than sharing one, because the throughput curve turns
/// over after 64 chunks in flight. Sharing also means a connection warmed by
/// one download is warm for the next.
static TOTAL_CONCURRENCY: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);

fn shared() -> Option<&'static std::sync::Arc<Shared>> {
    pool().map(|pool| pool.budget())
}

/// The pool every job in this process draws sessions from.
///
/// One login is reused across jobs rather than paid per job, and concurrent
/// jobs get different sessions so none waits on another. They still share one
/// chunk budget, so pooling sessions does not multiply the load on the CDN.
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

/// Sets the total chunks in flight across every job in this process.
///
/// Must be called before the first job starts; afterwards the budget is fixed
/// and this returns [`TAPLINE_BAD_ARGUMENT`], because moving it underneath
/// downloads already drawing on it is not something a caller can reason about.
///
/// 0 restores the default.
#[unsafe(no_mangle)]
pub extern "C" fn tapline_set_total_concurrency(chunks: u32) -> i32 {
    TOTAL_CONCURRENCY.store(chunks, std::sync::atomic::Ordering::Relaxed);
    // If a job already built the budget, saying so is better than pretending.
    if STARTED.load(std::sync::atomic::Ordering::Relaxed) {
        set_error("the concurrency budget is already in use and cannot be resized");
        return TAPLINE_BAD_ARGUMENT;
    }
    TAPLINE_OK
}

/// Whether any job has been started, which fixes the budget.
static STARTED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// The total chunks in flight allowed across the process.
#[unsafe(no_mangle)]
pub extern "C" fn tapline_total_concurrency() -> u32 {
    shared().map_or(0, |shared| shared.concurrency() as u32)
}

/// How much of that budget is free right now.
#[unsafe(no_mangle)]
pub extern "C" fn tapline_available_concurrency() -> u32 {
    shared().map_or(0, |shared| shared.available() as u32)
}

/// Builds the extensions named in a comma-separated list.
///
/// Names, not function pointers. The C ABI deliberately carries no callbacks —
/// see the module docs — and an extension is code compiled into this library
/// rather than supplied by the caller, so selecting one by name is the whole
/// interface. An unknown name is an error: silently running nothing is how a
/// caller ends up with a directory of `.gma` files it thought were unpacked.
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

/// Reads a C string, or `None` if it is null or not UTF-8.
///
/// # Safety
///
/// `pointer` must be null or a NUL-terminated string valid for reads.
unsafe fn read_str<'a>(pointer: *const c_char) -> Option<&'a str> {
    if pointer.is_null() {
        return None;
    }
    unsafe { CStr::from_ptr(pointer) }.to_str().ok()
}

impl TaplineOptions {
    /// Turns the C struct into the Rust options.
    ///
    /// # Safety
    ///
    /// `self.branch` must be null or a valid C string.
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

/// Spawns a job around a future that produces events.
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

    // Fixes the budget: from here on it is in use.
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

/// Sends an error event, then ends the stream.
fn send_error(sender: &tokio::sync::mpsc::UnboundedSender<String>, message: &str) {
    let mut out = String::from("{");
    json::push_str_field(&mut out, "kind", "error");
    json::push_str_field(&mut out, "message", message);
    out.push('}');
    let _ = sender.send(out);
}

/// Starts an install.
///
/// Writes an opaque job to `out`. Events arrive through [`tapline_job_next`],
/// ending with one of `finished`, `error` or `cancelled`.
///
/// `extensions` is a comma-separated list of built-in extension names, or null
/// for none. See [`build_extensions`].
///
/// # Safety
///
/// `dir` must be a valid C string, `options` a valid pointer or null, and `out`
/// a writable pointer.
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

    // Resolved before the job starts, so a typo is reported now rather than
    // after a download has already run.
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
                            // Send failures mean the caller freed the job; the
                            // download is aborted with it, so there is nothing
                            // to report to.
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

/// Computes what an install would cost, without fetching content.
///
/// Emits exactly one `planned` event, then ends.
///
/// # Safety
///
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

/// Downloads one Workshop item.
///
/// `flat` non-zero writes the item's files straight into `dir`; zero uses
/// steamcmd's `steamapps/workshop/content/<app>/<item>/` layout.
///
/// `extensions` is a comma-separated list of built-in extension names, or null
/// for none. See [`build_extensions`] for what is known.
///
/// `stream` selects a streaming target, writing the addon as it downloads and
/// never storing the `.gma`: 0 off, 1 unpack into `dir`, 2 write a `.zip`,
/// 3 write a `.zip` without deflating. Any of the streaming modes imply the
/// flat layout and ignore `extensions`, because the archive those would act on
/// never exists.
///
/// # Safety
///
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
    // Non-zero writes the item's files into `dir` itself. A Garry's Mod addon
    // belongs in garrysmod/addons, and under the steamcmd layout it would land
    // four directories below where the server looks for it.
    install.workshop_layout = if flat == 0 {
        WorkshopLayout::SteamCmd
    } else {
        WorkshopLayout::Flat
    };

    // Resolved before the job starts, so a typo is reported now rather than
    // after a download has already run.
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

/// Searches an app's Workshop.
///
/// Query parts cross as scalars and strings rather than a struct, the same
/// rule every other entry point here follows: a `repr(C)` struct would make
/// each runtime reproduce Rust's padding by hand.
///
/// `tags` and `excluded_tags` are comma-separated, `sort` is one of the names
/// `BrowseSort` accepts, and `cursor` is the `nextCursor` from a previous page
/// or null for the first. Results arrive as `result` events followed by one
/// `searched` event carrying the totals.
///
/// `tag_groups` is Steam's sidebar: groups separated by `;`, tags within a
/// group by `,`, so `"Scene,Video;Anime"` means *(Scene or Video) and Anime*.
/// A flat tag list cannot express that, which is the whole reason it is a
/// second parameter rather than more of the first.
///
/// `count_only` asks how many match and fetches none of them, which is what a
/// filter list showing a number beside each option wants. It emits a single
/// `counted` event instead of any `result` or `searched` event.
///
/// `search_in` narrows where `text` is matched: `all`, `title` or
/// `description`. Narrowing without text is refused, since there is nothing to
/// narrow.
///
/// `excluded_content` is a comma-separated list of Steam's own content labels
/// — `nudity`, `violence`, `adult-only`, `gratuitous`, `mature` — which are a
/// truer filter than excluding a tag by name, because the label is Valve's
/// rather than whatever the author ticked.
///
/// The four date bounds are Unix seconds, and zero is unset — a search for
/// items published or updated in a window, which filters hard rather than
/// reordering. A window whose end precedes its start is refused.
///
/// `trend_days` is the period a `trend` sort ranks over. Zero means unset,
/// which is also what Steam does with a zero it is sent, and it applies to no
/// other sort — passing one elsewhere is refused rather than ignored.
///
/// # Safety
///
/// Every pointer must be a null-terminated UTF-8 string or null, and `out` must
/// be a valid pointer to write the job handle to.
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

    // Zero is unset rather than 1970, which no Workshop item predates anyway.
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
                // An empty group would be refused by validate; dropping the
                // empties first means a trailing `;` is a typo rather than an
                // error, the same way an empty tag in a list is skipped.
                .filter(|group: &Vec<String>| !group.is_empty())
                .collect()
        })
        .unwrap_or_default()
    }

    let defaults = tapline::BrowseQuery::default();
    // Resolved before the job starts, so a bad name is a synchronous error
    // rather than one delivered through the event queue.
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
    // Resolved before the job starts, so a bad sort name is a synchronous
    // error rather than one delivered through the event queue.
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
                            // A string: item ids exceed what a JSON number
                            // holds exactly, and a rounded id is a different
                            // item.
                            json::push_str_field(
                                &mut out,
                                "item",
                                &found.item.id.get().to_string(),
                            );
                            json::push_str_field(&mut out, "title", &found.item.title);
                            json::push_str_field(&mut out, "description", &found.description);
                            json::push_u64(&mut out, "size", found.item.size);
                            json::push_u64(&mut out, "updated", u64::from(found.item.updated));
                            json::push_u64(&mut out, "subscriptions", found.subscriptions);
                            json::push_u64(&mut out, "favorites", found.favorites);
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

/// Runs a pipeline given in its text form.
///
/// The chain in `tapline-pipe` is a Rust API and cannot cross a C ABI, so what
/// travels is the text form it builds — one directive per line:
///
/// ```text
/// decode gma
/// only lua/**
/// zip /srv/out.zip
/// ```
///
/// This is the entry point every binding's chain compiles down to. It is also
/// what makes a filtered download cheap from JavaScript: a pipeline that
/// selects part of an archive fetches only the chunks that part lives in,
/// which the existing `tapline_workshop_download` has no way to express.
///
/// # Safety
///
/// `spec` must be a null-terminated UTF-8 string or null, and `out` must be a
/// valid pointer to write the job handle to.
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

    // Parsed and validated before the job starts, so a bad directive is a
    // synchronous error rather than a failure delivered through the event
    // queue after a download has already begun.
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

/// Streams a Workshop item into a Garry's Mod extractor.
///
/// Kept out of the job body because the borrow of the extractor by the consumer
/// closure has to end before the extractor can be finished.
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

/// The options used when a caller passes null.
const DEFAULTS: TaplineOptions = TaplineOptions {
    concurrency: 0,
    branch: std::ptr::null(),
    os: 0,
    validate: 0,
    include_dlc: 0,
    file_modes: 0,
};

/// Waits for the next event and writes it to `buf` as UTF-8 JSON.
///
/// Returns [`TAPLINE_OK`] with the length in `out_len`, [`TAPLINE_TIMEOUT`] if
/// `timeout_ms` elapsed first, [`TAPLINE_DONE`] when the job is over, or
/// [`TAPLINE_BUFFER_TOO_SMALL`] with the needed length in `out_len` — in which
/// case the event is kept and the next call returns it.
///
/// A `timeout_ms` of 0 polls without blocking, which is what a runtime with no
/// async FFI should use.
///
/// # Safety
///
/// `job` must come from a `tapline_*` start function and not have been freed.
/// `buf` must be writable for `cap` bytes.
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

    // An event that did not fit last time takes priority over a new one, or the
    // stream would reorder itself under a small buffer.
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
                // Constructed inside the async block, not outside it.
                // `tokio::time::timeout` builds a timer at the moment it is
                // called, and a timer built outside a runtime context panics
                // with "there is no reactor running". Across an FFI boundary a
                // panic cannot unwind, so it aborts — the whole Node process,
                // in someone else's application. Bun never reached this line
                // because it polls with a zero timeout; Node passes 250ms and
                // dumped core on the first live call.
                match runtime.block_on(async { tokio::time::timeout(wait, receiver.recv()).await })
                {
                    Ok(Some(message)) => Some(message),
                    // The senders are gone: the job finished.
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
        // Keep it rather than drop it: the caller is expected to retry with a
        // buffer of the length just reported, and losing the event would make
        // that retry silently skip one.
        if let Ok(mut slot) = job.held.lock() {
            *slot = Some(message);
        }
        return TAPLINE_BUFFER_TOO_SMALL;
    }

    unsafe { std::ptr::copy_nonoverlapping(message.as_ptr(), buf, message.len()) };
    TAPLINE_OK
}

/// Stops a job.
///
/// The download is abandoned where it stands. Whatever is already on disk stays
/// there, and a later install resumes from it rather than starting over.
///
/// # Safety
///
/// `job` must be a live job pointer.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_job_cancel(job: *mut TaplineJob) {
    if let Some(job) = unsafe { job.as_ref() } {
        job.handle.abort();
    }
}

/// Frees a job, cancelling it first if it is still running.
///
/// # Safety
///
/// `job` must come from a `tapline_*` start function and must not be used
/// afterwards. Passing null is allowed and does nothing.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn tapline_job_free(job: *mut TaplineJob) {
    if job.is_null() {
        return;
    }
    let job = unsafe { Box::from_raw(job) };
    job.handle.abort();
    drop(job);
}

/// Writes the last error on this thread into `buf`.
///
/// Returns the message length, or the needed length with
/// [`TAPLINE_BUFFER_TOO_SMALL`] if it does not fit. Only meaningful straight
/// after a call that returned a negative code.
///
/// # Safety
///
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

/// The library version, as a static NUL-terminated string.
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
        // A caller in any language can memset this and get sane behaviour,
        // which is the whole reason 0 means "default" rather than "none".
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
        // Installing into a path we invented because the caller passed nothing
        // is how a binding writes 7 GB somewhere nobody asked for.
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
        // Called from a JS finaliser that may run twice.
        unsafe { tapline_job_free(std::ptr::null_mut()) };
        unsafe { tapline_job_cancel(std::ptr::null_mut()) };
    }

    #[test]
    fn a_blocking_wait_does_not_panic_outside_a_runtime() {
        // This is the regression test for an abort, not a failure. The timeout
        // path was only ever exercised with `timeout_ms == 0`, which does not
        // build a timer, so the panic waited until a real Node consumer called
        // it with 250 and took the process down.
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
        // Nothing queued: this must time out rather than abort.
        let code =
            unsafe { tapline_job_next(job, 20, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_TIMEOUT);

        // And it must still deliver an event that arrives while it waits.
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

        // Too small: reports the length and keeps the event.
        let mut needed = 0_usize;
        let mut tiny = [0_u8; 2];
        let code =
            unsafe { tapline_job_next(job, 0, tiny.as_mut_ptr(), tiny.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_BUFFER_TOO_SMALL);
        assert_eq!(needed, r#"{"kind":"progress"}"#.len());

        // Big enough: the same event, not the next one.
        let mut buf = vec![0_u8; needed];
        let code =
            unsafe { tapline_job_next(job, 0, buf.as_mut_ptr(), buf.len(), &raw mut needed) };
        assert_eq!(code, TAPLINE_OK);
        assert_eq!(
            std::str::from_utf8(&buf).ok(),
            Some(r#"{"kind":"progress"}"#)
        );

        // Nothing left, and the senders are still alive, so this is a timeout
        // rather than the end of the stream.
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
