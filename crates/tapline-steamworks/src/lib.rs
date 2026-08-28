//! Account-scoped Workshop actions, through a running Steam client.
//!
//! tapline speaks the CM protocol itself for everything else, and subscribing
//! is finished there too once an account has signed in. This is the other way
//! to reach it: the session already open in the desktop Steam client. It
//! initialises the Steamworks API as an app the account owns and asks the
//! logged-in user's `ISteamUGC` to subscribe. The client does the work.
//!
//! Modelled on the same approach kirie's Steam helper uses, folded into tapline
//! so tapline depends on nothing else for it — the dependency runs the other
//! way, kirie onto tapline.
//!
//! # Why this is separate, off by default, and unpublished
//!
//! It links Valve's closed SDK, which the rest of tapline is built to avoid —
//! no third-party Steam library at any layer, a pure-Rust floor, a 5 MB scratch
//! container. None of that survives contact with `steamclient.so`, so this is
//! walled off: its own crate, `publish = false`, reached only through tapline's
//! `steamworks` feature. The SDK is `dlopen`ed at runtime, so a machine without
//! it still compiles tapline; only using this needs the libraries.
//!
//! # Playtime
//!
//! Initialising as an app makes Steam count the process as that app running.
//! Keep a [`Steam`] only as long as the action takes and drop it — a one-shot
//! `tapline workshop subscribe` does exactly that. A long-running service
//! should spawn a short-lived child to do the same rather than hold one open.

use std::ffi::{CStr, c_char, c_int, c_void};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use tapline_ids::{AppId, PublishedFileId};

/// Steam's success result code.
const RESULT_OK: i32 = 1;

/// `RemoteStorageSubscribePublishedFileResult_t` — `k_iSteamRemoteStorageCallbacks + 13`.
const SUBSCRIBE_RESULT: i32 = 1313;

/// The versioned interface accessors. These are the symbols most likely to move
/// between SDK releases; a rename surfaces as [`SteamError::SymbolMissing`]
/// rather than a crash.
const UGC_ACCESSOR: &[u8] = b"SteamAPI_SteamUGC_v021\0";
const UTILS_ACCESSOR: &[u8] = b"SteamAPI_SteamUtils_v011\0";
const APPS_ACCESSOR: &[u8] = b"SteamAPI_SteamApps_v009\0";

/// Where a Steam install keeps `libsteam_api.so`, relative to a root.
const LIB_RELATIVE: [&str; 2] = ["steamrt64/libsteam_api.so", "linux64/libsteam_api.so"];

type Handle = *mut c_void;

/// The SDK entry points this binds, resolved by name from `libsteam_api.so`.
struct Api {
    _lib: libloading::Library,
    shutdown: unsafe extern "C" fn(),
    subscribe: unsafe extern "C" fn(Handle, u64) -> u64,
    unsubscribe: unsafe extern "C" fn(Handle, u64) -> u64,
    item_state: unsafe extern "C" fn(Handle, u64) -> u32,
    owns_app: unsafe extern "C" fn(Handle, u32) -> bool,
    is_call_completed: unsafe extern "C" fn(Handle, u64, *mut bool) -> bool,
    get_call_result:
        unsafe extern "C" fn(Handle, u64, *mut c_void, c_int, c_int, *mut bool) -> bool,
}

/// A connection to the running client, initialised as one owned app.
///
/// Dropping it calls `SteamAPI_Shutdown`, so the app context does not outlive
/// what asked for it.
pub struct Steam {
    api: Api,
    ugc: Handle,
    utils: Handle,
    apps: Handle,
    app: AppId,
}

/// Why a Steamworks action could not be done.
#[derive(Debug)]
pub enum SteamError {
    /// `libsteam_api.so` was not found under any known Steam root.
    LibraryNotFound(String),
    /// The library loaded but a symbol was missing — usually a versioned
    /// accessor after a client update.
    SymbolMissing(String),
    /// Steam is not running.
    NotRunning,
    /// `SteamAPI_Init` refused, with the reason it gave. Usually: the account
    /// does not own the app.
    InitFailed(String),
    /// An interface pointer came back null.
    NoInterface(&'static str),
    /// The account does not own the app this was initialised as.
    NotOwned(AppId),
    /// Steam accepted the call but reported it failed.
    CallFailed(String),
    /// Steam did not answer within the time allowed.
    TimedOut,
}

impl std::fmt::Display for SteamError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LibraryNotFound(tried) => {
                write!(f, "could not find libsteam_api.so; looked in: {tried}")
            }
            Self::SymbolMissing(name) => write!(
                f,
                "the installed Steam SDK is missing {name}, a version this build \
                 does not know how to talk to"
            ),
            Self::NotRunning => write!(f, "Steam is not running; this needs a logged-in client"),
            Self::InitFailed(why) => write!(f, "Steam refused to initialise: {why}"),
            Self::NoInterface(name) => write!(f, "Steam returned no {name} interface"),
            Self::NotOwned(app) => write!(
                f,
                "this account does not own app {app}; a subscription is made as a \
                 game the account owns"
            ),
            Self::CallFailed(what) => write!(f, "Steam refused: {what}"),
            Self::TimedOut => write!(f, "Steam did not answer in time"),
        }
    }
}

impl std::error::Error for SteamError {}

/// The Steam roots to search for the library.
fn steam_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Some(over) = std::env::var_os("TAPLINE_STEAM_ROOT") {
        roots.push(PathBuf::from(over));
    }
    if let Some(home) = std::env::var_os("HOME") {
        let home = Path::new(&home);
        roots.push(home.join(".local/share/Steam"));
        roots.push(home.join(".steam/steam"));
        roots.push(home.join(".steam/root"));
    }
    roots
}

/// Finds `libsteam_api.so` under a Steam root.
fn find_library(roots: &[PathBuf]) -> Option<PathBuf> {
    roots
        .iter()
        .flat_map(|root| LIB_RELATIVE.iter().map(move |rel| root.join(rel)))
        .find(|path| path.is_file())
}

impl Steam {
    /// Connects to the running client, initialised as `app`.
    ///
    /// `app` must be owned by the signed-in account; init refuses otherwise,
    /// and [`Steam::owns_app`] confirms it after.
    ///
    /// # Errors
    /// If the library is missing, Steam is not running, init fails, or an
    /// interface is null.
    pub fn connect(app: AppId) -> Result<Self, SteamError> {
        let roots = steam_roots();
        let path = find_library(&roots).ok_or_else(|| {
            SteamError::LibraryNotFound(
                roots
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", "),
            )
        })?;

        // Steam reads the app identity from the environment at init. Set before
        // the call, and only for this process.
        //
        // SAFETY: single-threaded here — nothing has been spawned, and no Steam
        // thread exists yet to observe the environment.
        unsafe {
            std::env::set_var("SteamAppId", app.get().to_string());
            std::env::set_var("SteamGameId", app.get().to_string());
        }

        // SAFETY: `path` is a real file; a load or resolution failure is
        // reported, never assumed. Every signature below is the documented
        // flat-API one, verified against the installed library's symbol table.
        unsafe {
            let lib = libloading::Library::new(&path)
                .map_err(|e| SteamError::InitFailed(format!("could not load {path:?}: {e}")))?;

            let sym = |name: &[u8]| -> Result<Handle, SteamError> {
                lib.get::<Handle>(name).map(|s| *s).map_err(|_| {
                    SteamError::SymbolMissing(
                        String::from_utf8_lossy(name.strip_suffix(b"\0").unwrap_or(name))
                            .into_owned(),
                    )
                })
            };

            let is_running = std::mem::transmute::<Handle, unsafe extern "C" fn() -> bool>(sym(
                b"SteamAPI_IsSteamRunning\0",
            )?);
            if !is_running() {
                return Err(SteamError::NotRunning);
            }

            let init_flat = std::mem::transmute::<Handle, unsafe extern "C" fn(*mut c_char) -> c_int>(
                sym(b"SteamAPI_InitFlat\0")?,
            );
            let mut err = [0_i8; 1024];
            let rc = init_flat(err.as_mut_ptr());
            if rc != 0 {
                let why = CStr::from_ptr(err.as_ptr())
                    .to_string_lossy()
                    .trim()
                    .to_owned();
                return Err(SteamError::InitFailed(if why.is_empty() {
                    format!("code {rc}")
                } else {
                    why
                }));
            }

            // Resolve every symbol first, into plain function pointers, so the
            // borrow of `lib` by `sym` ends before `lib` is moved into `Api`.
            let shutdown = sym(b"SteamAPI_Shutdown\0")?;
            let subscribe = sym(b"SteamAPI_ISteamUGC_SubscribeItem\0")?;
            let unsubscribe = sym(b"SteamAPI_ISteamUGC_UnsubscribeItem\0")?;
            let item_state = sym(b"SteamAPI_ISteamUGC_GetItemState\0")?;
            let owns_app = sym(b"SteamAPI_ISteamApps_BIsSubscribedApp\0")?;
            let is_call_completed = sym(b"SteamAPI_ISteamUtils_IsAPICallCompleted\0")?;
            let get_call_result = sym(b"SteamAPI_ISteamUtils_GetAPICallResult\0")?;
            let ugc_ptr = sym(UGC_ACCESSOR)?;
            let utils_ptr = sym(UTILS_ACCESSOR)?;
            let apps_ptr = sym(APPS_ACCESSOR)?;

            let api = Api {
                shutdown: std::mem::transmute::<Handle, unsafe extern "C" fn()>(shutdown),
                subscribe: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, u64) -> u64>(
                    subscribe,
                ),
                unsubscribe: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, u64) -> u64>(
                    unsubscribe,
                ),
                item_state: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, u64) -> u32>(
                    item_state,
                ),
                owns_app: std::mem::transmute::<Handle, unsafe extern "C" fn(Handle, u32) -> bool>(
                    owns_app,
                ),
                is_call_completed: std::mem::transmute::<
                    Handle,
                    unsafe extern "C" fn(Handle, u64, *mut bool) -> bool,
                >(is_call_completed),
                get_call_result: std::mem::transmute::<
                    Handle,
                    unsafe extern "C" fn(Handle, u64, *mut c_void, c_int, c_int, *mut bool) -> bool,
                >(get_call_result),
                _lib: lib,
            };

            let ugc_fn = std::mem::transmute::<Handle, unsafe extern "C" fn() -> Handle>(ugc_ptr);
            let utils_fn =
                std::mem::transmute::<Handle, unsafe extern "C" fn() -> Handle>(utils_ptr);
            let apps_fn = std::mem::transmute::<Handle, unsafe extern "C" fn() -> Handle>(apps_ptr);

            let ugc = ugc_fn();
            let utils = utils_fn();
            let apps = apps_fn();
            if ugc.is_null() {
                (api.shutdown)();
                return Err(SteamError::NoInterface("ISteamUGC"));
            }
            if utils.is_null() {
                (api.shutdown)();
                return Err(SteamError::NoInterface("ISteamUtils"));
            }
            if apps.is_null() {
                (api.shutdown)();
                return Err(SteamError::NoInterface("ISteamApps"));
            }

            Ok(Self {
                api,
                ugc,
                utils,
                apps,
                app,
            })
        }
    }

    /// Whether the signed-in account owns the app this connected as.
    #[must_use]
    pub fn owns_app(&self) -> bool {
        // SAFETY: the interface pointer Steam returned plus an integer.
        unsafe { (self.api.owns_app)(self.apps, self.app.get()) }
    }

    /// Subscribes the signed-in account to an item, waiting for Steam's answer.
    ///
    /// Unlike a fire-and-forget, this reads the actual result — Steam's
    /// `SubscribeItem` returns an async handle, and this polls it and checks the
    /// result code, so a refusal is reported rather than mistaken for success.
    ///
    /// # Errors
    /// If the account does not own the app, or Steam refuses or times out.
    pub fn subscribe(&self, item: PublishedFileId, timeout: Duration) -> Result<(), SteamError> {
        if !self.owns_app() {
            return Err(SteamError::NotOwned(self.app));
        }
        // SAFETY: interface pointer and integer.
        let call = unsafe { (self.api.subscribe)(self.ugc, item.get()) };
        if call == 0 {
            return Err(SteamError::CallFailed(
                "the subscription was not accepted".to_owned(),
            ));
        }
        self.await_call(call, timeout)?;

        // The result carries the EResult. Read it and require success.
        #[repr(C)]
        struct SubscribeResult {
            result: i32,
            published_file_id: u64,
        }
        let mut out = std::mem::MaybeUninit::<SubscribeResult>::zeroed();
        let mut failed = false;
        // SAFETY: `out` is a live local of exactly the size passed, and
        // `SUBSCRIBE_RESULT` is the callback id for this result type.
        let got = unsafe {
            (self.api.get_call_result)(
                self.utils,
                call,
                out.as_mut_ptr().cast(),
                i32::try_from(std::mem::size_of::<SubscribeResult>()).unwrap_or(i32::MAX),
                SUBSCRIBE_RESULT,
                &raw mut failed,
            )
        };
        if !got || failed {
            return Err(SteamError::CallFailed("no subscription result".to_owned()));
        }
        // SAFETY: `got` was true, so Steam wrote the struct.
        let out = unsafe { out.assume_init() };
        if out.result != RESULT_OK {
            return Err(SteamError::CallFailed(format!(
                "result code {}",
                out.result
            )));
        }
        Ok(())
    }

    /// Removes the signed-in account's subscription to an item.
    ///
    /// # Errors
    /// If Steam refuses or times out.
    pub fn unsubscribe(&self, item: PublishedFileId, timeout: Duration) -> Result<(), SteamError> {
        // SAFETY: interface pointer and integer.
        let call = unsafe { (self.api.unsubscribe)(self.ugc, item.get()) };
        if call == 0 {
            return Err(SteamError::CallFailed(
                "the unsubscribe was not accepted".to_owned(),
            ));
        }
        self.await_call(call, timeout)
    }

    /// Steam's item-state bitflags for an item, raw.
    ///
    /// Bit 0 set means subscribed. Exposed for a caller that wants to confirm
    /// what changed without decoding the whole `EItemStateFlags` set here.
    #[must_use]
    pub fn item_state(&self, item: PublishedFileId) -> u32 {
        // SAFETY: interface pointer and integer.
        unsafe { (self.api.item_state)(self.ugc, item.get()) }
    }

    /// Whether the account is currently subscribed to an item.
    #[must_use]
    pub fn is_subscribed(&self, item: PublishedFileId) -> bool {
        self.item_state(item) & 1 != 0
    }

    /// Waits for an async call to complete, or times out.
    fn await_call(&self, call: u64, timeout: Duration) -> Result<(), SteamError> {
        let started = Instant::now();
        loop {
            let mut failed = false;
            // SAFETY: a valid call handle and a live out-parameter.
            if unsafe { (self.api.is_call_completed)(self.utils, call, &raw mut failed) } {
                if failed {
                    return Err(SteamError::CallFailed("the call failed".to_owned()));
                }
                return Ok(());
            }
            if started.elapsed() > timeout {
                return Err(SteamError::TimedOut);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
}

impl Drop for Steam {
    fn drop(&mut self) {
        // SAFETY: shutting down an API that was successfully initialised, once.
        unsafe { (self.api.shutdown)() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn library_search_covers_the_known_layouts() {
        // A missing install returns None rather than panicking, which is what
        // lets `connect` give a clean "not found" with the paths it tried.
        let none = find_library(&[PathBuf::from("/definitely/not/steam")]);
        assert!(none.is_none());
    }

    #[test]
    fn roots_include_the_three_standard_locations() {
        // The three symlinks a desktop install exposes. Missing one is how a
        // real account went undetected before.
        // SAFETY: test-local env set on a single thread.
        unsafe { std::env::set_var("HOME", "/home/someone") };
        let roots = steam_roots();
        assert!(roots.iter().any(|r| r.ends_with(".local/share/Steam")));
        assert!(roots.iter().any(|r| r.ends_with(".steam/steam")));
        assert!(roots.iter().any(|r| r.ends_with(".steam/root")));
    }
}
