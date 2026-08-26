//! A Steam session that can plan and perform installs.

use crate::{InstallError, InstallOptions, InstallReport};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tapline_cdn::{Host, HostPool, fetch_chunk, fetch_manifest};
use tapline_event::Plan;
use tapline_fs::validate_path;
use tapline_ids::{AppId, DepotId};
use tapline_io::Sink;
use tapline_manifest::Manifest;
use tapline_net::{EMsg, Frame, Session as CmSession};
use tapline_pics::Depot;
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_2::{
    CMsgClientGetDepotDecryptionKey, CMsgClientGetDepotDecryptionKeyResponse,
};
use tapline_proto::steammessages_contentsystem_steamclient::{
    CContentServerDirectory_GetManifestRequestCode_Request,
    CContentServerDirectory_GetServersForSteamPipe_Request,
};
use tapline_rt_tokio::{CmTransport, FileSink, HttpClient, cm_list};
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

    /// Installs an app.
    pub async fn install(
        &mut self,
        app: AppId,
        options: &InstallOptions,
    ) -> Result<InstallReport, InstallError> {
        let resolved = self.resolve(app, options).await?;
        std::fs::create_dir_all(&options.install_dir)?;

        let mut report = InstallReport {
            app,
            ..InstallReport::default()
        };

        for entry in &resolved {
            report.depots.push(entry.depot.id);
            self.install_depot(entry, options, &mut report).await?;
        }
        Ok(report)
    }

    /// Downloads one depot's files.
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

        for file in entry.manifest.regular_files() {
            let safe = validate_path(&file.path).map_err(|reason| InstallError::UnsafePath {
                path: file.path.clone(),
                reason,
            })?;
            let target = safe.resolve(&options.install_dir);

            let sink = FileSink::create(&target)?;
            sink.allocate(file.size).await?;

            for chunk in &file.chunks {
                let mut attempts = 0_u32;
                let plaintext = loop {
                    let host = self.pool.acquire()?;
                    match fetch_chunk(
                        self.http.as_ref(),
                        &host.host,
                        entry.depot.id,
                        chunk,
                        &entry.key,
                    )
                    .await
                    {
                        Ok(bytes) => {
                            self.pool.succeed(&host.host);
                            break bytes;
                        }
                        Err(error) => {
                            // A host that served a chunk failing its hash check
                            // is not a host to ask again — that is the shape of
                            // a poisoned cache, and retrying it returns the same
                            // wrong bytes.
                            self.pool.demote(&host.host);
                            attempts += 1;
                            if attempts >= 4 {
                                return Err(error.into());
                            }
                        }
                    }
                };

                report.bytes_downloaded += u64::from(chunk.compressed_size);
                sink.write_at(chunk.offset, &plaintext).await?;
                report.bytes_written += plaintext.len() as u64;
            }

            sink.sync().await?;
            set_permissions(&target, file.flags.executable)?;
            report.files += 1;
        }

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

/// Applies the executable bit the manifest asked for.
fn set_permissions(path: &Path, executable: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    let mut permissions = std::fs::metadata(path)?.permissions();
    // 0o755 for executables, 0o644 otherwise — what Steam itself produces, and
    // what a server launcher expects to find.
    permissions.set_mode(if executable { 0o755 } else { 0o644 });
    std::fs::set_permissions(path, permissions)
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

        set_permissions(&path, true).expect("chmod");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o777, 0o755, "an executable must be runnable");

        set_permissions(&path, false).expect("chmod");
        let mode = std::fs::metadata(&path).expect("stat").permissions().mode();
        assert_eq!(mode & 0o777, 0o644);

        let _ = std::fs::remove_file(&path);
    }
}
