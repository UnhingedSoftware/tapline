//! A logged-on CM session.
//!
//! Owns the job-id counter, the session id Steam assigns, and the rule that
//! every received frame goes through [`crate::expand`] before anything looks at
//! it — because most replies arrive inside a batch alongside traffic nobody
//! asked for.
//!
//! # Anonymous logon
//!
//! The common case, and the one dedicated servers need: no credentials, no
//! stored token, no account. Steam hands back a `SteamId` of type `AnonUser`
//! and grants depot keys for anything anonymously accessible — which is every
//! dedicated-server depot. Nothing in this path touches a password.

use crate::{EMsg, Frame, NetError, expand};
use tapline_io::Transport;
use tapline_proto::steammessages_base::CMsgProtoBufHeader;
use tapline_proto::steammessages_clientserver_login::{
    CMsgClientHeartBeat, CMsgClientHello, CMsgClientLogon, CMsgClientLogonResponse,
};
use tapline_wire::{Message, Rpc};

/// Steam's own result code for success.
const RESULT_OK: i32 = 1;

/// The protocol version the client claims.
///
/// Steam checks this and refuses a logon that claims something it does not
/// support. `65580` is what current clients send.
const PROTOCOL_VERSION: u32 = 65_580;

/// The account type bits for an anonymous user, in the position `CMsgClientLogon`
/// wants them.
///
/// An anonymous logon sends a SteamId that names no account: universe 1
/// (public), type 10 (`AnonUser`), instance 0, account id 0.
const ANONYMOUS_STEAMID: u64 = (1_u64 << 56) | (10_u64 << 52);

/// What Steam said when we logged on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogonOutcome {
    /// The `SteamId` Steam assigned. For an anonymous logon this is a fresh
    /// account number Steam picked.
    pub steam_id: u64,
    /// The session id, which every later message must carry.
    pub session_id: i32,
    /// How often to heartbeat, in seconds. Miss these and Steam drops the
    /// session without explanation.
    pub heartbeat_seconds: u32,
    /// The cell id Steam thinks we are in, which decides which CDN hosts are
    /// near us.
    pub cell_id: u32,
}

/// A CM session over some transport.
#[derive(Debug)]
pub struct Session<T> {
    transport: T,
    session_id: i32,
    steam_id: u64,
    next_job_id: u64,
    /// Frames that arrived while we were waiting for a different reply.
    ///
    /// Steam batches unrelated traffic into the same `Multi` as the answer you
    /// asked for. Discarding the rest would throw away license lists and
    /// disconnect notices, so they queue here for the caller to drain.
    pending: Vec<Frame>,
}

impl<T: Transport> Session<T> {
    /// Wraps a connected transport. No traffic has happened yet.
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            session_id: 0,
            steam_id: 0,
            next_job_id: 1,
            pending: Vec::new(),
        }
    }

    /// The session id Steam assigned, or zero before logon.
    #[must_use]
    pub const fn session_id(&self) -> i32 {
        self.session_id
    }

    /// The `SteamId` Steam assigned, or zero before logon.
    #[must_use]
    pub const fn steam_id(&self) -> u64 {
        self.steam_id
    }

    /// Frames that arrived unsolicited, oldest first.
    ///
    /// Draining this is how a caller sees the license list, which Steam pushes
    /// after logon without being asked.
    pub fn take_unsolicited(&mut self) -> Vec<Frame> {
        std::mem::take(&mut self.pending)
    }

    /// A fresh job id.
    ///
    /// Public because callers outside this crate — PICS, the CDN directory —
    /// build their own request frames and need an id the session will recognise
    /// when the reply comes back.
    pub fn next_job_id(&mut self) -> u64 {
        let id = self.next_job_id;
        // Wrapping keeps this total. u64::MAX is the "no job" sentinel and must
        // never be handed out as a real id.
        self.next_job_id = self.next_job_id.wrapping_add(1);
        if self.next_job_id == u64::MAX {
            self.next_job_id = 1;
        }
        id
    }

    /// A header carrying the current session, and `job_id` if given.
    fn header(&self, job_id: Option<u64>) -> CMsgProtoBufHeader {
        CMsgProtoBufHeader {
            client_sessionid: Some(self.session_id),
            steamid: Some(self.steam_id),
            jobid_source: job_id,
            ..CMsgProtoBufHeader::default()
        }
    }

    /// Sends a frame.
    pub async fn send(&mut self, frame: &Frame) -> Result<(), NetError> {
        self.transport.send(&frame.encode()).await?;
        Ok(())
    }

    /// Receives the next frame, expanding batches.
    ///
    /// Returns frames one at a time; anything else in the same batch is queued
    /// for [`Session::take_unsolicited`].
    pub async fn recv(&mut self) -> Result<Frame, NetError> {
        loop {
            if !self.pending.is_empty() {
                // `pending` is non-empty, so removing the first element cannot
                // fail; the fallback keeps the no-panic rule.
                if self.pending.is_empty() {
                    continue;
                }
                return Ok(self.pending.remove(0));
            }
            let bytes = self.transport.recv().await?;
            let frames = expand(Frame::decode(&bytes)?)?;
            if frames.is_empty() {
                // An empty batch is legal and means nothing arrived worth
                // reporting; wait for the next one rather than returning a
                // frame we do not have.
                continue;
            }
            self.pending = frames;
        }
    }

    /// Waits for the reply to `job_id`, queueing anything else that arrives.
    ///
    /// `ClientLoggedOff` short-circuits: once Steam has ended the session, no
    /// reply is ever coming and waiting for one would hang until the transport
    /// timed out.
    pub async fn wait_for_job(&mut self, job_id: u64) -> Result<Frame, NetError> {
        let mut unrelated = Vec::new();
        loop {
            let frame = self.recv().await?;

            if frame.reply_to() == Some(job_id) {
                self.pending.splice(0..0, unrelated);
                return Ok(frame);
            }
            if frame.emsg == EMsg::CLIENT_LOGGED_OFF {
                let eresult = frame.header.eresult.unwrap_or(0);
                return Err(NetError::Steam { eresult });
            }
            unrelated.push(frame);
        }
    }

    /// Sends `ClientHello`, which a WebSocket session must do before logging on.
    pub async fn hello(&mut self) -> Result<(), NetError> {
        let hello = CMsgClientHello {
            protocol_version: Some(PROTOCOL_VERSION),
        };
        let frame = Frame::new(EMsg::CLIENT_HELLO, self.header(None), hello.encode_to_vec());
        self.send(&frame).await
    }

    /// Logs on anonymously.
    ///
    /// This is what a dedicated-server install uses. Steam grants depot keys for
    /// anonymously accessible content, which covers every dedicated server, and
    /// no credential is involved at any point.
    pub async fn logon_anonymous(&mut self, cell_id: u32) -> Result<LogonOutcome, NetError> {
        self.hello().await?;

        self.steam_id = ANONYMOUS_STEAMID;
        let job_id = self.next_job_id();

        let logon = CMsgClientLogon {
            protocol_version: Some(PROTOCOL_VERSION),
            cell_id: Some(cell_id),
            client_os_type: Some(OS_LINUX),
            should_remember_password: Some(false),
            ..CMsgClientLogon::default()
        };

        let mut header = self.header(Some(job_id));
        header.steamid = Some(ANONYMOUS_STEAMID);
        let frame = Frame::new(EMsg::CLIENT_LOGON, header, logon.encode_to_vec());
        self.send(&frame).await?;

        let reply = self.wait_for_logon_response().await?;
        let response: CMsgClientLogonResponse = reply.decode_body()?;

        let eresult = response.eresult.unwrap_or(0);
        if eresult != RESULT_OK {
            return Err(NetError::Steam { eresult });
        }

        self.session_id = reply.header.client_sessionid.unwrap_or(0);
        self.steam_id = reply.header.steamid.unwrap_or(ANONYMOUS_STEAMID);

        Ok(LogonOutcome {
            steam_id: self.steam_id,
            session_id: self.session_id,
            heartbeat_seconds: response.heartbeat_seconds.unwrap_or(9).max(1) as u32,
            cell_id: response.cell_id.unwrap_or(cell_id),
        })
    }

    /// Waits for the logon response.
    ///
    /// Matched by message type rather than by job id: Steam's logon response
    /// does not always carry `jobid_target`, and a session that waited for one
    /// would hang on a successful logon.
    async fn wait_for_logon_response(&mut self) -> Result<Frame, NetError> {
        let mut unrelated = Vec::new();
        loop {
            let frame = self.recv().await?;
            match frame.emsg {
                EMsg::CLIENT_LOGON_RESPONSE => {
                    self.pending.splice(0..0, unrelated);
                    return Ok(frame);
                }
                EMsg::CLIENT_LOGGED_OFF => {
                    return Err(NetError::Steam {
                        eresult: frame.header.eresult.unwrap_or(0),
                    });
                }
                _ => unrelated.push(frame),
            }
        }
    }

    /// Sends one heartbeat.
    ///
    /// Steam drops a session that stops heartbeating, without saying why, so
    /// this is not optional for a session that stays open across a download.
    pub async fn heartbeat(&mut self) -> Result<(), NetError> {
        let frame = Frame::new(
            EMsg::CLIENT_HEARTBEAT,
            self.header(None),
            CMsgClientHeartBeat::default().encode_to_vec(),
        );
        self.send(&frame).await
    }

    /// Makes a unified-message call and decodes its reply.
    ///
    /// The request type names its own target and response type through
    /// [`Rpc`], so a caller cannot pair a request with another method's reply.
    pub async fn call<R: Rpc>(&mut self, request: &R) -> Result<R::Response, NetError> {
        let job_id = self.next_job_id();

        let mut header = self.header(Some(job_id));
        // Steam's unified messages are versioned; `#1` is what every current
        // method uses.
        header.target_job_name = Some(format!("{}#1", R::TARGET));

        // Before logon there is no session to authenticate the call, and Steam
        // rejects the authed variant outright.
        let emsg = if self.session_id == 0 {
            EMsg::SERVICE_METHOD_CALL_NON_AUTHED
        } else {
            EMsg::SERVICE_METHOD_CALL
        };

        let frame = Frame::new(emsg, header, request.encode_to_vec());
        self.send(&frame).await?;

        let reply = self.wait_for_job(job_id).await?;

        // A unified-message failure arrives as an eresult on the header with an
        // empty body, which would otherwise decode to a response of all
        // defaults and look like success.
        match reply.header.eresult {
            Some(result) if result != RESULT_OK => {
                return Err(NetError::Steam { eresult: result });
            }
            _ => {}
        }

        reply.decode_body()
    }

    /// Closes the connection.
    pub async fn close(&mut self) -> Result<(), NetError> {
        self.transport.close().await?;
        Ok(())
    }
}

/// `EOSType` for Linux, which is what `client_os_type` wants.
const OS_LINUX: u32 = 16;

#[cfg(test)]
mod tests {
    use super::*;
    use tapline_io::testing::{MemoryTransport, block_on};
    use tapline_proto::steammessages_base::CMsgMulti;

    /// Encodes a frame the way Steam would send it.
    fn reply(emsg: EMsg, header: CMsgProtoBufHeader, body: Vec<u8>) -> Vec<u8> {
        Frame::new(emsg, header, body).encode()
    }

    fn logon_response(eresult: i32, session_id: i32, steam_id: u64) -> Vec<u8> {
        let body = CMsgClientLogonResponse {
            eresult: Some(eresult),
            heartbeat_seconds: Some(9),
            cell_id: Some(63),
            ..CMsgClientLogonResponse::default()
        };
        reply(
            EMsg::CLIENT_LOGON_RESPONSE,
            CMsgProtoBufHeader {
                client_sessionid: Some(session_id),
                steamid: Some(steam_id),
                ..CMsgProtoBufHeader::default()
            },
            body.encode_to_vec(),
        )
    }

    #[test]
    fn an_anonymous_logon_sends_hello_then_logon_and_reads_the_session_back() {
        let transport = MemoryTransport::new(vec![logon_response(1, 4242, 0x0A00_0000_0000_007B)]);
        let mut session = Session::new(transport);

        let outcome = block_on(session.logon_anonymous(0)).expect("logon must succeed");

        assert_eq!(outcome.session_id, 4242);
        assert_eq!(outcome.steam_id, 0x0A00_0000_0000_007B);
        assert_eq!(outcome.heartbeat_seconds, 9);
        assert_eq!(outcome.cell_id, 63);

        // Hello must come first on a WebSocket session, then the logon.
        let sent = session.transport.sent();
        assert_eq!(sent.len(), 2, "expected ClientHello then ClientLogon");

        let hello = Frame::decode(sent.first().expect("hello")).expect("must decode");
        assert_eq!(hello.emsg, EMsg::CLIENT_HELLO);

        let logon = Frame::decode(sent.get(1).expect("logon")).expect("must decode");
        assert_eq!(logon.emsg, EMsg::CLIENT_LOGON);
        // The anonymous SteamId names no account: universe 1, type 10.
        assert_eq!(logon.header.steamid, Some(ANONYMOUS_STEAMID));
        let body: CMsgClientLogon = logon.decode_body().expect("must decode");
        assert_eq!(body.protocol_version, Some(PROTOCOL_VERSION));
    }

    #[test]
    fn a_refused_logon_reports_steams_own_result_code() {
        // The difference between rate-limited and invalid-password is the
        // difference between waiting and giving up, so the code is preserved.
        let transport = MemoryTransport::new(vec![logon_response(84, 0, 0)]);
        let mut session = Session::new(transport);

        assert_eq!(
            block_on(session.logon_anonymous(0)),
            Err(NetError::Steam { eresult: 84 })
        );
    }

    #[test]
    fn a_logged_off_message_ends_the_wait_instead_of_hanging() {
        // Once Steam has ended the session no reply is coming, and waiting for
        // one would hang until the transport timed out.
        let transport = MemoryTransport::new(vec![reply(
            EMsg::CLIENT_LOGGED_OFF,
            CMsgProtoBufHeader {
                eresult: Some(6),
                ..CMsgProtoBufHeader::default()
            },
            Vec::new(),
        )]);
        let mut session = Session::new(transport);

        assert_eq!(
            block_on(session.logon_anonymous(0)),
            Err(NetError::Steam { eresult: 6 })
        );
    }

    #[test]
    fn the_logon_response_is_found_inside_a_batch_alongside_other_traffic() {
        // This is what actually arrives: the answer wrapped in a Multi with
        // unrelated messages Steam decided to send at the same time.
        let mut payload = Vec::new();
        for message in [
            reply(
                EMsg::CLIENT_LICENSE_LIST,
                CMsgProtoBufHeader::default(),
                vec![],
            ),
            logon_response(1, 77, 0x0A00_0000_0000_0001),
        ] {
            payload.extend_from_slice(&(message.len() as u32).to_le_bytes());
            payload.extend_from_slice(&message);
        }

        let multi = CMsgMulti {
            size_unzipped: None,
            message_body: Some(payload),
        };
        let batch = reply(
            EMsg::MULTI,
            CMsgProtoBufHeader::default(),
            multi.encode_to_vec(),
        );

        let mut session = Session::new(MemoryTransport::new(vec![batch]));
        let outcome = block_on(session.logon_anonymous(0)).expect("logon must succeed");
        assert_eq!(outcome.session_id, 77);

        // The license list must not have been thrown away just because it
        // shared a batch with the answer.
        let unsolicited = session.take_unsolicited();
        assert_eq!(unsolicited.len(), 1);
        assert_eq!(
            unsolicited.first().map(|f| f.emsg),
            Some(EMsg::CLIENT_LICENSE_LIST)
        );
    }

    #[test]
    fn job_ids_are_never_the_no_job_sentinel() {
        let mut session = Session::new(MemoryTransport::new(Vec::new()));
        session.next_job_id = u64::MAX - 1;

        for _ in 0..4 {
            let id = session.next_job_id();
            assert_ne!(id, crate::NO_JOB, "handed out the no-job sentinel");
            assert_ne!(id, 0);
        }
    }

    #[test]
    fn a_disconnect_is_reported_as_one() {
        let mut session = Session::new(MemoryTransport::new(Vec::new()));
        assert_eq!(
            block_on(session.logon_anonymous(0)),
            Err(NetError::Disconnected)
        );
    }

    #[test]
    fn a_heartbeat_carries_the_session_id() {
        let transport = MemoryTransport::new(vec![logon_response(1, 555, 1)]);
        let mut session = Session::new(transport);

        block_on(async {
            session.logon_anonymous(0).await.expect("logon");
            session.heartbeat().await.expect("heartbeat");
        });

        let sent = session.transport.sent();
        let beat = Frame::decode(sent.last().expect("a heartbeat")).expect("must decode");
        assert_eq!(beat.emsg, EMsg::CLIENT_HEARTBEAT);
        assert_eq!(beat.header.client_sessionid, Some(555));
    }
}
