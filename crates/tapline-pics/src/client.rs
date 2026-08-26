//! Running the PICS exchange over a session.
//!
//! Two round trips, in this order:
//!
//! 1. `PICSAccessTokenRequest` — many apps refuse product info without a token.
//!    An app that needs no token is granted one of zero, which is not the same
//!    as being denied.
//! 2. `PICSProductInfoRequest` — the document itself.
//!
//! Denial is reported rather than worked around. An app in `app_denied_tokens`
//! is one the signed-in account may not see, and asking for its product info
//! anyway returns a stub with `missing_token` set — which parses fine and
//! describes nothing.

use crate::AppInfo;
use std::fmt;
use tapline_ids::AppId;
use tapline_io::Transport;
use tapline_net::{EMsg, Frame, NetError, Session};
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_appinfo::{
    CMsgClientPICSAccessTokenRequest, CMsgClientPICSAccessTokenResponse,
    CMsgClientPICSProductInfoRequest, CMsgClientPICSProductInfoResponse,
    c_msg_client_pics_product_info_request,
};
use tapline_wire::Message;

/// What went wrong asking PICS about an app.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PicsError {
    /// The session failed.
    Net(NetError),
    /// Steam does not know this app.
    UnknownApp(AppId),
    /// The account may not see this app.
    ///
    /// Distinct from [`PicsError::UnknownApp`]: the app exists, and a different
    /// account could read it. For anonymous sessions this usually means the app
    /// is not anonymously accessible.
    AccessDenied(AppId),
    /// Steam answered without the document.
    ///
    /// Happens when a response is large enough that Steam offers it over HTTP
    /// instead, which is a path tapline does not need for a single app and does
    /// not pretend to have taken.
    NoBuffer(AppId),
    /// The document did not parse.
    Malformed {
        /// Which app.
        app: AppId,
        /// Why.
        reason: String,
    },
}

impl fmt::Display for PicsError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Net(e) => write!(f, "{e}"),
            Self::UnknownApp(app) => write!(f, "Steam does not know app {app}"),
            Self::AccessDenied(app) => write!(f, "access to app {app} was denied"),
            Self::NoBuffer(app) => write!(f, "PICS returned no document for app {app}"),
            Self::Malformed { app, reason } => {
                write!(f, "the PICS document for app {app} did not parse: {reason}")
            }
        }
    }
}

impl std::error::Error for PicsError {}

impl From<NetError> for PicsError {
    fn from(error: NetError) -> Self {
        Self::Net(error)
    }
}

/// Fetches and parses one app's PICS document.
pub async fn product_info<T: Transport>(
    session: &mut Session<T>,
    app: AppId,
) -> Result<AppInfo, PicsError> {
    let token = access_token(session, app).await?;

    let request = CMsgClientPICSProductInfoRequest {
        apps: vec![c_msg_client_pics_product_info_request::AppInfo {
            appid: Some(app.get()),
            access_token: token,
            only_public_obsolete: None,
        }],
        meta_data_only: Some(false),
        ..CMsgClientPICSProductInfoRequest::default()
    };

    let reply = request_response(session, EMsg::PICS_PRODUCT_INFO_REQUEST, &request).await?;
    let response: CMsgClientPICSProductInfoResponse = reply.decode_body()?;

    if response.unknown_appids.contains(&app.get()) {
        return Err(PicsError::UnknownApp(app));
    }

    let entry = response
        .apps
        .iter()
        .find(|candidate| candidate.appid == Some(app.get()))
        .ok_or(PicsError::UnknownApp(app))?;

    // `missing_token` means Steam answered with a stub. It parses fine and says
    // nothing, so treating it as success would hand the caller an app with no
    // depots and no explanation.
    if entry.missing_token == Some(true) {
        return Err(PicsError::AccessDenied(app));
    }

    let buffer = entry.buffer.as_ref().ok_or(PicsError::NoBuffer(app))?;

    AppInfo::parse(app, buffer).map_err(|error| PicsError::Malformed {
        app,
        reason: error.to_string(),
    })
}

/// Asks for an app's access token.
///
/// Returns `None` when Steam grants no token but does not deny one either, which
/// is a legitimate state for an app that needs none.
async fn access_token<T: Transport>(
    session: &mut Session<T>,
    app: AppId,
) -> Result<Option<u64>, PicsError> {
    let request = CMsgClientPICSAccessTokenRequest {
        appids: vec![app.get()],
        packageids: Vec::new(),
    };

    let reply = request_response(session, EMsg::PICS_ACCESS_TOKEN_REQUEST, &request).await?;
    let response: CMsgClientPICSAccessTokenResponse = reply.decode_body()?;

    if response.app_denied_tokens.contains(&app.get()) {
        return Err(PicsError::AccessDenied(app));
    }

    Ok(response
        .app_access_tokens
        .iter()
        .find(|token| token.appid == Some(app.get()))
        .and_then(|token| token.access_token))
}

/// Sends a request and waits for its reply, correlated by job id.
async fn request_response<T: Transport, R: Message>(
    session: &mut Session<T>,
    emsg: EMsg,
    request: &R,
) -> Result<Frame, NetError> {
    let job_id = session.next_job_id();

    let header = CMsgProtoBufHeader {
        client_sessionid: Some(session.session_id()),
        steamid: Some(session.steam_id()),
        jobid_source: Some(job_id),
        ..CMsgProtoBufHeader::default()
    };

    session
        .send(&Frame::new(emsg, header, request.encode_to_vec()))
        .await?;
    session.wait_for_job(job_id).await
}
