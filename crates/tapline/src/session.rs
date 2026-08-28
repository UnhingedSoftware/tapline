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
    shared: Arc<crate::Shared>,
    extensions: Arc<Vec<Arc<dyn tapline_ext::Extension>>>,
    pool: HostPool,
    cell_id: u32,
    keys: HashMap<DepotId, [u8; 32]>,
    /// Steam drops a session that stops heartbeating, without saying why.
    heartbeat_interval: std::time::Duration,
    last_heartbeat: std::time::Instant,
    app_name: Option<String>,
    build_id: Option<u64>,
    account: Option<String>,
}

/// Steam answers a wrong password by failing the RPC with a result code.
fn login_failure(error: tapline_net::NetError) -> crate::LoginError {
    match error {
        tapline_net::NetError::Steam { eresult } => crate::LoginError::Refused {
            eresult,
            message: None,
        },
        other => crate::LoginError::Session(other.to_string()),
    }
}

struct ResolvedDepot {
    depot: Depot,
    manifest: Manifest,
    key: [u8; 32],
}

impl Session {
    /// Connects to Steam and logs on anonymously.
    pub async fn anonymous() -> Result<Self, InstallError> {
        Self::anonymous_shared(crate::Shared::new(InstallOptions::default().concurrency)).await
    }

    /// Connects and logs on anonymously, sharing resources with other sessions.
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
            // Steam asks for ~9s; halved because heartbeats ride between chunk fetches.
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

        // The logon header must name the account's SteamID, which the token carries.
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
    pub async fn automatic(account: Option<&str>) -> Result<Self, InstallError> {
        Self::automatic_shared(
            account,
            crate::Shared::new(InstallOptions::default().concurrency),
        )
        .await
    }

    /// A session that signs in if it can, sharing resources with others.
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
                    // Falls through to anonymous; the caller sees which via `Session::account`.
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

    /// The cell Steam placed this session in; decides which CDN hosts are nearby.
    #[must_use]
    pub const fn cell_id(&self) -> u32 {
        self.cell_id
    }

    /// 20 hosts stay warm; wider lists evict pooled connections and collapse throughput.
    const MAX_CDN_HOSTS: u32 = 20;

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
                    // Load is int32 on the wire; absent must sort last, not first.
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
    pub async fn keep_alive(&mut self) -> Result<(), InstallError> {
        self.cm.heartbeat().await?;
        self.last_heartbeat = std::time::Instant::now();
        Ok(())
    }

    async fn maybe_heartbeat(&mut self) -> Result<(), InstallError> {
        if self.last_heartbeat.elapsed() < self.heartbeat_interval {
            return Ok(());
        }
        self.cm.heartbeat().await?;
        self.last_heartbeat = std::time::Instant::now();
        Ok(())
    }

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
            // A borrowed depot's key belongs to the app that owns it.
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

    /// Streams a Workshop item's bytes to `consumer` in order, without writing it to disk.
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

        // The manifest does not promise offset order; the consumer needs it.
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

        // Started in order, at most `window.size` in flight: the buffer stays bounded.
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
                // Draws on the same process-wide budget as ordinary downloads.
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
    pub fn register(&mut self, extension: Arc<dyn tapline_ext::Extension>) {
        Arc::make_mut(&mut self.extensions).push(extension);
    }

    /// The extensions registered on this session.
    #[must_use]
    pub fn extensions(&self) -> &[Arc<dyn tapline_ext::Extension>] {
        &self.extensions
    }

    /// Installs or updates an app.
    pub async fn install(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<InstallReport, InstallError> {
        self.install_observed(app, options, &mut |_| {}).await
    }

    /// Installs, reporting progress as it goes.
    pub async fn install_observed(
        &mut self,
        app: AppId,
        options: &InstallOptions,
        observe: &mut (dyn FnMut(Event) + Send),
    ) -> Result<InstallReport, InstallError> {
        let resolved = self.resolve(app, options).await?;

        // Planned first: a progress bar needs the denominator before the numerator moves.
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
                // Already at this build; verifying bytes is `validate`'s job.
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

        // A depot the app no longer ships must leave the record.
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

        // Steam reports a failed password on the reply, not as an error.
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
                // Said now rather than after two minutes of doomed polling.
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
    pub async fn qr_login(
        &mut self,
        timeout: std::time::Duration,
        on_code: &mut (dyn FnMut(&str) + Send),
    ) -> Result<tapline_auth::StoredToken, crate::LoginError> {
        let mut current = self.begin_qr_login().await?;

        // Show the first code before any polling.
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
                    // The old code expired; polling it again waits forever.
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

        // A refreshed QR code moves the session; the old client id is dead.
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

        // PICS is asked once per app, not once per item.
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

        // An item Steam did not return is still an answer.
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
    pub async fn browse_workshop(
        &mut self,
        query: &crate::BrowseQuery,
    ) -> Result<crate::BrowsePage, InstallError> {
        query.validate()?;
        let response = self.cm.call(&query.to_request()).await?;

        // One PICS lookup per page, not one per result.
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

        // Steam repeats the cursor at the end; unchanged means stop, not loop.
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
    pub async fn count_workshop(
        &mut self,
        query: &crate::BrowseQuery,
    ) -> Result<u32, InstallError> {
        query.validate()?;
        let mut request = query.to_request();
        request.totalonly = Some(true);
        // Steam sends no items for a totalonly query; ask for the minimum.
        request.numperpage = Some(1);
        let response = self.cm.call(&request).await?;
        Ok(response.total.unwrap_or(0))
    }

    /// Subscribes this account to a Workshop item.
    pub async fn subscribe_workshop_item(
        &mut self,
        app: AppId,
        item: PublishedFileId,
        include_dependencies: bool,
    ) -> Result<(), InstallError> {
        self.cm
            .call(&CPublishedFile_Subscribe_Request {
                publishedfileid: Some(item.get()),
                // Without the appid the subscribe applies to nothing yet answers success.
                appid: i32::try_from(app.get()).ok(),
                // So a running Steam client notices immediately.
                notify_client: Some(true),
                include_dependencies: Some(include_dependencies),
                ..CPublishedFile_Subscribe_Request::default()
            })
            .await?;
        Ok(())
    }

    /// Tells Steam this account no longer wants an item.
    pub async fn unsubscribe_workshop_item(
        &mut self,
        app: AppId,
        item: PublishedFileId,
    ) -> Result<(), InstallError> {
        self.cm
            .call(&CPublishedFile_Unsubscribe_Request {
                publishedfileid: Some(item.get()),
                // Without the appid the unsubscribe applies to nothing yet answers success.
                appid: i32::try_from(app.get()).ok(),
                // So a running Steam client notices immediately.
                notify_client: Some(true),
                ..CPublishedFile_Unsubscribe_Request::default()
            })
            .await?;
        Ok(())
    }

    /// Downloads one Workshop item.
    pub async fn download_workshop_item(
        &mut self,
        item: &crate::WorkshopItem,
        options: &InstallOptions,
    ) -> Result<InstallReport, InstallError> {
        self.download_workshop_item_observed(item, options, &mut |_| {})
            .await
    }

    /// Downloads a Workshop item, reporting progress as it goes.
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

                // Planned first, same contract as an app install.
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
                // The filename is attacker-authored; validate it like any manifest path.
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
        create_directories(&entry.manifest, &options.install_dir)?;

        // Snapshot: fetch tasks take no lock; health folds back afterwards.
        let hosts: Vec<String> = self.pool.snapshot();
        if hosts.is_empty() {
            return Err(InstallError::Pool(tapline_cdn::PoolError::Empty));
        }

        // Both bounds are needed: shared caps the process, local stops one download hogging it.
        let shared_limit = Arc::clone(&self.shared.limit);
        let local_limit = Arc::new(tokio::sync::Semaphore::new(options.concurrency.max(1)));
        let fetch = ChunkContext {
            http: Arc::clone(&self.shared.http),
            hosts: Arc::new(hosts),
            next_host: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
            key: entry.key,
            depot: entry.depot.id,
        };
        let mut tasks: tokio::task::JoinSet<(usize, Result<ChunkOutcome, InstallError>)> =
            tokio::task::JoinSet::new();

        // An fsync must never stop the loop dispatching the next chunk.
        let mut finalizing: tokio::task::JoinSet<Result<Finished, InstallError>> =
            tokio::task::JoinSet::new();

        let mut files_written = 0_u64;
        // A file closes its descriptor when its last chunk lands, or big depots hit EMFILE.
        let mut pending: std::collections::BTreeMap<usize, PendingFile> =
            std::collections::BTreeMap::new();

        for (index, file) in entry.manifest.regular_files().enumerate() {
            self.maybe_heartbeat().await?;

            let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
                path: file.path.clone(),
                reason,
            })?;
            let target = safe.resolve(&options.install_dir);

            // Right-length files may be partial; hashing each chunk answers directly.
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
                // Before the permit: acquiring blocks, and the CM must not go quiet.
                self.maybe_heartbeat().await?;

                // Local before shared, always: a consistent order cannot deadlock.
                let local_permit = Arc::clone(&local_limit)
                    .acquire_owned()
                    .await
                    .map_err(|e| InstallError::Io(e.to_string()))?;
                let shared_permit = Arc::clone(&shared_limit)
                    .acquire_owned()
                    .await
                    .map_err(|e| InstallError::Io(e.to_string()))?;

                let sink = Arc::clone(&sink);
                let fetch = fetch.clone();
                let chunk = chunk.clone();

                tasks.spawn(async move {
                    // Held for the task's life; this is what bounds concurrency.
                    let _permits = (local_permit, shared_permit);
                    let outcome = fetch_decode_write_chunk(&fetch, &chunk, &sink, resuming).await;
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
                // A zero-length file has no chunks; queue it or its descriptor leaks.
                finalizing.spawn_blocking(move || finalize_file(entry_pending));
            } else {
                pending.insert(index, entry_pending);
            }

            // Drain as we go so descriptors release and the set stays bounded.
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

            // A hard ceiling on open descriptors, whatever the chunk distribution.
            while pending.len() + finalizing.len() >= MAX_OPEN_FILES {
                // A syncing file still holds its descriptor.
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

        // Pure waiting; without heartbeats here the CM connection dies.
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

        // Anything left is a file silently unsynced: an error, not cleanup.
        if let Some((_, leftover)) = pending.pop_first() {
            return Err(InstallError::Io(format!(
                "{} still had {} chunks outstanding after every task finished",
                leftover.target.display(),
                leftover.outstanding
            )));
        }
        report.files += files_written;

        create_symlinks(&entry.manifest, &options.install_dir, report)?;

        Ok(())
    }
}

#[derive(Clone)]
struct ChunkContext {
    http: Arc<tapline_rt_tokio::HttpClient>,
    hosts: Arc<Vec<String>>,
    next_host: Arc<std::sync::atomic::AtomicUsize>,
    key: [u8; 32],
    depot: DepotId,
}

async fn fetch_decode_write_chunk(
    ctx: &ChunkContext,
    chunk: &tapline_manifest::Chunk,
    sink: &FileSink,
    resuming: bool,
) -> Result<ChunkOutcome, InstallError> {
    if resuming
        && let Ok(bytes) = sink.read_at(chunk.offset, chunk.uncompressed_size as usize)
        && tapline_crypto::sha1(&bytes) == chunk.id
    {
        return Ok(ChunkOutcome::reused());
    }

    let mut last_error = None;
    for attempt in 0..4_usize {
        // Round-robin: concentrating requests on one host triggers rate limiting.
        let index = ctx
            .next_host
            .fetch_add(1, std::sync::atomic::Ordering::Relaxed)
            .wrapping_add(attempt);
        let host = ctx
            .hosts
            .get(index % ctx.hosts.len().max(1))
            .cloned()
            .unwrap_or_default();

        let stored = match fetch_chunk_bytes(ctx.http.as_ref(), &host, ctx.depot, chunk).await {
            Ok(stored) => stored,
            Err(error) => {
                last_error = Some(error);
                continue;
            }
        };

        let for_decode = chunk.clone();
        let host_for_decode = host.clone();
        let key = ctx.key;
        let decoded = tokio::task::spawn_blocking(move || {
            tapline_cdn::decode_chunk_owned(stored, &for_decode, &key, &host_for_decode)
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
            Err(error) => last_error = Some(error),
        }
    }

    match last_error {
        Some(error) => Err(error.into()),
        None => Err(InstallError::Pool(tapline_cdn::PoolError::AllDemoted)),
    }
}

/// Directories first, so a file never races the directory it lives in.
fn create_directories(
    manifest: &Manifest,
    install_dir: &std::path::Path,
) -> Result<(), InstallError> {
    for file in &manifest.files {
        if !file.flags.directory {
            continue;
        }
        let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
            path: file.path.clone(),
            reason,
        })?;
        std::fs::create_dir_all(safe.resolve(install_dir))?;
    }
    Ok(())
}

/// Links go last, and are validated: a link is an indirect path traversal.
fn create_symlinks(
    manifest: &Manifest,
    install_dir: &std::path::Path,
    report: &mut InstallReport,
) -> Result<(), InstallError> {
    for file in &manifest.files {
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

        let link_path = safe.resolve(install_dir);
        if let Some(parent) = link_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _ = std::fs::remove_file(&link_path);
        std::os::unix::fs::symlink(&resolved_target, &link_path)?;
        report.files += 1;
    }
    Ok(())
}

struct ChunkOutcome {
    host: Option<String>,
    downloaded: u64,
    written: u64,
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

struct PendingFile {
    sink: Arc<FileSink>,
    target: std::path::PathBuf,
    path: String,
    size: u64,
    root: std::path::PathBuf,
    app: AppId,
    extensions: Arc<Vec<Arc<dyn tapline_ext::Extension>>>,
    mode: u32,
    outstanding: usize,
}

/// The open-file ceiling; well under the usual 1,024 soft limit.
const MAX_OPEN_FILES: usize = 64;

/// Blocking on purpose; never called from the task that dispatches chunks.
fn finalize_file(file: PendingFile) -> Result<Finished, InstallError> {
    file.sink
        .sync_blocking()
        .map_err(|error| InstallError::Io(error.to_string()))?;
    set_permissions(&file.target, file.mode)?;
    // Extensions read the file back; the sync must land first.
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
            // A failed extension fails the install rather than leaving a half-processed tree.
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
        // Rotated per caller so concurrent reads spread across hosts.
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

type StreamedChunk = (Vec<u8>, u64, String);

type StreamTasks = tokio::task::JoinSet<(usize, Result<StreamedChunk, InstallError>)>;

struct Finished {
    path: String,
    bytes: u64,
    extended: Vec<(String, u64)>,
}

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
    // Emitted only after the sync: before that the file is not on disk.
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

    // Decrement before the `?` so a failure leaves no stale entry.
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

fn set_permissions(path: &Path, mode: u32) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(mode);
    std::fs::set_permissions(path, permissions)
}

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

        assert_eq!(
            mode_of(crate::FileModes::Manifest, true),
            0o755,
            "an executable must be runnable"
        );
        assert_eq!(mode_of(crate::FileModes::Manifest, false), 0o644);

        assert_eq!(mode_of(crate::FileModes::SteamCmd, false), 0o755);
        assert_eq!(mode_of(crate::FileModes::SteamCmd, true), 0o755);

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn the_default_mode_policy_is_the_compatible_one() {
        assert_eq!(
            crate::InstallOptions::default().file_modes,
            crate::FileModes::SteamCmd
        );
    }
}
