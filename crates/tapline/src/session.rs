//! A Steam session that can plan and perform installs.

use crate::{InstallError, InstallOptions, InstallReport};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tapline_cdn::{Host, HostPool, fetch_chunk_bytes, fetch_manifest};
use tapline_event::{Event, Plan};
use tapline_fs::validate_path;
use tapline_ids::{AppId, DepotId, PublishedFileId};
use tapline_io::Sink;
use tapline_manifest::Manifest;
use tapline_net::{EMsg, Frame, Session as CmSession};
use tapline_pics::Depot;
use tapline_proto::steammessages_auth_steamclient::{
    CAuthentication_BeginAuthSessionViaCredentials_Request,
    CAuthentication_BeginAuthSessionViaQR_Request, CAuthentication_GetPasswordRSAPublicKey_Request,
    CAuthentication_PollAuthSessionStatus_Request,
};
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_2::{
    CMsgClientGetDepotDecryptionKey, CMsgClientGetDepotDecryptionKeyResponse,
};
use tapline_proto::steammessages_contentsystem_steamclient::{
    CContentServerDirectory_GetManifestRequestCode_Request,
    CContentServerDirectory_GetServersForSteamPipe_Request,
};
use tapline_proto::steammessages_publishedfile_steamclient::{
    CPublishedFile_GetDetails_Request, CPublishedFile_Subscribe_Request,
    CPublishedFile_Unsubscribe_Request,
};
use tapline_rt_tokio::{CmTransport, FileSink, cm_list};
use tapline_state::AppState;
use tapline_wire::Message;

/// Steam's result code for success.
const RESULT_OK: i32 = 1;

/// A logged-on Steam session.
pub struct Session {
    cm: CmSession<CmTransport>,
    /// The connection pool and chunk budget, shared with any other session
    /// built on the same [`Shared`].
    shared: Arc<crate::Shared>,
    /// Extensions run against each file as it lands.
    extensions: Arc<Vec<Arc<dyn tapline_ext::Extension>>>,
    pool: HostPool,
    cell_id: u32,
    /// Depot keys, cached for the life of the session.
    ///
    /// Steam grants these per depot and they do not change during an install;
    /// asking again for every chunk would be a round trip per megabyte.
    keys: HashMap<DepotId, [u8; 32]>,
    /// How often Steam wants a heartbeat, and when the last one went.
    ///
    /// Not optional. Steam drops a session that stops heartbeating and does not
    /// say why, and a large download spends minutes doing HTTP without touching
    /// the CM connection — so the second install of a pair fails with a bare
    /// disconnect, which is exactly how this was found.
    heartbeat_interval: std::time::Duration,
    last_heartbeat: std::time::Instant,
    /// The name of the app most recently resolved, for the install record.
    app_name: Option<String>,
    /// The build id of the branch most recently resolved.
    build_id: Option<u64>,
    /// The account this session signed in as, or `None` when anonymous.
    account: Option<String>,
}

/// Turns a CM failure on a login RPC into a login error.
///
/// Steam answers a wrong password by failing the RPC with a result code rather
/// than by returning a reply that says so. Wrapping that as an opaque session
/// error loses the one thing the caller needs — "EResult 5" reaching a person
/// instead of "wrong account name or password" is the whole difference.
fn login_failure(error: tapline_net::NetError) -> crate::LoginError {
    match error {
        tapline_net::NetError::Steam { eresult } => crate::LoginError::Refused {
            eresult,
            message: None,
        },
        other => crate::LoginError::Session(other.to_string()),
    }
}

/// Everything needed to download one depot.
struct ResolvedDepot {
    depot: Depot,
    manifest: Manifest,
    key: [u8; 32],
}

impl Session {
    /// Connects to Steam and logs on anonymously.
    ///
    /// The common case, and the one a dedicated-server install needs: no
    /// credentials are read, none are stored, and Steam grants keys for
    /// anonymously accessible content — which is every dedicated server.
    pub async fn anonymous() -> Result<Self, InstallError> {
        Self::anonymous_shared(crate::Shared::new(InstallOptions::default().concurrency)).await
    }

    /// Connects and logs on anonymously, sharing resources with other sessions.
    ///
    /// Use this when a process runs more than one download at a time. Sessions
    /// built on the same [`Shared`] draw from one chunk budget instead of each
    /// taking a full one, and reuse each other's warm connections. Three
    /// downloads that each take 64 chunks in flight are measurably slower than
    /// three that split 64 — see the [`Shared`] docs for the curve.
    ///
    /// [`Shared`]: crate::Shared
    pub async fn anonymous_shared(shared: Arc<crate::Shared>) -> Result<Self, InstallError> {
        let servers = cm_list(0)
            .await
            .map_err(|e| InstallError::Io(format!("could not reach the CM directory: {e}")))?;
        let endpoint = servers
            .first()
            .ok_or_else(|| InstallError::Io("Steam offered no CM servers".to_owned()))?
            .endpoint
            .clone();

        let transport = CmTransport::connect(&endpoint)
            .await
            .map_err(|e| InstallError::Io(format!("could not connect to {endpoint}: {e}")))?;

        let mut cm = CmSession::new(transport);
        let outcome = cm.logon_anonymous(0).await?;

        let mut session = Self {
            cm,
            shared,
            extensions: Arc::new(Vec::new()),
            pool: HostPool::new(Vec::new()),
            cell_id: outcome.cell_id,
            keys: HashMap::new(),
            // Steam asks for 9 seconds. Halved, because the heartbeat is sent
            // between chunk fetches rather than by a timer: whatever is in
            // flight when the deadline passes delays it, and the margin is what
            // absorbs that.
            heartbeat_interval: std::time::Duration::from_secs(
                (u64::from(outcome.heartbeat_seconds).clamp(2, 60)) / 2,
            ),
            last_heartbeat: std::time::Instant::now(),
            app_name: None,
            build_id: None,
            account: None,
        };
        session.refresh_hosts().await?;
        Ok(session)
    }

    /// Logs on as an account, using a refresh token from a completed login.
    ///
    /// The token comes from [`Session::poll_login`] and is what
    /// [`TokenStore`] persists, so a program that has logged in once never has
    /// to ask again until Steam expires it.
    ///
    /// [`TokenStore`]: tapline_auth::TokenStore
    pub async fn with_token(token: &tapline_auth::StoredToken) -> Result<Self, InstallError> {
        Self::with_token_shared(
            token,
            crate::Shared::new(InstallOptions::default().concurrency),
        )
        .await
    }

    /// Logs on as an account, sharing resources with other sessions.
    pub async fn with_token_shared(
        token: &tapline_auth::StoredToken,
        shared: Arc<crate::Shared>,
    ) -> Result<Self, InstallError> {
        let servers = cm_list(0)
            .await
            .map_err(|e| InstallError::Io(format!("could not reach the CM directory: {e}")))?;
        let endpoint = servers
            .first()
            .ok_or_else(|| InstallError::Io("Steam offered no CM servers".to_owned()))?
            .endpoint
            .clone();

        let transport = CmTransport::connect(&endpoint)
            .await
            .map_err(|e| InstallError::Io(format!("could not connect to {endpoint}: {e}")))?;

        // The logon header must name the account's own SteamID, and the token
        // carries it. A token without one is not a token Steam will accept, so
        // say that rather than sending a logon that fails opaquely.
        let steam_id = token.steam_id().ok_or_else(|| {
            InstallError::Io(
                "the saved token is not a Steam refresh token; sign in again with \
                 `tapline login`"
                    .to_owned(),
            )
        })?;

        let mut cm = CmSession::new(transport);
        let outcome = cm
            .logon_with_token(0, &token.account, &token.refresh_token, steam_id)
            .await?;

        let mut session = Self {
            cm,
            shared,
            extensions: Arc::new(Vec::new()),
            pool: HostPool::new(Vec::new()),
            cell_id: outcome.cell_id,
            keys: HashMap::new(),
            heartbeat_interval: std::time::Duration::from_secs(
                (u64::from(outcome.heartbeat_seconds).clamp(2, 60)) / 2,
            ),
            last_heartbeat: std::time::Instant::now(),
            app_name: None,
            build_id: None,
            account: Some(token.account.clone()),
        };
        session.refresh_hosts().await?;
        Ok(session)
    }

    /// A session that signs in if it can, and stays anonymous if it cannot.
    ///
    /// What a caller almost always wants, and what the CLI uses. In order:
    ///
    /// 1. a saved token for `account`, when one is named;
    /// 2. a saved token for whichever account this machine's Steam client last
    ///    used, so a desktop with Steam on it needs no arguments;
    /// 3. anonymous, which is enough for every dedicated server.
    ///
    /// A saved token that Steam rejects — expired, revoked, password changed —
    /// falls back to anonymous rather than failing, because an expired token is
    /// not a reason to stop a download that never needed one. [`Session::account`]
    /// says which of the three happened.
    pub async fn automatic(account: Option<&str>) -> Result<Self, InstallError> {
        Self::automatic_shared(
            account,
            crate::Shared::new(InstallOptions::default().concurrency),
        )
        .await
    }

    /// A session that signs in if it can, sharing resources with others.
    ///
    /// What [`SessionPool`] hands out, so a pooled session is signed in on a
    /// machine that has logged in — the alternative is a library that works
    /// from the command line and silently does not from a binding.
    ///
    /// [`SessionPool`]: crate::SessionPool
    pub async fn automatic_shared(
        account: Option<&str>,
        shared: Arc<crate::Shared>,
    ) -> Result<Self, InstallError> {
        let store = tapline_auth::TokenStore::default_file();
        let wanted = match account {
            Some(name) => Some(name.to_owned()),
            None => tapline_auth::most_recent().map(|found| found.account),
        };

        if let Some(name) = wanted
            && let Ok(Some(token)) = store.load(&name)
        {
            match Self::with_token_shared(&token, Arc::clone(&shared)).await {
                Ok(session) => return Ok(session),
                Err(_) => {
                    // Falls through to anonymous. The caller sees which
                    // happened through `Session::account`, which is a better
                    // channel than a log line a library has no business
                    // deciding to print.
                }
            }
        }

        Self::anonymous_shared(shared).await
    }

    /// The account this session signed in as, or `None` when anonymous.
    #[must_use]
    pub fn account(&self) -> Option<&str> {
        self.account.as_deref()
    }

    /// The cell Steam placed this session in, which decides which CDN hosts are
    /// nearby.
    #[must_use]
    pub const fn cell_id(&self) -> u32 {
        self.cell_id
    }

    /// How many CDN hosts to ask Steam for.
    ///
    /// 20, and widening it is a trap worth documenting. Steam offers 83 for this
    /// cell, and the guess was that ~184 MB/s across 20 of them meant a per-host
    /// cap that more hosts would lift. Measured on Garry's Mod:
    ///
    /// | hosts | chunks in flight | throughput |
    /// |---|---|---|
    /// | 20 | 64  | 184 MB/s |
    /// | 20 | 96  | 164 MB/s |
    /// | 40 | 128 |  60 MB/s |
    /// | 60 | 192 |  59 MB/s |
    ///
    /// The last two hold chunks-per-host constant at 3.2 and still collapse, so
    /// it is not contention for a fixed number of caches. It is connection
    /// reuse: chunks round-robin across the whole list, so a wider list means a
    /// given host is revisited less often, its pooled connection is evicted
    /// before it is wanted again, and the request pays a fresh TLS handshake
    /// instead of riding a warm socket. Twenty hosts stay warm; sixty do not.
    ///
    /// Raising this without also making host selection sticky per in-flight
    /// slot makes downloads three times slower. That work is worth doing — it
    /// would let a fat link use the whole fleet — and until it exists, 20.
    const MAX_CDN_HOSTS: u32 = 20;

    /// Fetches the CDN host list for this cell.
    async fn refresh_hosts(&mut self) -> Result<(), InstallError> {
        let directory = self
            .cm
            .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
                cell_id: Some(self.cell_id),
                max_servers: Some(Self::MAX_CDN_HOSTS),
                ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
            })
            .await?;

        let hosts: Vec<Host> = directory
            .servers
            .iter()
            .filter_map(|server| {
                if !tapline_cdn::usable_over_tls(server.https_support.as_deref()) {
                    return None;
                }
                let host = server.host.clone()?;
                Some(Host {
                    vhost: server.vhost.clone().unwrap_or_else(|| host.clone()),
                    host,
                    // Steam types load as int32. A negative is nonsense, and an
                    // absent one must sort last rather than first.
                    load: server
                        .load
                        .and_then(|value| u32::try_from(value).ok())
                        .unwrap_or(u32::MAX),
                    https_required: server.https_support.as_deref() == Some("mandatory"),
                })
            })
            .collect();

        self.pool = HostPool::new(hosts);
        Ok(())
    }

    /// Sends a heartbeat now, whether or not one is due.
    ///
    /// What keeps a pooled session alive while nobody is using it. Steam drops
    /// a session that goes quiet and does not say so; the failure appears much
    /// later as an unrelated request returning "disconnected".
    pub async fn keep_alive(&mut self) -> Result<(), InstallError> {
        self.cm.heartbeat().await?;
        self.last_heartbeat = std::time::Instant::now();
        Ok(())
    }

    /// Sends a heartbeat if one is due.
    ///
    /// Called between chunks. A download is minutes of HTTP with no CM traffic,
    /// and Steam ends a silent session without warning — the failure surfaces
    /// much later, as an unrelated request returning "disconnected".
    async fn maybe_heartbeat(&mut self) -> Result<(), InstallError> {
        if self.last_heartbeat.elapsed() < self.heartbeat_interval {
            return Ok(());
        }
        self.cm.heartbeat().await?;
        self.last_heartbeat = std::time::Instant::now();
        Ok(())
    }

    /// Asks Steam for a depot's decryption key.
    ///
    /// Cached: the key does not change during an install, and asking per chunk
    /// would be a round trip per megabyte.
    async fn depot_key(&mut self, app: AppId, depot: DepotId) -> Result<[u8; 32], InstallError> {
        if let Some(key) = self.keys.get(&depot) {
            return Ok(*key);
        }

        let job = self.cm.next_job_id();
        let header = CMsgProtoBufHeader {
            client_sessionid: Some(self.cm.session_id()),
            steamid: Some(self.cm.steam_id()),
            jobid_source: Some(job),
            ..CMsgProtoBufHeader::default()
        };
        self.cm
            .send(&Frame::new(
                EMsg::GET_DEPOT_DECRYPTION_KEY,
                header,
                CMsgClientGetDepotDecryptionKey {
                    depot_id: Some(depot.get()),
                    app_id: Some(app.get()),
                }
                .encode_to_vec(),
            ))
            .await?;

        let reply = self.cm.wait_for_job(job).await?;
        let response: CMsgClientGetDepotDecryptionKeyResponse = reply.decode_body()?;

        let eresult = response.eresult.unwrap_or(0);
        if eresult != RESULT_OK {
            return Err(InstallError::NoDepotKey { depot, eresult });
        }

        let key: [u8; 32] = response
            .depot_encryption_key
            .as_deref()
            .and_then(|bytes| bytes.try_into().ok())
            .ok_or(InstallError::NoDepotKey { depot, eresult })?;

        self.keys.insert(depot, key);
        Ok(key)
    }

    /// Resolves every depot an install needs: keys and manifests.
    async fn resolve(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<Vec<ResolvedDepot>, InstallError> {
        let info = tapline_pics::product_info(&mut self.cm, app).await?;
        self.app_name = info.name().map(str::to_owned);
        self.build_id = info.build_id(&options.branch);
        let depots = info.depots(&options.filter());

        if depots.is_empty() {
            return Err(InstallError::NothingToInstall {
                app,
                branch: options.branch.clone(),
            });
        }

        let mut resolved = Vec::with_capacity(depots.len());
        for depot in depots {
            // A borrowed depot's key belongs to the app that owns it, not the
            // app being installed.
            let key = self.depot_key(depot.owner, depot.id).await?;

            let code = self
                .cm
                .call(&CContentServerDirectory_GetManifestRequestCode_Request {
                    app_id: Some(depot.owner.get()),
                    depot_id: Some(depot.id.get()),
                    manifest_id: Some(depot.manifest.get()),
                    app_branch: Some(options.branch.clone()),
                    branch_password_hash: None,
                })
                .await?
                .manifest_request_code
                .unwrap_or(0);

            let host = self.pool.acquire()?;
            let manifest = fetch_manifest(
                self.shared.http.as_ref(),
                &host.host,
                depot.id,
                depot.manifest.get(),
                code,
                Some(&key),
            )
            .await?;

            resolved.push(ResolvedDepot {
                depot,
                manifest,
                key,
            });
        }
        Ok(resolved)
    }

    /// Works out what an install would cost, without fetching any content.
    ///
    /// Reads what is already on disk to decide what can be reused, which is why
    /// the answer is useful for an update rather than only for a fresh install.
    pub async fn plan(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<Plan, InstallError> {
        let resolved = self.resolve(app, options).await?;

        let mut plan = Plan::default();
        for entry in &resolved {
            let (chunks, download_bytes) = entry.manifest.distinct_chunks();
            plan.chunk_count += chunks.len() as u64;
            plan.total_bytes += entry.manifest.total_size;
            plan.file_count += entry.manifest.regular_files().count() as u64;

            // Anything already correct on disk is reuse, not download.
            let mut reusable = 0_u64;
            for file in entry.manifest.regular_files() {
                if let Ok(safe) = validate_path(&file.path) {
                    let target = safe.resolve(&options.install_dir);
                    if let Ok(metadata) = std::fs::metadata(&target)
                        && metadata.len() == file.size
                    {
                        reusable += file.size;
                    }
                }
            }
            plan.reused_bytes += reusable;
            plan.download_bytes += download_bytes;
        }
        Ok(plan)
    }

    /// Resolves a SteamPipe Workshop item to its depot, manifest and key.
    async fn resolve_workshop(
        &mut self,
        app: AppId,
        depot: DepotId,
        manifest: tapline_ids::ManifestId,
    ) -> Result<ResolvedDepot, InstallError> {
        let key = self.depot_key(app, depot).await?;

        let code = self
            .cm
            .call(&CContentServerDirectory_GetManifestRequestCode_Request {
                app_id: Some(app.get()),
                depot_id: Some(depot.get()),
                manifest_id: Some(manifest.get()),
                app_branch: Some("public".to_owned()),
                branch_password_hash: None,
            })
            .await?
            .manifest_request_code
            .unwrap_or(0);

        let host = self.pool.acquire()?;
        let content = fetch_manifest(
            self.shared.http.as_ref(),
            &host.host,
            depot,
            manifest.get(),
            code,
            Some(&key),
        )
        .await?;

        Ok(ResolvedDepot {
            depot: tapline_pics::Depot {
                id: depot,
                manifest,
                size: content.total_size,
                download_size: 0,
                owner: app,
            },
            manifest: content,
            key,
        })
    }

    /// Opens a Workshop item for reading without downloading it.
    ///
    /// The returned [`RemoteFile`] fetches only the chunks a read crosses, so a
    /// caller can read a container's index — wherever it lives — and then only
    /// the entries it wants.
    ///
    /// That is what makes formats other than GMAD possible. A ZIP keeps its
    /// central directory at the end, which no sequential reader can reach
    /// early; here it is a read like any other. It is also what lets a filter
    /// stop paying for what it discards.
    ///
    /// [`RemoteFile`]: crate::RemoteFile
    pub async fn open_workshop_item(
        &mut self,
        item: &crate::WorkshopItem,
    ) -> Result<crate::RemoteFile, InstallError> {
        let crate::WorkshopContent::SteamPipe { depot, manifest } = &item.content else {
            return Err(InstallError::Io(
                "only SteamPipe Workshop items can be read by range; this one is a legacy UFS blob"
                    .to_owned(),
            ));
        };

        let entry = self.resolve_workshop(item.app, *depot, *manifest).await?;
        let files: Vec<_> = entry.manifest.regular_files().collect();
        let [file] = files.as_slice() else {
            return Err(InstallError::Io(format!(
                "reading by range needs a single-file item; this one has {}",
                files.len()
            )));
        };

        let hosts = self.pool.snapshot();
        if hosts.is_empty() {
            return Err(InstallError::Pool(tapline_cdn::PoolError::Empty));
        }

        Ok(crate::RemoteFile::new(
            file.chunks.clone(),
            *depot,
            entry.key,
            hosts,
            Arc::clone(&self.shared.http),
            Arc::clone(&self.shared.limit),
        ))
    }

    /// Downloads a Workshop item and feeds its bytes to `consumer`, in order,
    /// without writing the item itself to disk.
    ///
    /// For formats that can be read as they arrive. A Garry's Mod addon is one:
    /// GMAD's header and index come first and its file contents follow in index
    /// order, so an extractor can write each file as its bytes land and the
    /// `.gma` never needs to exist. Measured on a real addon that is 8.4 MB
    /// neither written nor read back.
    ///
    /// Chunks are still fetched in parallel — that is where the throughput is —
    /// and reordered through a bounded window, so peak memory is the window
    /// rather than the file. See [`Window`].
    ///
    /// Only single-file items are accepted. A multi-file item has no meaningful
    /// byte order to stream, and guessing one would hand the consumer a
    /// concatenation nothing can parse.
    ///
    /// [`Window`]: crate::Window
    pub async fn stream_workshop_item(
        &mut self,
        item: &crate::WorkshopItem,
        window: crate::Window,
        consumer: crate::Consumer<'_>,
        observe: &mut (dyn FnMut(Event) + Send),
    ) -> Result<crate::StreamReport, InstallError> {
        let crate::WorkshopContent::SteamPipe { depot, manifest } = &item.content else {
            return Err(InstallError::Io(
                "only SteamPipe Workshop items can be streamed; this one is a legacy UFS blob"
                    .to_owned(),
            ));
        };

        let entry = self.resolve_workshop(item.app, *depot, *manifest).await?;
        let files: Vec<_> = entry.manifest.regular_files().collect();
        let [file] = files.as_slice() else {
            return Err(InstallError::Io(format!(
                "streaming needs a single-file item; this one has {}",
                files.len()
            )));
        };

        observe(Event::Planned {
            plan: Plan {
                download_bytes: entry.manifest.distinct_chunks().1,
                reused_bytes: 0,
                total_bytes: entry.manifest.total_size,
                file_count: 1,
                chunk_count: file.chunks.len() as u64,
            },
        });

        // Offset order is the order the consumer must see. The manifest does
        // not promise the index is sorted, so this does not assume it is.
        let mut chunks = file.chunks.clone();
        chunks.sort_by_key(|chunk| chunk.offset);

        let hosts: Vec<String> = self.pool.snapshot();
        if hosts.is_empty() {
            return Err(InstallError::Pool(tapline_cdn::PoolError::Empty));
        }

        let mut report = crate::StreamReport::default();
        let mut reorderer = crate::Reorderer::new();
        let mut tasks: StreamTasks = tokio::task::JoinSet::new();
        let mut next_to_fetch = 0_usize;

        // Only ever `window.size` in flight, and only ever started in order, so
        // the reorder buffer can never hold more than the window.
        while next_to_fetch < chunks.len() && tasks.len() < window.size {
            self.spawn_stream_chunk(
                &chunks,
                next_to_fetch,
                &hosts,
                entry.key,
                *depot,
                &mut tasks,
            );
            next_to_fetch += 1;
        }

        while let Some(joined) = tasks.join_next().await {
            let (index, outcome) = joined
                .map_err(|error| InstallError::Io(format!("a stream task failed: {error}")))?;
            let (bytes, downloaded, host) = outcome?;
            self.pool.succeed(&host);

            report.chunks += 1;
            report.bytes_downloaded += downloaded;

            for ready in reorderer.accept(index, bytes) {
                report.bytes_streamed += ready.len() as u64;
                consumer(&ready)?;
                observe(Event::Progress {
                    bytes_done: report.bytes_streamed,
                    bytes_total: entry.manifest.total_size,
                });
            }
            report.peak_buffered = report.peak_buffered.max(reorderer.buffered());

            if next_to_fetch < chunks.len() {
                self.spawn_stream_chunk(
                    &chunks,
                    next_to_fetch,
                    &hosts,
                    entry.key,
                    *depot,
                    &mut tasks,
                );
                next_to_fetch += 1;
            }
            self.maybe_heartbeat().await?;
        }

        if !reorderer.is_empty() || reorderer.delivered() != chunks.len() {
            return Err(InstallError::Io(format!(
                "the stream ended with {} of {} chunks delivered",
                reorderer.delivered(),
                chunks.len()
            )));
        }
        Ok(report)
    }

    /// Spawns one chunk fetch for a streamed download.
    fn spawn_stream_chunk(
        &self,
        chunks: &[tapline_manifest::Chunk],
        index: usize,
        hosts: &[String],
        key: [u8; 32],
        depot: DepotId,
        tasks: &mut StreamTasks,
    ) {
        let Some(chunk) = chunks.get(index).cloned() else {
            return;
        };
        let http = Arc::clone(&self.shared.http);
        let hosts = hosts.to_vec();
        let shared_limit = Arc::clone(&self.shared.limit);

        tasks.spawn(async move {
            let outcome = async move {
                // The same process-wide budget an ordinary download draws on,
                // so a streamed item and a normal install do not each take one.
                let _permit = shared_limit
                    .acquire_owned()
                    .await
                    .map_err(|error| InstallError::Io(error.to_string()))?;

                let mut last_error = None;
                for attempt in 0..4_usize {
                    let host = hosts
                        .get((index + attempt) % hosts.len())
                        .cloned()
                        .unwrap_or_default();
                    match fetch_chunk_bytes(http.as_ref(), &host, depot, &chunk).await {
                        Ok(stored) => {
                            let for_decode = chunk.clone();
                            let host_for_decode = host.clone();
                            let decoded = tokio::task::spawn_blocking(move || {
                                tapline_cdn::decode_chunk_owned(
                                    stored,
                                    &for_decode,
                                    &key,
                                    &host_for_decode,
                                )
                            })
                            .await
                            .map_err(|error| InstallError::Io(error.to_string()))?;
                            match decoded {
                                Ok(plaintext) => {
                                    return Ok((plaintext, u64::from(chunk.compressed_size), host));
                                }
                                Err(error) => last_error = Some(error),
                            }
                        }
                        Err(error) => last_error = Some(error),
                    }
                }
                Err(last_error.map_or(
                    InstallError::Pool(tapline_cdn::PoolError::AllDemoted),
                    Into::into,
                ))
            }
            .await;
            (index, outcome)
        });
    }

    /// Adds an extension, which runs against every file this session writes.
    ///
    /// Extensions are code the operator chose and compiled in. Nothing in a
    /// manifest, a Workshop item or a CDN response can introduce one — the same
    /// line tapline draws by parsing `installscript.vdf` and refusing to run
    /// it.
    ///
    /// ```ignore
    /// // tapline-gmad is a separate crate, so this is not compiled here.
    /// let mut session = Session::anonymous().await?;
    /// session.register(std::sync::Arc::new(tapline_gmad::Extract::new()));
    /// ```
    pub fn register(&mut self, extension: Arc<dyn tapline_ext::Extension>) {
        Arc::make_mut(&mut self.extensions).push(extension);
    }

    /// The extensions registered on this session.
    #[must_use]
    pub fn extensions(&self) -> &[Arc<dyn tapline_ext::Extension>] {
        &self.extensions
    }

    /// Installs or updates an app.
    ///
    /// Reads the existing install record first. A depot already at the manifest
    /// being installed is skipped entirely — running an update when nothing
    /// changed must not move a byte, and that is the case an operator hits most
    /// often.
    pub async fn install(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<InstallReport, InstallError> {
        self.install_observed(app, options, &mut |_| {}).await
    }

    /// Installs, reporting progress as it goes.
    ///
    /// The observer is called on the task driving the download, never from a
    /// fetch task, so it needs no synchronisation of its own and cannot be
    /// invoked concurrently with itself. It should still return quickly: time
    /// spent in it is time no chunk is dispatched.
    ///
    /// [`Event`] existed as a vocabulary long before anything emitted it, which
    /// meant every consumer had to choose between "no progress at all" and
    /// reimplementing the walk. This is what emits it.
    pub async fn install_observed(
        &mut self,
        app: AppId,
        options: &InstallOptions,
        observe: &mut (dyn FnMut(Event) + Send),
    ) -> Result<InstallReport, InstallError> {
        let resolved = self.resolve(app, options).await?;

        // Planned first, always: a consumer drawing a progress bar needs the
        // denominator before the numerator starts moving.
        let mut planned = Plan::default();
        for entry in &resolved {
            let (chunks, download_bytes) = entry.manifest.distinct_chunks();
            planned.chunk_count += chunks.len() as u64;
            planned.total_bytes += entry.manifest.total_size;
            planned.file_count += entry.manifest.regular_files().count() as u64;
            planned.download_bytes += download_bytes;
        }
        observe(Event::Planned { plan: planned });
        std::fs::create_dir_all(&options.install_dir)?;

        let mut state = AppState::read(&options.install_dir, app)
            .map_err(|e| InstallError::Io(e.to_string()))?
            .unwrap_or_else(|| {
                AppState::new(
                    app,
                    &self.app_name.clone().unwrap_or_else(|| app.to_string()),
                    ".",
                )
            });

        let mut report = InstallReport {
            app,
            ..InstallReport::default()
        };

        for entry in &resolved {
            report.depots.push(entry.depot.id);

            if state.installed_manifest(entry.depot.id) == Some(entry.depot.manifest)
                && !options.force
            {
                // Already at this build. Verifying every byte would be the
                // `validate` command's job, not an update's.
                report.depots_unchanged += 1;
                report.chunks_reused += u64::from(entry.manifest.unique_chunks);
                continue;
            }

            observe(Event::DepotStarted {
                depot: entry.depot.id,
                manifest: entry.depot.manifest,
                bytes: entry.manifest.total_size,
            });
            self.install_depot(
                entry,
                options,
                &mut report,
                observe,
                planned.total_bytes,
                app,
            )
            .await?;
            observe(Event::DepotCompleted {
                depot: entry.depot.id,
            });
            state.set_depot(
                entry.depot.id,
                entry.depot.manifest,
                entry.manifest.total_size,
            );
        }

        // A depot the app no longer ships must leave the record, or the next
        // update believes content is present that is not.
        let current: std::collections::HashSet<DepotId> =
            resolved.iter().map(|entry| entry.depot.id).collect();
        for depot in state.installed_depots().keys().copied().collect::<Vec<_>>() {
            if !current.contains(&depot) {
                state.remove_depot(depot);
            }
        }

        let total: u64 = resolved.iter().map(|e| e.manifest.total_size).sum();
        state.mark_installed(self.build_id.unwrap_or(0), total, now_unix());
        state
            .write(&options.install_dir, app)
            .map_err(|e| InstallError::Io(e.to_string()))?;

        Ok(report)
    }

    /// Starts a QR login.
    ///
    /// No password is involved at any point: Steam issues a challenge URL, the
    /// user approves it in the mobile app, and [`Session::poll_login`] returns a
    /// token. For anything interactive this is both easier and safer than
    /// typing a password into a terminal.
    pub async fn begin_qr_login(&mut self) -> Result<crate::PendingLogin, crate::LoginError> {
        let response = self
            .cm
            .call(&CAuthentication_BeginAuthSessionViaQR_Request {
                device_friendly_name: Some(tapline_auth::DEVICE_NAME.to_owned()),
                platform_type: Some(
                    tapline_proto::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                        tapline_auth::PLATFORM_STEAM_CLIENT,
                    ),
                ),
                device_details: Some(crate::login::device_details()),
                website_id: Some("Client".to_owned()),
            })
            .await
            .map_err(login_failure)?;

        Ok(crate::PendingLogin {
            client_id: response.client_id.unwrap_or(0),
            request_id: response.request_id.clone().unwrap_or_default(),
            interval: response.interval.unwrap_or(5.0),
            confirmations: crate::login::confirmations_from(&response.allowed_confirmations),
            challenge_url: response.challenge_url.clone(),
            account: None,
            // A QR login does not know the account until it is approved.
            steam_id: 0,
        })
    }

    /// Fetches the per-account RSA key Steam issues for a password login.
    ///
    /// A separate step because it needs no secret — only the account name — and
    /// separating it means the password exists for as short a time as possible:
    /// the caller can prompt for it *after* this returns.
    pub async fn password_key(
        &mut self,
        account: &str,
    ) -> Result<tapline_auth::PublicKey, crate::LoginError> {
        let response = self
            .cm
            .call(&CAuthentication_GetPasswordRSAPublicKey_Request {
                account_name: Some(account.to_owned()),
            })
            .await
            .map_err(login_failure)?;

        crate::login::key_from_response(&response)
    }

    /// Starts a password login.
    ///
    /// Takes the password by value and hands it straight to the encrypter, which
    /// zeroes it. Nothing here logs it, stores it, or keeps a second copy.
    pub async fn begin_password_login(
        &mut self,
        account: &str,
        password: String,
        key: &tapline_auth::PublicKey,
    ) -> Result<crate::PendingLogin, crate::LoginError> {
        let encrypted = tapline_auth::encrypt_password(password, key)
            .map_err(|error| crate::LoginError::Password(error.to_string()))?;

        let response = self
            .cm
            .call(&CAuthentication_BeginAuthSessionViaCredentials_Request {
                device_friendly_name: Some(tapline_auth::DEVICE_NAME.to_owned()),
                account_name: Some(account.to_owned()),
                encrypted_password: Some(encrypted),
                encryption_timestamp: Some(key.timestamp),
                remember_login: Some(true),
                platform_type: Some(
                    tapline_proto::steammessages_auth_steamclient::EAuthTokenPlatformType::from(
                        tapline_auth::PLATFORM_STEAM_CLIENT,
                    ),
                ),
                device_details: Some(crate::login::device_details()),
                website_id: Some("Client".to_owned()),
                ..CAuthentication_BeginAuthSessionViaCredentials_Request::default()
            })
            .await
            .map_err(login_failure)?;

        // Steam reports a failed password as an eresult on the reply rather than
        // as an error, so a caller that only checked for transport failures
        // would see an empty session and no reason.
        if response.client_id.is_none() {
            return Err(crate::LoginError::Refused {
                eresult: 0,
                message: response.extended_error_message.clone(),
            });
        }

        Ok(crate::PendingLogin {
            client_id: response.client_id.unwrap_or(0),
            request_id: response.request_id.clone().unwrap_or_default(),
            interval: response.interval.unwrap_or(5.0),
            confirmations: crate::login::confirmations_from(&response.allowed_confirmations),
            challenge_url: None,
            account: Some(account.to_owned()),
            steam_id: response.steamid.unwrap_or(0),
        })
    }

    /// Signs in with a username and a password, in one call.
    ///
    /// The whole flow — RSA key, encrypted credentials, an optional Steam Guard
    /// code, and polling until Steam finishes — behind the two things a caller
    /// actually has. Returns the token to persist with [`TokenStore`], after
    /// which [`Session::automatic`] signs in by itself and this is never needed
    /// again.
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use tapline::Session;
    /// use tapline_auth::TokenStore;
    ///
    /// let mut session = Session::anonymous().await?;
    /// let token = session.sign_in("username", "password", None).await?;
    /// TokenStore::default_file().save(&token)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// `guard_code` is the emailed or authenticator code. Pass `None` when
    /// there is none; if Steam asks for one and none was given, the error says
    /// so rather than polling until it times out.
    ///
    /// Steam refuses a login whose password is right and whose code is missing,
    /// so an account with Steam Guard on it cannot be signed into with two
    /// strings alone — that is Steam's rule, not this one. Either supply the
    /// code or use the QR flow, which needs neither.
    ///
    /// [`TokenStore`]: tapline_auth::TokenStore
    pub async fn sign_in(
        &mut self,
        username: &str,
        password: &str,
        guard_code: Option<&str>,
    ) -> Result<tapline_auth::StoredToken, crate::LoginError> {
        let key = self.password_key(username).await?;
        let pending = self
            .begin_password_login(username, password.to_owned(), &key)
            .await?;

        let wanted = pending
            .confirmations
            .iter()
            .copied()
            .find(|kind| kind.needs_a_code());

        match (wanted, guard_code) {
            (Some(kind), Some(code)) => self.submit_guard_code(&pending, code, kind).await?,
            (Some(kind), None) => {
                // Said now rather than after polling for two minutes at
                // something that is never going to complete.
                return Err(crate::LoginError::Password(format!(
                    "this account needs {kind}; pass it as guard_code"
                )));
            }
            (None, _) => {}
        }

        let interval = std::time::Duration::from_secs_f32(pending.interval.max(1.0));
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(180);
        let mut current = pending;
        while std::time::Instant::now() < deadline {
            match self.poll_login(&current).await? {
                crate::PollOutcome::Complete {
                    account,
                    refresh_token,
                    ..
                } => {
                    return Ok(tapline_auth::StoredToken {
                        account,
                        refresh_token,
                    });
                }
                crate::PollOutcome::Moved { client_id, .. } => current.client_id = client_id,
                crate::PollOutcome::Pending { .. } => {}
            }
            tokio::time::sleep(interval).await;
        }
        Err(crate::LoginError::Session(
            "timed out waiting for the login to complete".to_owned(),
        ))
    }

    /// Submits a Steam Guard code for a pending login.
    ///
    /// Needed when [`PendingLogin::confirmations`] asks for a code — emailed or
    /// from the authenticator. The kind must be the one Steam asked for:
    /// a device code sent where an email code was wanted is refused, which is
    /// why the caller echoes the [`GuardType`] it saw rather than this guessing.
    ///
    /// Poll as usual afterwards. Steam completes the session on the next poll
    /// rather than in the reply here.
    ///
    /// [`PendingLogin::confirmations`]: crate::PendingLogin::confirmations
    /// [`GuardType`]: tapline_auth::GuardType
    pub async fn submit_guard_code(
        &mut self,
        pending: &crate::PendingLogin,
        code: &str,
        kind: tapline_auth::GuardType,
    ) -> Result<(), crate::LoginError> {
        use tapline_proto::steammessages_auth_steamclient::{
            CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request, EAuthSessionGuardType,
        };

        self.cm
            .call(
                &CAuthentication_UpdateAuthSessionWithSteamGuardCode_Request {
                    client_id: Some(pending.client_id),
                    steamid: Some(pending.steam_id),
                    code: Some(code.to_owned()),
                    code_type: Some(EAuthSessionGuardType::from(kind.as_i32())),
                },
            )
            .await
            .map_err(login_failure)?;
        Ok(())
    }

    /// Runs a QR login to completion, showing the code and refreshing it.
    ///
    /// This is the whole flow, so a caller does not have to drive the poll loop
    /// by hand and remember the one thing that is easy to get wrong: a QR code
    /// **expires**, and Steam hands back a new one mid-login. `on_code` is
    /// called with the URL to render — once at the start, and again every time
    /// Steam rotates it — so a display that redraws on each call always shows a
    /// scannable code.
    ///
    /// Render the URL as a QR image however suits the caller; tapline gives the
    /// string rather than pixels, so it pulls in no rendering dependency. The
    /// URL is `https://s.team/q/...`, which the Steam mobile app scans.
    ///
    /// Returns the token to persist with [`TokenStore`]. After that,
    /// [`Session::automatic`] signs in on its own.
    ///
    /// ```no_run
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// use tapline::Session;
    /// use tapline_auth::TokenStore;
    ///
    /// let mut session = Session::anonymous().await?;
    /// let token = session
    ///     .qr_login(std::time::Duration::from_secs(300), &mut |url| {
    ///         println!("scan this: {url}");
    ///     })
    ///     .await?;
    /// TokenStore::default_file().save(&token)?;
    /// # Ok(())
    /// # }
    /// ```
    ///
    /// `timeout` bounds the whole attempt — a code nobody scans should not wait
    /// forever. Steam's own rotation happens inside it.
    ///
    /// [`TokenStore`]: tapline_auth::TokenStore
    pub async fn qr_login(
        &mut self,
        timeout: std::time::Duration,
        on_code: &mut (dyn FnMut(&str) + Send),
    ) -> Result<tapline_auth::StoredToken, crate::LoginError> {
        let mut current = self.begin_qr_login().await?;

        // The first code, before any polling. A caller that only rendered on
        // refresh would show nothing until Steam's first rotation.
        if let Some(url) = &current.challenge_url {
            on_code(url);
        }

        let interval = std::time::Duration::from_secs_f32(current.interval.max(1.0));
        let deadline = std::time::Instant::now() + timeout;

        while std::time::Instant::now() < deadline {
            tokio::time::sleep(interval).await;

            match self.poll_login(&current).await? {
                crate::PollOutcome::Pending { .. } => {}
                crate::PollOutcome::Moved {
                    client_id,
                    challenge_url,
                } => {
                    // The old code has expired; polling it again waits forever.
                    // Take the new one and show it.
                    current.client_id = client_id;
                    current.challenge_url = challenge_url;
                    if let Some(url) = &current.challenge_url {
                        on_code(url);
                    }
                }
                crate::PollOutcome::Complete {
                    account,
                    refresh_token,
                    ..
                } => {
                    return Ok(tapline_auth::StoredToken {
                        account,
                        refresh_token,
                    });
                }
            }
        }

        Err(crate::LoginError::Session(
            "the QR login was not approved in time".to_owned(),
        ))
    }

    /// Polls a login once.
    ///
    /// The caller sleeps for `PendingLogin::interval` between calls. Polling
    /// faster is how a login gets rate limited, and Steam says what it wants.
    pub async fn poll_login(
        &mut self,
        pending: &crate::PendingLogin,
    ) -> Result<crate::PollOutcome, crate::LoginError> {
        let response = self
            .cm
            .call(&CAuthentication_PollAuthSessionStatus_Request {
                client_id: Some(pending.client_id),
                request_id: Some(pending.request_id.clone()),
                token_to_revoke: None,
            })
            .await
            .map_err(login_failure)?;

        if let Some(refresh_token) = response
            .refresh_token
            .as_deref()
            .filter(|token| !token.is_empty())
        {
            return Ok(crate::PollOutcome::Complete {
                account: response
                    .account_name
                    .clone()
                    .or_else(|| pending.account.clone())
                    .unwrap_or_default(),
                refresh_token: refresh_token.to_owned(),
                access_token: response.access_token.clone().unwrap_or_default(),
            });
        }

        // A QR code that refreshed moves the session. Polling the old client id
        // afterwards waits forever.
        if let Some(new_client_id) = response.new_client_id.filter(|id| *id != 0) {
            return Ok(crate::PollOutcome::Moved {
                client_id: new_client_id,
                challenge_url: response.new_challenge_url.clone(),
            });
        }

        Ok(crate::PollOutcome::Pending {
            had_interaction: response.had_remote_interaction.unwrap_or(false),
        })
    }

    /// Makes a unified-message call on the session's CM connection.
    ///
    /// Exposed so a caller can reach a service this facade does not wrap —
    /// `PublishedFile.QueryFiles`, say — without opening a second session.
    pub async fn call_raw<R: tapline_wire::Rpc>(
        &mut self,
        request: &R,
    ) -> Result<R::Response, InstallError> {
        Ok(self.cm.call(request).await?)
    }

    /// Fetches an app's PICS document.
    pub async fn app_info(&mut self, app: AppId) -> Result<tapline_pics::AppInfo, InstallError> {
        Ok(tapline_pics::product_info(&mut self.cm, app).await?)
    }

    /// Describes Workshop items.
    ///
    /// Items Steam refuses, or describes with nothing fetchable, come back as
    /// errors in place rather than being dropped: asking for five items and
    /// getting three back silently is worse than being told which two failed.
    pub async fn workshop_details(
        &mut self,
        ids: &[PublishedFileId],
    ) -> Result<Vec<Result<crate::WorkshopItem, crate::WorkshopError>>, InstallError> {
        let response = self
            .cm
            .call(&CPublishedFile_GetDetails_Request {
                publishedfileids: ids.iter().map(|id| id.get()).collect(),
                includechildren: Some(true),
                short_description: Some(true),
                ..CPublishedFile_GetDetails_Request::default()
            })
            .await?;

        // Each item needs its app's workshop depot, and several items usually
        // share an app — so PICS is asked once per app rather than once per
        // item.
        let mut depots: HashMap<AppId, Option<DepotId>> = HashMap::new();
        let mut out = Vec::with_capacity(ids.len());

        for details in &response.publishedfiledetails {
            let app = AppId(details.consumer_appid.unwrap_or(0));

            let workshop_depot = match depots.get(&app) {
                Some(cached) => *cached,
                None => {
                    let resolved = if app.get() == 0 {
                        None
                    } else {
                        tapline_pics::product_info(&mut self.cm, app)
                            .await
                            .ok()
                            .and_then(|info| info.workshop_depot())
                    };
                    depots.insert(app, resolved);
                    resolved
                }
            };

            out.push(crate::classify(details, workshop_depot));
        }

        // An item Steam did not return at all is still an answer the caller
        // asked for.
        for id in ids {
            let mentioned = response
                .publishedfiledetails
                .iter()
                .any(|d| d.publishedfileid == Some(id.get()));
            if !mentioned {
                out.push(Err(crate::WorkshopError::NotReturned { id: *id }));
            }
        }

        Ok(out)
    }

    /// Searches an app's Workshop.
    ///
    /// The same query the Workshop website runs, over the CM session already
    /// open — no WebAPI key, and no login: an anonymous session searches fine.
    ///
    /// Results carry a [`WorkshopItem`], so anything found here can be passed
    /// straight to [`Session::download_workshop_item`] without a second lookup.
    ///
    /// Paging is a cursor. Pass [`BrowsePage::next_cursor`] back as
    /// [`BrowseQuery::cursor`] to continue, and stop when it is `None`.
    ///
    /// [`WorkshopItem`]: crate::WorkshopItem
    /// [`BrowsePage::next_cursor`]: crate::BrowsePage::next_cursor
    /// [`BrowseQuery::cursor`]: crate::BrowseQuery::cursor
    pub async fn browse_workshop(
        &mut self,
        query: &crate::BrowseQuery,
    ) -> Result<crate::BrowsePage, InstallError> {
        query.validate()?;
        let response = self.cm.call(&query.to_request()).await?;

        // One PICS lookup for the app, not one per result: a page is twenty
        // items from the same Workshop.
        let workshop_depot = if query.app.get() == 0 {
            None
        } else {
            tapline_pics::product_info(&mut self.cm, query.app)
                .await
                .ok()
                .and_then(|info| info.workshop_depot())
        };

        let mut items = Vec::with_capacity(response.publishedfiledetails.len());
        let mut skipped = Vec::new();
        for details in &response.publishedfiledetails {
            match crate::browse::describe(details, workshop_depot) {
                Ok(found) => items.push(found),
                Err(why) => skipped.push(why),
            }
        }

        // Steam repeats the cursor when there is nothing after it, so an
        // unchanged cursor is the end rather than an invitation to loop.
        let sent = query.cursor.as_deref().unwrap_or(crate::FIRST_PAGE);
        let next_cursor = response
            .next_cursor
            .filter(|cursor| !cursor.is_empty() && cursor != sent && !items.is_empty());

        Ok(crate::BrowsePage {
            items,
            total: response.total.unwrap_or(0),
            next_cursor,
            skipped,
        })
    }

    /// Counts what a search would match, without fetching any of it.
    ///
    /// Steam answers a `totalonly` query with the count and no items, which is
    /// what a filter sidebar needs: the number beside each option costs one
    /// round trip rather than a page of results that gets thrown away. The
    /// PICS lookup that `browse_workshop` does for the workshop depot is
    /// skipped too, since nothing here is described.
    ///
    /// # Errors
    /// The same as [`Session::browse_workshop`]: an invalid query, or the call
    /// failing.
    pub async fn count_workshop(
        &mut self,
        query: &crate::BrowseQuery,
    ) -> Result<u32, InstallError> {
        query.validate()?;
        let mut request = query.to_request();
        request.totalonly = Some(true);
        // Asking for a page of results Steam is not going to send.
        request.numperpage = Some(1);
        let response = self.cm.call(&request).await?;
        Ok(response.total.unwrap_or(0))
    }

    /// Subscribes this account to a Workshop item.
    ///
    /// What the "Subscribe" button does. Downloading an item and subscribing to
    /// it are separate things: a download gets the files now, a subscription
    /// tells Steam the account wants them, so the client keeps them updated and
    /// they follow the account to another machine.
    ///
    /// Needs an account. An anonymous session has no subscriptions, and Steam
    /// refuses rather than quietly doing nothing — see [`Session::automatic`]
    /// for signing in.
    ///
    /// `include_dependencies` also subscribes to whatever the item requires,
    /// which is what the website does when an addon lists prerequisites. Off by
    /// default, because subscribing an account to things it did not ask for is
    /// a surprise.
    ///
    /// # Errors
    /// When the call fails, or the session is not signed in.
    pub async fn subscribe_workshop_item(
        &mut self,
        app: AppId,
        item: PublishedFileId,
        include_dependencies: bool,
    ) -> Result<(), InstallError> {
        self.cm
            .call(&CPublishedFile_Subscribe_Request {
                publishedfileid: Some(item.get()),
                // Steam wants the app the item belongs to; without it the
                // subscription applies to nothing and still answers success.
                appid: i32::try_from(app.get()).ok(),
                // So a running Steam client notices rather than finding out on
                // its next sync.
                notify_client: Some(true),
                include_dependencies: Some(include_dependencies),
                ..CPublishedFile_Subscribe_Request::default()
            })
            .await?;
        Ok(())
    }

    /// Tells Steam this account no longer wants an item.
    ///
    /// The other half of removing a wallpaper: deleting the files leaves the
    /// subscription, and the Steam client downloads it again the next time it
    /// syncs. Needs an account — an anonymous session has no subscriptions to
    /// change.
    ///
    /// # Errors
    /// When the call fails, or the session is not signed in.
    pub async fn unsubscribe_workshop_item(
        &mut self,
        app: AppId,
        item: PublishedFileId,
    ) -> Result<(), InstallError> {
        self.cm
            .call(&CPublishedFile_Unsubscribe_Request {
                publishedfileid: Some(item.get()),
                // Steam wants the app the item belongs to; without it the
                // unsubscribe applies to nothing and still answers success.
                appid: i32::try_from(app.get()).ok(),
                // So a running Steam client notices rather than finding out on
                // its next sync.
                notify_client: Some(true),
                ..CPublishedFile_Unsubscribe_Request::default()
            })
            .await?;
        Ok(())
    }

    /// Downloads one Workshop item.
    ///
    /// SteamPipe items go through the same path as depot content — request
    /// code, manifest, chunks, verify — because that is literally what they
    /// are. Legacy items are a single HTTPS fetch.
    pub async fn download_workshop_item(
        &mut self,
        item: &crate::WorkshopItem,
        options: &InstallOptions,
    ) -> Result<InstallReport, InstallError> {
        self.download_workshop_item_observed(item, options, &mut |_| {})
            .await
    }

    /// Downloads a Workshop item, reporting progress as it goes.
    ///
    /// Same observer contract as [`Session::install_observed`].
    pub async fn download_workshop_item_observed(
        &mut self,
        item: &crate::WorkshopItem,
        options: &InstallOptions,
        observe: &mut (dyn FnMut(Event) + Send),
    ) -> Result<InstallReport, InstallError> {
        let target = crate::options_for(options, item.app, item.id);
        std::fs::create_dir_all(&target.install_dir)?;

        let mut report = InstallReport {
            app: item.app,
            ..InstallReport::default()
        };

        match &item.content {
            crate::WorkshopContent::SteamPipe { depot, manifest } => {
                let key = self.depot_key(item.app, *depot).await?;

                let code = self
                    .cm
                    .call(&CContentServerDirectory_GetManifestRequestCode_Request {
                        app_id: Some(item.app.get()),
                        depot_id: Some(depot.get()),
                        manifest_id: Some(manifest.get()),
                        app_branch: Some("public".to_owned()),
                        branch_password_hash: None,
                    })
                    .await?
                    .manifest_request_code
                    .unwrap_or(0);

                let host = self.pool.acquire()?;
                let content = fetch_manifest(
                    self.shared.http.as_ref(),
                    &host.host,
                    *depot,
                    manifest.get(),
                    code,
                    Some(&key),
                )
                .await?;

                let entry = ResolvedDepot {
                    depot: tapline_pics::Depot {
                        id: *depot,
                        manifest: *manifest,
                        size: content.total_size,
                        download_size: 0,
                        owner: item.app,
                    },
                    manifest: content,
                    key,
                };
                report.depots.push(*depot);
                let bytes_total = entry.manifest.total_size;

                // Same contract as an app install: Planned first, so a consumer
                // has the denominator before the numerator moves. A Workshop
                // item used to emit nothing until its first file landed, which
                // meant the same progress code could not drive both.
                let (chunks, download_bytes) = entry.manifest.distinct_chunks();
                observe(Event::Planned {
                    plan: Plan {
                        download_bytes,
                        reused_bytes: 0,
                        total_bytes: bytes_total,
                        file_count: entry.manifest.regular_files().count() as u64,
                        chunk_count: chunks.len() as u64,
                    },
                });
                observe(Event::DepotStarted {
                    depot: *depot,
                    manifest: *manifest,
                    bytes: bytes_total,
                });
                self.install_depot(&entry, &target, &mut report, observe, bytes_total, item.app)
                    .await?;
                observe(Event::DepotCompleted { depot: *depot });
            }

            crate::WorkshopContent::Legacy { url, filename } => {
                // No chunking, no encryption, no manifest — just the blob. The
                // filename comes from the item, and is validated like any other
                // path from a manifest because it is just as attacker-authored.
                let name = filename.as_deref().unwrap_or("contents.bin");
                let safe = validate_path(name).map_err(|reason| InstallError::UnsafePath {
                    path: name.to_owned(),
                    reason,
                })?;

                let response = tapline_io::Fetch::get(
                    self.shared.http.as_ref(),
                    tapline_io::Request::get(url.clone()),
                    item.size.max(1) + 4096,
                )
                .await
                .map_err(|e| InstallError::Io(e.to_string()))?;

                if !response.is_success() {
                    return Err(InstallError::Io(format!(
                        "the Workshop CDN answered {} for {url}",
                        response.status
                    )));
                }

                let path = safe.resolve(&target.install_dir);
                let sink = FileSink::create(&path)?;
                sink.write_at(0, &response.body).await?;
                sink.sync().await?;

                report.files = 1;
                report.bytes_written = response.body.len() as u64;
                report.bytes_downloaded = response.body.len() as u64;
            }
        }

        Ok(report)
    }

    /// Checks an install against its manifests, hashing every chunk on disk.
    ///
    /// Answers the question an update takes on trust. Slower — it reads the
    /// whole install — and always right, which is the trade `validate` exists
    /// to make.
    pub async fn validate(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<crate::ValidationReport, InstallError> {
        let resolved = self.resolve(app, options).await?;

        let mut combined = crate::ValidationReport::default();
        for entry in &resolved {
            self.maybe_heartbeat().await?;

            let report = crate::validate_manifest(
                &entry.manifest,
                &options.install_dir,
                |path, offset, len| {
                    use std::os::unix::fs::FileExt;
                    let file = std::fs::File::open(path)?;
                    let mut buffer = vec![0_u8; len];
                    file.read_exact_at(&mut buffer, offset)?;
                    Ok(buffer)
                },
            );

            combined.files_checked += report.files_checked;
            combined.bytes_checked += report.bytes_checked;
            combined.damaged.extend(report.damaged);
        }
        Ok(combined)
    }

    /// Downloads one depot's files.
    ///
    /// Chunks are fetched concurrently across the host pool. Sequentially, a
    /// 1.47 GB Valheim install took 238 seconds — one request at a time, each
    /// waiting a full round trip before the next began, which leaves the link
    /// idle for most of the download.
    ///
    /// The concurrency is bounded and the bound is not decoration: Steam rate
    /// limits per host, and a download that opens fifty connections to one
    /// cache is a download that gets throttled. Work is spread across the pool
    /// rather than piled onto the least-loaded host for the same reason.
    #[allow(clippy::too_many_arguments)]
    async fn install_depot(
        &mut self,
        entry: &ResolvedDepot,
        options: &InstallOptions,
        report: &mut InstallReport,
        observe: &mut (dyn FnMut(Event) + Send),
        bytes_total: u64,
        app: AppId,
    ) -> Result<(), InstallError> {
        // Directories first, so a file never races the directory it lives in.
        for file in &entry.manifest.files {
            if !file.flags.directory {
                continue;
            }
            let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
                path: file.path.clone(),
                reason,
            })?;
            std::fs::create_dir_all(safe.resolve(&options.install_dir))?;
        }

        // A snapshot of the pool, so the fetch tasks need no lock on it. Health
        // is tracked per task and folded back afterwards.
        let hosts: Vec<String> = self.pool.snapshot();
        if hosts.is_empty() {
            return Err(InstallError::Pool(tapline_cdn::PoolError::Empty));
        }

        // Two bounds, and both are needed.
        //
        // The shared one is the process's total: every session built on the
        // same `Shared` draws from it, so three downloads split 64 chunks in
        // flight rather than taking 64 each. Past 64 the throughput curve turns
        // over, so multiplying it by the number of downloads makes all of them
        // slower.
        //
        // The local one stops a single download from holding the whole budget
        // while another waits behind it. With one download running they are the
        // same number and it costs nothing.
        let shared_limit = Arc::clone(&self.shared.limit);
        let local_limit = Arc::new(tokio::sync::Semaphore::new(options.concurrency.max(1)));
        let next_host = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut tasks: tokio::task::JoinSet<(usize, Result<ChunkOutcome, InstallError>)> =
            tokio::task::JoinSet::new();

        // Finalisations run here rather than inline, so an fsync never stops the
        // loop from dispatching the next chunk.
        let mut finalizing: tokio::task::JoinSet<Result<Finished, InstallError>> =
            tokio::task::JoinSet::new();

        let mut files_written = 0_u64;
        // Files whose chunks are still in flight, by index. A file leaves this
        // map — and closes its descriptor — as soon as its own last chunk
        // lands, which is the whole point: holding every sink until the depot
        // finished meant one open descriptor per file in the depot. Garry's Mod
        // has 2,329 of them and the default limit is 1,024, so it failed with
        // "Too many open files" on any machine that had not raised the limit.
        let mut pending: std::collections::BTreeMap<usize, PendingFile> =
            std::collections::BTreeMap::new();

        for (index, file) in entry.manifest.regular_files().enumerate() {
            self.maybe_heartbeat().await?;

            let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
                path: file.path.clone(),
                reason,
            })?;
            let target = safe.resolve(&options.install_dir);

            // A file already the right length may be a resumed download or a
            // partially-correct one. Reading a chunk back and hashing it
            // answers "is this range already right?" directly, without trusting
            // any record of what happened last time.
            let existing = if options.resume {
                std::fs::metadata(&target)
                    .ok()
                    .filter(|metadata| metadata.len() == file.size)
                    .and_then(|_| FileSink::open_existing(&target).ok())
            } else {
                None
            };
            let resuming = existing.is_some();

            let sink = Arc::new(match existing {
                Some(sink) => sink,
                None => {
                    let sink = FileSink::create(&target)?;
                    sink.allocate(file.size).await?;
                    sink
                }
            });

            for chunk in &file.chunks {
                // Before the permit, not after: acquiring blocks once the
                // window is full, and a file with many chunks would otherwise
                // sit here past the deadline without touching the CM.
                self.maybe_heartbeat().await?;

                // Local before shared, always in that order. Two semaphores
                // taken in a consistent order cannot deadlock against each
                // other; taken in different orders by different downloads they
                // could.
                let local_permit = Arc::clone(&local_limit)
                    .acquire_owned()
                    .await
                    .map_err(|e| InstallError::Io(e.to_string()))?;
                let shared_permit = Arc::clone(&shared_limit)
                    .acquire_owned()
                    .await
                    .map_err(|e| InstallError::Io(e.to_string()))?;

                let sink = Arc::clone(&sink);
                let http = Arc::clone(&self.shared.http);
                let hosts = hosts.clone();
                let next_host = Arc::clone(&next_host);
                let chunk = chunk.clone();
                let key = entry.key;
                let depot = entry.depot.id;

                tasks.spawn(async move {
                    // Held for the life of the task, which is what bounds the
                    // concurrency — released on the error paths too.
                    let _permits = (local_permit, shared_permit);
                    let outcome = async move {
                        if resuming
                            && let Ok(bytes) =
                                sink.read_at(chunk.offset, chunk.uncompressed_size as usize)
                            && tapline_crypto::sha1(&bytes) == chunk.id
                        {
                            // Already correct: a read instead of a transfer.
                            return Ok(ChunkOutcome::reused());
                        }

                        let mut last_error = None;
                        for attempt in 0..4_usize {
                            // Round-robin rather than always the best host.
                            // Concentrating a depot's requests on one host is what
                            // triggers rate limiting.
                            let index = next_host
                                .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
                                .wrapping_add(attempt);
                            let host = hosts.get(index % hosts.len()).cloned().unwrap_or_default();

                            // The fetch is IO; the decode is not. Decrypting,
                            // decompressing and hashing a megabyte is real CPU
                            // work, and leaving it on an async worker means one
                            // chunk's LZMA stalls every other task sharing that
                            // thread. 2,404 chunks of it is the difference between
                            // saturating a link and saturating one core.
                            match fetch_chunk_bytes(http.as_ref(), &host, depot, &chunk).await {
                                Ok(stored) => {
                                    let for_decode = chunk.clone();
                                    let host_for_decode = host.clone();
                                    let decoded = tokio::task::spawn_blocking(move || {
                                        tapline_cdn::decode_chunk_owned(
                                            stored,
                                            &for_decode,
                                            &key,
                                            &host_for_decode,
                                        )
                                    })
                                    .await
                                    .map_err(|e| InstallError::Io(e.to_string()))?;

                                    match decoded {
                                        Ok(plaintext) => {
                                            sink.write_at(chunk.offset, &plaintext).await?;
                                            return Ok(ChunkOutcome::fetched(
                                                &host,
                                                u64::from(chunk.compressed_size),
                                                plaintext.len() as u64,
                                            ));
                                        }
                                        Err(error) => last_error = Some((host, error)),
                                    }
                                }
                                Err(error) => {
                                    // A host that served a chunk failing its hash
                                    // check is not one to ask again — that is the
                                    // shape of a poisoned cache, and retrying it
                                    // returns the same wrong bytes.
                                    last_error = Some((host, error));
                                }
                            }
                        }

                        match last_error {
                            Some((_, error)) => Err(error.into()),
                            None => Err(InstallError::Pool(tapline_cdn::PoolError::AllDemoted)),
                        }
                    }
                    .await;
                    (index, outcome)
                });
            }

            let outstanding = file.chunks.len();
            let entry_pending = PendingFile {
                sink,
                target,
                path: file.path.clone(),
                size: file.size,
                root: options.install_dir.clone(),
                app,
                extensions: Arc::clone(&self.extensions),
                mode: options.file_modes.mode_for(file.flags.executable),
                outstanding,
            };
            files_written += 1;

            if outstanding == 0 {
                // A zero-length file has no chunks, so nothing will ever finish
                // for it. Queue it or its descriptor is never released.
                finalizing.spawn_blocking(move || finalize_file(entry_pending));
            } else {
                pending.insert(index, entry_pending);
            }

            // Drain finished tasks as we go, so results are folded back, the
            // set does not grow without bound, and finished files release their
            // descriptors.
            while let Some(joined) = tasks.try_join_next() {
                apply_outcome(
                    joined,
                    report,
                    &mut self.pool,
                    &mut pending,
                    &mut finalizing,
                )?;
                observe(Event::Progress {
                    bytes_done: report.bytes_written,
                    bytes_total,
                });
            }
            while let Some(joined) = finalizing.try_join_next() {
                collect_finalize(joined, observe)?;
            }

            // A hard ceiling on open descriptors, independent of how a depot
            // happens to distribute chunks across files. The chunk semaphore
            // already bounds this in practice; this is the guarantee rather
            // than the expectation.
            while pending.len() + finalizing.len() >= MAX_OPEN_FILES {
                // A file being synced still holds its descriptor, so it counts
                // against the ceiling until the sync returns.
                if let Some(joined) = finalizing.try_join_next() {
                    collect_finalize(joined, observe)?;
                    continue;
                }
                let Some(joined) = tasks.join_next().await else {
                    let Some(joined) = finalizing.join_next().await else {
                        break;
                    };
                    collect_finalize(joined, observe)?;
                    continue;
                };
                apply_outcome(
                    joined,
                    report,
                    &mut self.pool,
                    &mut pending,
                    &mut finalizing,
                )?;
                self.maybe_heartbeat().await?;
            }
        }

        // Everything still in flight. The heartbeat matters most here: every
        // file has been queued, so this loop is pure waiting, and without it
        // the connection goes quiet for as long as the tail takes. That is how
        // this failed the first time it ran — a broken pipe 101 seconds in.
        while let Some(joined) = tasks.join_next().await {
            apply_outcome(
                joined,
                report,
                &mut self.pool,
                &mut pending,
                &mut finalizing,
            )?;
            self.maybe_heartbeat().await?;
        }

        // Every file must be on disk before the install record claims it is.
        while let Some(joined) = finalizing.join_next().await {
            collect_finalize(joined, observe)?;
            self.maybe_heartbeat().await?;
        }

        // Nothing should be left: every file's chunks have been accounted for.
        // Anything still here would be a file silently left unsynced, so it is
        // an error rather than a cleanup step.
        if let Some((_, leftover)) = pending.pop_first() {
            return Err(InstallError::Io(format!(
                "{} still had {} chunks outstanding after every task finished",
                leftover.target.display(),
                leftover.outstanding
            )));
        }
        report.files += files_written;

        // Symlinks last: their targets must exist, and their validation is
        // separate because a link is the indirect form of a traversal.
        for file in &entry.manifest.files {
            if !file.flags.symlink {
                continue;
            }
            let Some(link_target) = &file.link_target else {
                report
                    .skipped
                    .push((file.path.clone(), "a symlink with no target".to_owned()));
                continue;
            };

            let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
                path: file.path.clone(),
                reason,
            })?;
            let resolved_target =
                tapline_fs::validate_symlink(&safe, link_target).map_err(|reason| {
                    InstallError::UnsafePath {
                        path: file.path.clone(),
                        reason,
                    }
                })?;

            let link_path = safe.resolve(&options.install_dir);
            if let Some(parent) = link_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let _ = std::fs::remove_file(&link_path);
            std::os::unix::fs::symlink(&resolved_target, &link_path)?;
            report.files += 1;
        }

        Ok(())
    }
}

/// What one chunk task did.
struct ChunkOutcome {
    /// The host that served it, when one did.
    host: Option<String>,
    /// Bytes fetched from the CDN.
    downloaded: u64,
    /// Bytes written to disk.
    written: u64,
    /// Whether the chunk was already correct.
    reused: bool,
}

impl ChunkOutcome {
    const fn reused() -> Self {
        Self {
            host: None,
            downloaded: 0,
            written: 0,
            reused: true,
        }
    }

    fn fetched(host: &str, downloaded: u64, written: u64) -> Self {
        Self {
            host: Some(host.to_owned()),
            downloaded,
            written,
            reused: false,
        }
    }
}

/// Folds a finished chunk task back into the report and the pool's health.
/// A file whose chunks are still being fetched.
struct PendingFile {
    /// The open descriptor being written through.
    sink: Arc<FileSink>,
    /// Where it lands.
    target: std::path::PathBuf,
    /// The manifest's own path for it, which is what a consumer recognises.
    path: String,
    /// Its size, so a completion event can report it without a stat.
    size: u64,
    /// The install root. An extension must not write outside it.
    root: std::path::PathBuf,
    /// The app this file belongs to, for the extension's context.
    app: AppId,
    /// The extensions to offer this file to.
    extensions: Arc<Vec<Arc<dyn tapline_ext::Extension>>>,
    /// The mode to apply once it is complete.
    mode: u32,
    /// How many of its chunks have not finished yet.
    outstanding: usize,
}

/// The most files that may be open at once during a depot.
///
/// Well under the usual 1,024 soft limit, because a process that linked tapline
/// has its own descriptors and this budget is not the whole of it.
const MAX_OPEN_FILES: usize = 64;

/// Syncs a finished file, applies its mode, and closes it.
///
/// Blocking on purpose, and never called from the task that dispatches chunks.
/// `fsync` on a multi-megabyte file is not quick: measured over a Garry's Mod
/// install it was 13.5 seconds of a 41-second wall clock, and every one of
/// those seconds was time the dispatch loop spent not starting new fetches,
/// draining all sixteen slots to idle. Off the critical path it overlaps with
/// the downloads instead.
fn finalize_file(file: PendingFile) -> Result<Finished, InstallError> {
    file.sink
        .sync_blocking()
        .map_err(|error| InstallError::Io(error.to_string()))?;
    set_permissions(&file.target, file.mode)?;
    // Before the extensions, not after: an extension reads the file back from
    // disk, and until the sync returns there is no guarantee of what is there.
    drop(file.sink);

    let mut extended = Vec::new();
    if !file.extensions.is_empty() {
        let landed = tapline_ext::Landed {
            app: file.app,
            root: &file.root,
            path: &file.path,
            full_path: &file.target,
            bytes: file.size,
        };
        for extension in file.extensions.iter() {
            if !extension.claims(&landed) {
                continue;
            }
            // An extension that was asked for and could not run has left the
            // install in a state the caller did not ask for, so this fails the
            // install rather than leaving a quietly half-processed tree.
            let produced = extension
                .run(&landed)
                .map_err(|error| InstallError::Io(error.to_string()))?;
            if produced.remove_original {
                std::fs::remove_file(&file.target)?;
            }
            extended.push((extension.name().to_owned(), produced.files.len() as u64));
        }
    }

    Ok(Finished {
        path: file.path,
        bytes: file.size,
        extended,
    })
}

/// Fetches one chunk and decodes it, trying other hosts on failure.
///
/// Shared by the streaming download and ranged reads, which want the same
/// thing: bytes for a chunk, verified, from whichever host will serve them.
pub(crate) async fn fetch_and_decode(
    http: &Arc<tapline_rt_tokio::HttpClient>,
    hosts: &[String],
    depot: DepotId,
    chunk: &tapline_manifest::Chunk,
    key: &[u8; 32],
    rotation: usize,
) -> Result<Vec<u8>, InstallError> {
    if hosts.is_empty() {
        return Err(InstallError::Pool(tapline_cdn::PoolError::Empty));
    }
    let mut last_error = None;
    for attempt in 0..4_usize {
        // Rotated per caller so concurrent reads spread across hosts rather
        // than all starting on the same one.
        let host = hosts
            .get((rotation + attempt) % hosts.len())
            .cloned()
            .unwrap_or_default();
        match fetch_chunk_bytes(http.as_ref(), &host, depot, chunk).await {
            Ok(stored) => {
                let for_decode = chunk.clone();
                let key = *key;
                let host_for_decode = host.clone();
                let decoded = tokio::task::spawn_blocking(move || {
                    tapline_cdn::decode_chunk_owned(stored, &for_decode, &key, &host_for_decode)
                })
                .await
                .map_err(|error| InstallError::Io(error.to_string()))?;
                match decoded {
                    Ok(plaintext) => return Ok(plaintext),
                    Err(error) => last_error = Some(error),
                }
            }
            Err(error) => last_error = Some(error),
        }
    }
    Err(last_error.map_or(
        InstallError::Pool(tapline_cdn::PoolError::AllDemoted),
        Into::into,
    ))
}

/// One streamed chunk: its plaintext, what it cost to fetch, and from where.
type StreamedChunk = (Vec<u8>, u64, String);

/// The tasks fetching a streamed file, tagged with each chunk's index.
type StreamTasks = tokio::task::JoinSet<(usize, Result<StreamedChunk, InstallError>)>;

/// What finalising a file produced.
struct Finished {
    /// The file's path, as the manifest spells it.
    path: String,
    /// Its size.
    bytes: u64,
    /// Extensions that ran, and how many files each produced.
    extended: Vec<(String, u64)>,
}

/// Unwraps a finished finalisation and reports the file it completed.
fn collect_finalize(
    joined: Result<Result<Finished, InstallError>, tokio::task::JoinError>,
    observe: &mut (dyn FnMut(Event) + Send),
) -> Result<(), InstallError> {
    let Finished {
        path,
        bytes,
        extended,
    } = joined
        .map_err(|error| InstallError::Io(format!("a file could not be finalised: {error}")))??;
    // Emitted here rather than when the last chunk lands: until the sync
    // returns, the file is not on disk, and a consumer acting on this event
    // (hashing it, launching it) would be acting on a file that is not there.
    observe(Event::FileCompleted {
        path: path.clone(),
        bytes,
    });
    for (extension, produced) in extended {
        observe(Event::Extended {
            extension,
            path: path.clone(),
            produced,
        });
    }
    Ok(())
}

fn apply_outcome(
    joined: Result<(usize, Result<ChunkOutcome, InstallError>), tokio::task::JoinError>,
    report: &mut InstallReport,
    pool: &mut HostPool,
    pending: &mut std::collections::BTreeMap<usize, PendingFile>,
    finalizing: &mut tokio::task::JoinSet<Result<Finished, InstallError>>,
) -> Result<(), InstallError> {
    let (index, outcome) =
        joined.map_err(|error| InstallError::Io(format!("a download task failed: {error}")))?;

    // The file is finished either way: a failed chunk fails the install, and a
    // successful one is one fewer outstanding. Decrement before the `?` so a
    // failure does not leave a stale entry behind.
    let finished = match pending.get_mut(&index) {
        Some(file) => {
            file.outstanding = file.outstanding.saturating_sub(1);
            file.outstanding == 0
        }
        None => false,
    };
    let outcome = outcome?;
    if finished && let Some(file) = pending.remove(&index) {
        finalizing.spawn_blocking(move || finalize_file(file));
    }

    if outcome.reused {
        report.chunks_reused += 1;
        return Ok(());
    }
    if let Some(host) = &outcome.host {
        pool.succeed(host);
    }
    report.bytes_downloaded += outcome.downloaded;
    report.bytes_written += outcome.written;
    Ok(())
}

/// Applies a mode chosen by the install's [`FileModes`] policy.
fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
}

/// The current time as a Unix timestamp.
///
/// Used only for `LastUpdated` in the install record, which is informational —
/// a clock that is wrong makes the field wrong and nothing else.
fn now_unix() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn executables_get_the_bit_a_launcher_needs() {
        use std::os::unix::fs::PermissionsExt;

        let dir = std::env::var("TAPLINE_TEST_DIR").map_or_else(
            |_| {
                std::path::PathBuf::from(std::env::var("HOME").unwrap_or_else(|_| ".".into()))
                    .join(".cache/tapline-test")
            },
            std::path::PathBuf::from,
        );
        let path = dir.join("permissions-probe");
        let _ = std::fs::create_dir_all(&dir);
        std::fs::write(&path, b"#!/bin/sh\n").expect("write");

        let mode_of = |policy: crate::FileModes, executable: bool| {
            set_permissions(&path, policy.mode_for(executable)).expect("chmod");
            std::fs::metadata(&path).expect("stat").permissions().mode() & 0o777
        };

        // The manifest policy says what the depot says.
        assert_eq!(
            mode_of(crate::FileModes::Manifest, true),
            0o755,
            "an executable must be runnable"
        );
        assert_eq!(mode_of(crate::FileModes::Manifest, false), 0o644);

        // The default matches steamcmd, which sets 0o755 on everything. A start
        // script whose manifest forgot the flag still has to be runnable, or
        // tapline is not a drop-in replacement for the tools that wrap it.
        assert_eq!(mode_of(crate::FileModes::SteamCmd, false), 0o755);
        assert_eq!(mode_of(crate::FileModes::SteamCmd, true), 0o755);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_default_mode_policy_is_the_compatible_one() {
        // Stated as a test because changing this default silently would break
        // installs that only fail at the moment someone tries to start a
        // server, far away from the change that caused it.
        assert_eq!(
            crate::InstallOptions::default().file_modes,
            crate::FileModes::SteamCmd
        );
    }
}
