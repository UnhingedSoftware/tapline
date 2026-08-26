//! A Steam session that can plan and perform installs.

use crate::{InstallError, InstallOptions, InstallReport};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tapline_cdn::{Host, HostPool, fetch_chunk_bytes, fetch_manifest};
use tapline_event::Plan;
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
use tapline_proto::steammessages_publishedfile_steamclient::CPublishedFile_GetDetails_Request;
use tapline_rt_tokio::{CmTransport, FileSink, HttpClient, cm_list};
use tapline_state::AppState;
use tapline_wire::Message;

/// Steam's result code for success.
const RESULT_OK: i32 = 1;

/// A logged-on Steam session.
pub struct Session {
    cm: CmSession<CmTransport>,
    http: Arc<HttpClient>,
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
            http: Arc::new(HttpClient::new()),
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
        };
        session.refresh_hosts().await?;
        Ok(session)
    }

    /// The cell Steam placed this session in, which decides which CDN hosts are
    /// nearby.
    #[must_use]
    pub const fn cell_id(&self) -> u32 {
        self.cell_id
    }

    /// Fetches the CDN host list for this cell.
    async fn refresh_hosts(&mut self) -> Result<(), InstallError> {
        let directory = self
            .cm
            .call(&CContentServerDirectory_GetServersForSteamPipe_Request {
                cell_id: Some(self.cell_id),
                max_servers: Some(20),
                ..CContentServerDirectory_GetServersForSteamPipe_Request::default()
            })
            .await?;

        let hosts: Vec<Host> = directory
            .servers
            .iter()
            .filter_map(|server| {
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
                self.http.as_ref(),
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
        let resolved = self.resolve(app, options).await?;
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

            self.install_depot(entry, options, &mut report).await?;
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
            .map_err(|e| crate::LoginError::Session(e.to_string()))?;

        Ok(crate::PendingLogin {
            client_id: response.client_id.unwrap_or(0),
            request_id: response.request_id.clone().unwrap_or_default(),
            interval: response.interval.unwrap_or(5.0),
            confirmations: crate::login::confirmations_from(&response.allowed_confirmations),
            challenge_url: response.challenge_url.clone(),
            account: None,
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
            .map_err(|e| crate::LoginError::Session(e.to_string()))?;

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
            .map_err(|e| crate::LoginError::Session(e.to_string()))?;

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
        })
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
            .map_err(|e| crate::LoginError::Session(e.to_string()))?;

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
                    self.http.as_ref(),
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
                self.install_depot(&entry, &target, &mut report).await?;
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
                    self.http.as_ref(),
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
    async fn install_depot(
        &mut self,
        entry: &ResolvedDepot,
        options: &InstallOptions,
        report: &mut InstallReport,
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

        let limit = Arc::new(tokio::sync::Semaphore::new(options.concurrency.max(1)));
        let next_host = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let mut tasks: tokio::task::JoinSet<Result<ChunkOutcome, InstallError>> =
            tokio::task::JoinSet::new();

        let mut files_written = 0_u64;
        let mut pending_sinks: Vec<(Arc<FileSink>, std::path::PathBuf, bool)> = Vec::new();

        for file in entry.manifest.regular_files() {
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

                let permit = Arc::clone(&limit)
                    .acquire_owned()
                    .await
                    .map_err(|e| InstallError::Io(e.to_string()))?;

                let sink = Arc::clone(&sink);
                let http = Arc::clone(&self.http);
                let hosts = hosts.clone();
                let next_host = Arc::clone(&next_host);
                let chunk = chunk.clone();
                let key = entry.key;
                let depot = entry.depot.id;

                tasks.spawn(async move {
                    // The permit is held for the life of the task, which is what
                    // bounds the concurrency.
                    let _permit = permit;

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
                                    tapline_cdn::decode_chunk(
                                        &stored,
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
                });
            }

            pending_sinks.push((sink, target, file.flags.executable));
            files_written += 1;

            // Drain finished tasks as we go, so results are folded back and the
            // set does not grow without bound on a depot with many files.
            while let Some(joined) = tasks.try_join_next() {
                apply_outcome(joined, report, &mut self.pool)?;
            }
        }

        // Everything still in flight. The heartbeat matters most here: every
        // file has been queued, so this loop is pure waiting, and without it
        // the connection goes quiet for as long as the tail takes. That is how
        // this failed the first time it ran — a broken pipe 101 seconds in.
        while let Some(joined) = tasks.join_next().await {
            apply_outcome(joined, report, &mut self.pool)?;
            self.maybe_heartbeat().await?;
        }

        for (sink, target, executable) in pending_sinks {
            sink.sync().await?;
            set_permissions(&target, options.file_modes.mode_for(executable))?;
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
fn apply_outcome(
    joined: Result<Result<ChunkOutcome, InstallError>, tokio::task::JoinError>,
    report: &mut InstallReport,
    pool: &mut HostPool,
) -> Result<(), InstallError> {
    let outcome =
        joined.map_err(|error| InstallError::Io(format!("a download task failed: {error}")))??;

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
