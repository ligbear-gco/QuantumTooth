//! @file
//! @brief One on-demand session owner with restricted bounded IPC and cancellation.
use crate::{ownership::ProbeLock, probe::secure::Link};
use qtooth_ble_core::{
    framing::Kind,
    protocol::{Pair, Session},
};
use std::{
    io,
    os::unix::fs::{DirBuilderExt, FileTypeExt, MetadataExt, PermissionsExt},
    path::PathBuf,
    process::Stdio,
    sync::Arc,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{UnixListener, UnixStream},
    time::{Duration, Instant, sleep_until, timeout, timeout_at},
};
/// @brief Generated input is empty for an unprovisioned build, never a default/test pair.
const PAIR: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/pair.bin"));
/// @brief Safe phase/reason strings never contain backend identifiers or plaintext.
pub type Result<T> = std::result::Result<T, &'static str>;
/// @brief Restrict each IPC request/write, startup and cleanup wait.
const LOCAL: Duration = Duration::from_secs(5);
/// @brief Versioned command opcodes on local IPC; no simulator opcode exists.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Send = 1,
    Keep = 2,
    Close = 3,
}
/// @brief Result codes expose only facts the local process can establish.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
#[repr(u8)]
pub enum Outcome {
    Transport = 0,
    Closed = 1,
    Busy = 2,
    Failed = 3,
    EstablishmentFailed = 4,
    AuthenticationFailed = 5,
    TransferFailed = 6,
    CleanupFailed = 7,
}
impl Outcome {
    /// @brief Stable safe CLI output, distinguishing transport completion from delivery.
    pub fn report(self) -> &'static str {
        match self {
            Self::Transport => "phase=complete transport_success=true delivery=unconfirmed",
            Self::Closed => "phase=close local_closed=true remote_close=unconfirmed",
            Self::Busy => "phase=ownership error=busy",
            Self::Failed => "phase=operation error=failed delivery=unconfirmed",
            Self::EstablishmentFailed => "phase=establishment error=failed delivery=unconfirmed",
            Self::AuthenticationFailed => "phase=authentication error=failed delivery=unconfirmed",
            Self::TransferFailed => "phase=transfer error=failed_or_expired delivery=unconfirmed",
            Self::CleanupFailed => "phase=cleanup error=failed delivery=unconfirmed",
        }
    }
    /// @brief Only locally completed data/close operations are successful.
    pub fn succeeded(self) -> bool {
        matches!(self, Self::Transport | Self::Closed)
    }
    /// @brief Preserve the failing phase through bounded IPC without returning backend strings.
    fn failure(reason: &str) -> Self {
        if reason.starts_with("cleanup") {
            Self::CleanupFailed
        } else if reason.contains("authentication")
            || reason.contains("confirmation")
            || reason.contains("record_encryption")
        {
            Self::AuthenticationFailed
        } else if reason.contains("setup")
            || reason.contains("establishment")
            || reason.contains("dbus")
            || reason.contains("adapter")
            || reason.contains("controller")
            || reason.contains("advertising")
            || reason.contains("registration")
            || reason.contains("observer")
        {
            Self::EstablishmentFailed
        } else {
            Self::TransferFailed
        }
    }
}
/// @brief Runtime paths are fixed in production; tests supply isolated temporary paths.
pub struct Paths {
    pub directory: PathBuf,
    pub lock: PathBuf,
    pub uid: u32,
}
impl Paths {
    /// @brief Root-only policy matches the diagnostic shell and existing probe lock.
    pub fn production() -> Self {
        Self {
            directory: PathBuf::from("/run/qtooth-ble"),
            lock: PathBuf::from("/run/qtooth-ble-probe.lock"),
            uid: 0,
        }
    }
    /// @brief Socket names never depend on untrusted IPC input.
    fn socket(&self) -> PathBuf {
        self.directory.join("worker.sock")
    }
    /// @brief Create/validate the complete runtime directory without following symlinks.
    fn prepare(&self) -> Result<()> {
        match std::fs::DirBuilder::new()
            .mode(0o700)
            .create(&self.directory)
        {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(_) => return Err("runtime_directory_unavailable"),
        }
        let m = std::fs::symlink_metadata(&self.directory)
            .map_err(|_| "runtime_directory_unavailable")?;
        if !m.is_dir() || m.uid() != self.uid || m.permissions().mode() & 0o777 != 0o700 {
            return Err("unsafe_runtime_directory");
        };
        Ok(())
    }
    /// @brief Only the live lock owner calls this stale-socket removal function.
    fn remove_socket(&self) -> Result<()> {
        match std::fs::symlink_metadata(self.socket()) {
            Ok(m)
                if m.file_type().is_socket()
                    && m.uid() == self.uid
                    && m.permissions().mode() & 0o777 == 0o600 =>
            {
                std::fs::remove_file(self.socket()).map_err(|_| "socket_remove_failed")
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
            _ => Err("unsafe_socket"),
        }
    }
}
/// @brief Validate public CLI data before any runtime, socket, lock or Bluetooth side effects.
pub fn validate(op: Op, text: &str) -> Result<()> {
    if (op == Op::Close && text.is_empty()) || (op != Op::Close && (1..=512).contains(&text.len()))
    {
        Ok(())
    } else {
        Err("text_requires_1_to_512_utf8_bytes")
    }
}
/// @brief Reject data commands without provisioning before launching or advertising.
pub fn provisioned() -> bool {
    PAIR.len() == 64
}
/// @brief Actual transport boundary used by both production worker and native test peers.
#[allow(async_fn_in_trait)]
pub trait Backend {
    /// @brief Establish a newly authenticated session; no reconnect inside an operation.
    async fn establish(&mut self, pair: Arc<Pair>) -> Result<Session>;
    /// @brief Send one already-encrypted record once, within its enclosing deadline.
    async fn send(&mut self, record: &[u8], until: Instant) -> Result<()>;
    /// @brief Observe idle link/subscription/protocol failure without extending lifetime.
    async fn idle(&mut self, session: &mut Session) -> Result<()>;
    /// @brief Release owned resources; preserve shared BlueZ/D-Bus and unrelated peers.
    async fn cleanup(&mut self) -> Result<()>;
}
/// @brief Production transport has no native-simulator or test provisioning fallback.
pub struct Ble {
    link: Option<Link>,
}
impl Ble {
    /// @brief Start with no Bluetooth resources.
    pub fn new() -> Self {
        Self { link: None }
    }
}
impl Backend for Ble {
    async fn establish(&mut self, pair: Arc<Pair>) -> Result<Session> {
        self.link = Some(
            timeout(LOCAL, Link::open())
                .await
                .map_err(|_| "setup_timeout")??,
        );
        let link = self.link.as_mut().unwrap();
        let d = link.phase_deadline();
        timeout_at(d, link.establish(pair))
            .await
            .map_err(|_| "establishment_timeout")?
    }
    async fn send(&mut self, r: &[u8], until: Instant) -> Result<()> {
        let link = self.link.as_mut().ok_or("no_link")?;
        link.deadline(until);
        link.send_object(Kind::Record, r, until).await
    }
    async fn idle(&mut self, s: &mut Session) -> Result<()> {
        self.link.as_mut().ok_or("no_link")?.idle(s).await
    }
    async fn cleanup(&mut self) -> Result<()> {
        if let Some(mut link) = self.link.take() {
            link.cleanup().await
        } else {
            Ok(())
        }
    }
}
/// @brief Kept keys and absolute expiry exist only inside the single worker process.
struct Kept {
    session: Session,
    until: Instant,
}
/// @brief Decode one bounded/versioned request; reject trailing data during the operation.
async fn request(s: &mut UnixStream) -> Result<(Op, Vec<u8>)> {
    timeout(LOCAL, async {
        let mut h = [0; 4];
        s.read_exact(&mut h)
            .await
            .map_err(|_| "ipc_header_failed")?;
        let op = match (h[0], h[1]) {
            (1, 1) => Op::Send,
            (1, 2) => Op::Keep,
            (1, 3) => Op::Close,
            _ => return Err("ipc_version_or_command"),
        };
        let n = u16::from_be_bytes([h[2], h[3]]) as usize;
        if (op == Op::Close && n != 0) || (op != Op::Close && !(1..=512).contains(&n)) {
            return Err("ipc_length");
        }
        let mut b = vec![0; n];
        s.read_exact(&mut b).await.map_err(|_| "ipc_body_failed")?;
        std::str::from_utf8(&b).map_err(|_| "ipc_utf8")?;
        Ok((op, b))
    })
    .await
    .map_err(|_| "ipc_request_timeout")?
}
/// @brief Complete the local result handoff; this local IPC commit is unrelated to BLE application ACKs.
async fn result(s: &mut UnixStream, o: Outcome) -> Result<()> {
    timeout(LOCAL, async {
        s.write_all(&[1, o as u8])
            .await
            .map_err(|_| "ipc_result_failed")?;
        let mut ack = [0];
        s.read_exact(&mut ack)
            .await
            .map_err(|_| "ipc_result_uncommitted")?;
        if ack != [1] {
            return Err("ipc_commit_invalid");
        }
        s.write_all(&[1]).await.map_err(|_| "ipc_commit_failed")?;
        Ok(())
    })
    .await
    .map_err(|_| "ipc_result_timeout")?
}
/// @brief Execute exactly one command; callers cancel and destroy state on IPC or signal loss.
async fn operation<B: Backend>(
    b: &mut B,
    kept: &mut Option<Kept>,
    pair: Arc<Pair>,
    op: Op,
    text: &[u8],
) -> Result<Outcome> {
    if op == Op::Send && kept.is_some() {
        return Ok(Outcome::Busy);
    }
    if op == Op::Close {
        if let Some(mut k) = kept.take()
            && Instant::now() < k.until
            && let Ok(r) = k.session.encrypt(&[], true)
        {
            let _ = timeout_at(
                k.until.min(Instant::now() + Duration::from_secs(2)),
                b.send(&r, k.until),
            )
            .await;
        }
        timeout(Duration::from_secs(2), b.cleanup())
            .await
            .map_err(|_| "cleanup_timeout")??;
        return Ok(Outcome::Closed);
    }
    if kept.is_none() {
        let session = b.establish(pair).await?;
        *kept = Some(Kept {
            session,
            until: Instant::now() + Duration::from_secs(120),
        });
    }
    let k = kept.as_mut().unwrap();
    if Instant::now() >= k.until {
        return Err("session_expired");
    }
    let r = k
        .session
        .encrypt(text, false)
        .map_err(|_| "record_encryption_failed")?;
    timeout_at(k.until, b.send(&r, k.until))
        .await
        .map_err(|_| "session_expired")??;
    if op == Op::Send {
        kept.take();
        timeout(Duration::from_secs(2), b.cleanup())
            .await
            .map_err(|_| "cleanup_timeout")??;
    }
    Ok(Outcome::Transport)
}
/// @brief Verify kernel-authenticated local peer UID before reading any client frame.
fn authorized(s: &UnixStream, uid: u32) -> bool {
    s.peer_cred().is_ok_and(|c| c.uid() == uid)
}
/// @brief Reject competing local operations without queuing/interleaving them.
fn busy(s: &mut UnixStream, uid: u32) {
    if authorized(s, uid) {
        let _ = s.try_write(&[1, Outcome::Busy as u8]);
    }
}
/// @brief Run the single-owner worker; every exit drops keys before bounded resource teardown.
pub async fn serve<B: Backend>(
    paths: Paths,
    pair: Arc<Pair>,
    mut backend: B,
    stop: impl std::future::Future<Output = ()>,
    _owner: &ProbeLock,
) -> Result<()> {
    paths.prepare()?;
    paths.remove_socket()?;
    let listener = UnixListener::bind(paths.socket()).map_err(|_| "socket_bind_failed")?;
    std::fs::set_permissions(paths.socket(), std::fs::Permissions::from_mode(0o600))
        .map_err(|_| "socket_mode_failed")?;
    let mut kept: Option<Kept> = None;
    tokio::pin!(stop);
    let work=async{loop{
        let mut client=if let Some(k)=kept.as_mut(){tokio::select!{biased;
            _=&mut stop=>return Err("cancelled"),_=sleep_until(k.until)=>return Err("session_expired"),
            _=backend.idle(&mut k.session)=>return Err("idle_link_or_protocol_failure"),
            s=listener.accept()=>s.map_err(|_|"ipc_accept_failed")?.0,
        }}else{tokio::select!{biased;_=&mut stop=>return Err("cancelled"),s=timeout(LOCAL,listener.accept())=>s.map_err(|_|"startup_idle_timeout")?.map_err(|_|"ipc_accept_failed")?.0}};
        if !authorized(&client,paths.uid){continue;}
        let admitted_until=kept.as_ref().map(|k|k.until).unwrap_or(Instant::now()+Duration::from_secs(47));
        let admitted=async{
            let(op,text)=request(&mut client).await?;
            let mut loss=[0];
            let op_result=tokio::select!{biased;_=client.read(&mut loss)=>Err("ipc_lost_or_trailing_data"),r=operation(&mut backend,&mut kept,pair.clone(),op,&text)=>r};
            let outcome=match op_result{Ok(o)=>o,Err(e)=>{eprintln!("phase=operation error={e}");kept.take();let _=client.try_write(&[1,Outcome::failure(e) as u8]);return Err(e)}};
            if outcome==Outcome::Busy{let _=timeout(LOCAL,client.write_all(&[1,Outcome::Busy as u8])).await;return Ok(true)}
            result(&mut client,outcome).await?;
            Ok(op==Op::Keep)
        };
        // @note An admitted request already owns the operation slot while framing is read.
        let keep={tokio::pin!(admitted);loop{tokio::select!{biased;
            _=&mut stop=>break Err("cancelled"),_=sleep_until(admitted_until)=>break Err("session_or_operation_expired"),c=listener.accept()=>{if let Ok((mut s,_))=c{busy(&mut s,paths.uid);}},r=&mut admitted=>break r,
        }}}?;
        if !keep{return Ok(())}
    }}.await;
    kept.take();
    let cleanup = timeout(Duration::from_secs(2), backend.cleanup())
        .await
        .map_err(|_| "cleanup_timeout")
        .and_then(|r| r);
    drop(listener);
    let removed = paths.remove_socket();
    work.and(cleanup).and(removed)
}
/// @brief Connect/start a worker within five seconds; no secrets/plaintext appear in worker arguments.
pub async fn client(op: Op, text: &str) -> Result<Outcome> {
    validate(op, text)?;
    if op != Op::Close && !provisioned() {
        return Err("pair_not_provisioned");
    }
    client_at(Paths::production(), op, text, || {
        tokio::process::Command::new(std::env::current_exe().map_err(|_| "executable_unavailable")?)
            .arg("__worker")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| "worker_launch_failed")
    })
    .await
}
/// @brief Shared startup/IPC implementation; tests inject only paths and process launch, never production keys.
async fn client_at(
    paths: Paths,
    op: Op,
    text: &str,
    mut launch: impl FnMut() -> Result<tokio::process::Child>,
) -> Result<Outcome> {
    validate(op, text)?;
    if op == Op::Close && !paths.directory.exists() {
        return Ok(Outcome::Closed);
    }
    paths.prepare()?;
    let until = Instant::now() + LOCAL;
    let mut child = None;
    let mut socket = loop {
        if Instant::now() >= until {
            return Err("busy_or_worker_startup_failed");
        }
        match timeout_at(until, UnixStream::connect(paths.socket()))
            .await
            .map_err(|_| "busy_or_worker_startup_failed")?
        {
            Ok(s) => break s,
            Err(_) => {
                if op == Op::Close {
                    if let Ok(_owner) = ProbeLock::acquire_as(&paths.lock, paths.uid) {
                        paths.remove_socket()?;
                        return Ok(Outcome::Closed);
                    }
                    return Ok(Outcome::Busy);
                }
                if child.is_none() {
                    child = Some(launch()?);
                }
                if Instant::now() >= until {
                    return Err("busy_or_worker_startup_failed");
                }
                sleep_until(until.min(Instant::now() + Duration::from_millis(20))).await;
            }
        }
    };
    if !authorized(&socket, paths.uid) {
        return Err("worker_credentials_invalid");
    }
    let out = exchange(&mut socket, op, text).await;
    // @note Successful kept workers intentionally outlive this CLI; reap any already-finished launch child.
    if let Some(mut c) = child {
        let _ = c.try_wait();
    }
    out
}
/// @brief Client half of versioned bounded IPC; used by isolated native process tests too.
pub async fn exchange(s: &mut UnixStream, op: Op, text: &str) -> Result<Outcome> {
    validate(op, text)?;
    let mut h = vec![1, op as u8];
    h.extend_from_slice(&(text.len() as u16).to_be_bytes());
    h.extend_from_slice(text.as_bytes());
    timeout(LOCAL, s.write_all(&h))
        .await
        .map_err(|_| "ipc_write_timeout")?
        .map_err(|_| "ipc_write_failed")?;
    let mut r = [0; 2];
    timeout(Duration::from_secs(47), s.read_exact(&mut r))
        .await
        .map_err(|_| "operation_timeout")?
        .map_err(|_| "worker_lost")?;
    let o = match r {
        [1, 0] => Outcome::Transport,
        [1, 1] => Outcome::Closed,
        [1, 2] => return Ok(Outcome::Busy),
        [1, 3] => return Ok(Outcome::Failed),
        [1, 4] => return Ok(Outcome::EstablishmentFailed),
        [1, 5] => return Ok(Outcome::AuthenticationFailed),
        [1, 6] => return Ok(Outcome::TransferFailed),
        [1, 7] => return Ok(Outcome::CleanupFailed),
        _ => return Err("ipc_result_invalid"),
    };
    timeout(LOCAL, async {
        s.write_all(&[1]).await.map_err(|_| "ipc_commit_failed")?;
        let mut done = [0];
        s.read_exact(&mut done)
            .await
            .map_err(|_| "ipc_commit_failed")?;
        if done != [1] {
            return Err("ipc_commit_invalid");
        };
        Ok(())
    })
    .await
    .map_err(|_| "ipc_commit_timeout")??;
    Ok(o)
}
/// @brief Build pair ownership from ignored compile-time input only.
pub fn pair() -> Result<Arc<Pair>> {
    Pair::from_bytes(PAIR)
        .map(Arc::new)
        .map_err(|_| "pair_not_provisioned")
}

#[cfg(test)]
mod tests;

/// @brief Register process cancellation before runtime resources; shared by production and native harness.
pub fn cancellation() -> Result<impl std::future::Future<Output = ()>> {
    let mut interrupt = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt())
        .map_err(|_| "signal_setup_failed")?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|_| "signal_setup_failed")?;
    Ok(async move {
        tokio::select! {_=interrupt.recv()=>{},_=terminate.recv()=>{}}
    })
}
