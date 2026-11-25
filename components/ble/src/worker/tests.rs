//! @file
//! @brief Real-crypto process harness and isolated IPC/ownership/lifetime tests.
use super::*;
use qtooth_ble_core::{
    framing::{self, Reassembly},
    protocol::Handshake,
};
use std::{
    io::Read,
    process::{Child, Command},
    sync::atomic::{AtomicU64, Ordering},
};
use zeroize::Zeroizing;
/// @brief Isolated private test directory; cleanup also runs after assertion failures.
struct Scratch(PathBuf);
impl Scratch {
    /// @brief Allocate a fresh test directory without operator provisioning inputs.
    fn new() -> Self {
        static ID: AtomicU64 = AtomicU64::new(0);
        let p = std::env::temp_dir().join(format!(
            "qtooth-g2-2-{}-{}",
            std::process::id(),
            ID.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::DirBuilder::new().mode(0o700).create(&p).unwrap();
        Self(p)
    }
    /// @brief Explicitly scoped paths keep tests away from production /run ownership.
    fn paths(&self) -> Paths {
        Paths {
            directory: self.0.join("run"),
            lock: self.0.join("owner.lock"),
            uid: std::fs::metadata("/proc/self").unwrap().uid(),
        }
    }
    /// @brief Generate disposable random PSK and identities into a private temporary file.
    fn pair(&self) -> Arc<Pair> {
        let mut b = Zeroizing::new([0; 64]);
        std::fs::File::open("/dev/urandom")
            .unwrap()
            .read_exact(&mut b[..])
            .unwrap();
        std::fs::write(self.0.join("pair.bin"), &b[..]).unwrap();
        std::fs::set_permissions(
            self.0.join("pair.bin"),
            std::fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        Arc::new(Pair::from_bytes(&b[..]).unwrap())
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
/// @brief Test-only peer transport; actual worker operations still own real sessions and deadlines.
struct Native {
    peer: Option<Session>,
    mtu: u16,
    log: Option<PathBuf>,
    delay: Duration,
    fail_fragment: Option<usize>,
}
impl Native {
    /// @brief Create a peer without a production command/transport selector.
    fn new(mtu: u16, log: Option<PathBuf>) -> Self {
        Self {
            peer: None,
            mtu,
            log,
            delay: Duration::ZERO,
            fail_fragment: None,
        }
    }
    /// @brief Record only safe public experiment outputs/counters.
    fn record(&self, s: &str) {
        if let Some(p) = &self.log {
            use std::io::Write;
            writeln!(
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(p)
                    .unwrap(),
                "{s}"
            )
            .unwrap();
        }
    }
}
/// @brief Exercise the same core framing as BLE at the chosen simulated MTU.
fn transfer(k: Kind, id: u16, body: &[u8], mtu: u16, rx: &mut Reassembly) -> Vec<u8> {
    let mut out = None;
    for v in framing::fragments(k, id, body, mtu).unwrap() {
        out = rx.push(&v, mtu, k, 0, 30_000).unwrap();
    }
    out.unwrap()
}
impl Backend for Native {
    async fn establish(&mut self, pair: Arc<Pair>) -> Result<Session> {
        let mut pi = Handshake::new(pair.clone(), true);
        let mut peer = Handshake::new(pair, false);
        let (mut pr, mut rr) = (Reassembly::default(), Reassembly::default());
        let h = transfer(
            Kind::ClientHello,
            0,
            &peer.start().unwrap(),
            self.mtu,
            &mut pr,
        );
        let (k, o) = pi.receive(Kind::ClientHello, &h).unwrap().unwrap();
        let o = transfer(k, 0, &o, self.mtu, &mut rr);
        let (k, f) = peer.receive(k, &o).unwrap().unwrap();
        let f = transfer(k, 1, &f, self.mtu, &mut pr);
        let (k, f) = pi.receive(k, &f).unwrap().unwrap();
        let f = transfer(k, 1, &f, self.mtu, &mut rr);
        peer.receive(k, &f).unwrap();
        pi.finish_sent().unwrap();
        self.peer = Some(peer.into_session().unwrap());
        self.record("established");
        Ok(pi.into_session().unwrap())
    }
    async fn send(&mut self, r: &[u8], until: Instant) -> Result<()> {
        timeout_at(until, tokio::time::sleep(self.delay))
            .await
            .map_err(|_| "send_expired")?;
        let mut rx = Reassembly::default();
        let mut body = None;
        for (index, v) in framing::fragments(Kind::Record, 0, r, self.mtu)
            .unwrap()
            .into_iter()
            .enumerate()
        {
            if self.fail_fragment == Some(index) {
                self.record("partial_transfer_failed");
                return Err("partial_transfer_failed");
            }
            body = rx.push(&v, self.mtu, Kind::Record, 0, 30_000).unwrap();
        }
        let r = body.unwrap();
        let sequence = u64::from_be_bytes(r[19..27].try_into().unwrap());
        if let Some(p) = self
            .peer
            .as_mut()
            .ok_or("peer_missing")?
            .decrypt(&r)
            .map_err(|_| "peer_rejected")?
        {
            assert_eq!(p, b"Hello, world!");
            self.record(&format!("decrypted=Hello, world! sequence={sequence}"));
        } else {
            self.record("closed");
        }
        Ok(())
    }
    async fn idle(&mut self, _: &mut Session) -> Result<()> {
        std::future::pending().await
    }
    async fn cleanup(&mut self) -> Result<()> {
        self.peer = None;
        self.record("cleanup");
        Ok(())
    }
}
/// @brief Kill/reap a server child on every path, including failed assertions.
struct Server(Child);
impl Drop for Server {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
/// @brief Launch only the unit-test executable's harness entry, never a production simulator.
fn child(mode: &str, root: &std::path::Path, mtu: u16) -> Command {
    let mut c = Command::new(std::env::current_exe().unwrap());
    c.args([
        "--exact",
        "worker::tests::g2_2_process_harness_entry",
        "--nocapture",
    ])
    .env("QTOOTH_G2_TEST_MODE", mode)
    .env("QTOOTH_G2_TEST_ROOT", root)
    .env("QTOOTH_G2_TEST_MTU", mtu.to_string());
    c
}
/// @brief Child role entry; all environment-based test hooks are compiled only under cfg(test).
#[test]
fn g2_2_process_harness_entry() {
    let Ok(mode) = std::env::var("QTOOTH_G2_TEST_MODE") else {
        return;
    };
    let root = PathBuf::from(std::env::var_os("QTOOTH_G2_TEST_ROOT").unwrap());
    let mtu = std::env::var("QTOOTH_G2_TEST_MTU")
        .unwrap()
        .parse()
        .unwrap();
    let scratch = std::mem::ManuallyDrop::new(Scratch(root.clone()));
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            if mode.starts_with("server") {
                let bytes = Zeroizing::new(std::fs::read(root.join("pair.bin")).unwrap());
                let pair = Arc::new(Pair::from_bytes(&bytes).unwrap());
                let paths = scratch.paths();
                let owner = ProbeLock::acquire_as(&paths.lock, paths.uid).unwrap();
                let mut backend = Native::new(mtu, Some(root.join("public-results")));
                if mode == "server-disconnect" {
                    backend.delay = Duration::from_secs(2);
                }
                let stopped = cancellation().unwrap();
                let r = serve(paths, pair, backend, stopped, &owner).await;
                let expected = match mode.as_str() {
                    "server" => Ok(()),
                    "server-disconnect" => Err("ipc_lost_or_trailing_data"),
                    "server-signal" => Err("cancelled"),
                    _ => panic!("unknown server mode"),
                };
                assert_eq!(r, expected);
            } else {
                let op = match mode.as_str() {
                    "keep" => Op::Keep,
                    "send" => Op::Send,
                    "close" => Op::Close,
                    _ => panic!(),
                };
                let o = client_at(
                    scratch.paths(),
                    op,
                    if op == Op::Close { "" } else { "Hello, world!" },
                    || {
                        let mut c = tokio::process::Command::new(std::env::current_exe().unwrap());
                        c.args([
                            "--exact",
                            "worker::tests::g2_2_process_harness_entry",
                            "--nocapture",
                        ])
                        .env("QTOOTH_G2_TEST_MODE", "server")
                        .env("QTOOTH_G2_TEST_ROOT", &root)
                        .env("QTOOTH_G2_TEST_MTU", mtu.to_string())
                        .stdout(Stdio::null())
                        .stderr(Stdio::null())
                        .spawn()
                        .map_err(|_| "test_launch_failed")
                    },
                )
                .await
                .unwrap();
                println!("outcome={o:?} {}", o.report());
            }
        });
}
/// @brief Two separate retained clients reuse one real-crypto session; fresh send conflicts, close exits.
#[test]
fn g2_2_retained_process_commands() {
    for mtu in [23, 517] {
        let scratch = Scratch::new();
        scratch.pair();
        let mut server = Server(
            child("server", &scratch.0, mtu)
                .stdout(Stdio::null())
                .spawn()
                .unwrap(),
        );
        let end = std::time::Instant::now() + LOCAL;
        while !scratch.paths().socket().exists() {
            assert!(std::time::Instant::now() < end);
            assert!(server.0.try_wait().unwrap().is_none());
            std::thread::sleep(Duration::from_millis(10));
        }
        for (mode, expected) in [
            ("keep", "Transport"),
            ("keep", "Transport"),
            ("send", "Busy"),
            ("close", "Closed"),
        ] {
            let o = child(mode, &scratch.0, mtu).output().unwrap();
            assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
            assert!(String::from_utf8_lossy(&o.stdout).contains(&format!("outcome={expected}")));
        }
        assert!(server.0.wait().unwrap().success());
        assert!(!scratch.paths().socket().exists());
        let log = std::fs::read_to_string(scratch.0.join("public-results")).unwrap();
        assert_eq!(log.matches("established").count(), 1);
        assert!(log.contains("sequence=0") && log.contains("sequence=1") && log.contains("closed"));
        println!(
            "g2_2 simulated_mtu={mtu} two_clients=one_session sequences=0,1 fresh_send=busy close=local no_application_ack=true"
        );
    }
}
/// @brief Exercise runtime-path validation, lock races, peer credentials and malicious IPC lengths.
#[tokio::test]
async fn g2_2_ownership_and_ipc() {
    let scratch = Scratch::new();
    let paths = scratch.paths();
    paths.prepare().unwrap();
    let owner = ProbeLock::acquire_as(&paths.lock, paths.uid).unwrap();
    assert!(ProbeLock::acquire_as(&paths.lock, paths.uid).is_err());
    let l = UnixListener::bind(paths.socket()).unwrap();
    std::fs::set_permissions(paths.socket(), std::fs::Permissions::from_mode(0o600)).unwrap();
    let mut c = UnixStream::connect(paths.socket()).await.unwrap();
    let (mut s, _) = l.accept().await.unwrap();
    assert!(authorized(&s, paths.uid));
    assert!(!authorized(&s, paths.uid + 1));
    c.write_all(&[1, 2, 0xff, 0xff]).await.unwrap();
    assert_eq!(request(&mut s).await.err(), Some("ipc_length"));
    drop(l);
    drop(owner);
    paths.remove_socket().unwrap();
    std::os::unix::fs::symlink("missing", paths.socket()).unwrap();
    assert_eq!(paths.remove_socket(), Err("unsafe_socket"));
    std::fs::remove_file(paths.socket()).unwrap();
    std::fs::set_permissions(&paths.directory, std::fs::Permissions::from_mode(0o777)).unwrap();
    assert_eq!(paths.prepare(), Err("unsafe_runtime_directory"));
}
/// @brief Controlled clocks prove sends cannot refresh 120 seconds and expiry cancels a send.
#[tokio::test(start_paused = true)]
async fn g2_2_absolute_lifetime_and_close() {
    let scratch = Scratch::new();
    let pair = scratch.pair();
    let mut b = Native::new(23, None);
    let mut kept = None;
    assert_eq!(
        operation(&mut b, &mut kept, pair.clone(), Op::Keep, b"Hello, world!")
            .await
            .unwrap(),
        Outcome::Transport
    );
    let until = kept.as_ref().unwrap().until;
    tokio::time::advance(Duration::from_secs(119)).await;
    assert_eq!(
        operation(&mut b, &mut kept, pair.clone(), Op::Keep, b"Hello, world!")
            .await
            .unwrap(),
        Outcome::Transport
    );
    assert_eq!(kept.as_ref().unwrap().until, until);
    b.delay = Duration::from_secs(2);
    assert!(
        operation(&mut b, &mut kept, pair.clone(), Op::Keep, b"Hello, world!")
            .await
            .is_err()
    );
    assert!(Instant::now() >= until);
    kept.take();
    b.cleanup().await.unwrap();
    assert_eq!(
        operation(&mut b, &mut kept, pair.clone(), Op::Close, &[])
            .await
            .unwrap(),
        Outcome::Closed
    );
    assert_eq!(
        operation(&mut b, &mut kept, pair, Op::Close, &[])
            .await
            .unwrap(),
        Outcome::Closed
    );
}
/// @brief Invalid UTF-8 byte lengths are rejected before any runtime paths can be touched.
#[test]
fn g2_2_input_validation() {
    assert!(validate(Op::Keep, "").is_err());
    assert!(validate(Op::Send, &"x".repeat(513)).is_err());
    assert!(validate(Op::Send, &"é".repeat(257)).is_err());
    assert!(validate(Op::Send, &"é".repeat(256)).is_ok());
    assert!(validate(Op::Close, "").is_ok());
    assert!(validate(Op::Close, "unexpected").is_err());
}

/// @brief Interrupted IPC and SIGINT/SIGTERM tear down workers; later processes establish fresh keys.
#[test]
fn g2_2_process_cancellation() {
    for mode in ["server-disconnect", "server-signal"] {
        for signal in ["-INT", "-TERM"] {
            let scratch = Scratch::new();
            scratch.pair();
            let mut server = Server(
                child(mode, &scratch.0, 23)
                    .stdout(Stdio::null())
                    .spawn()
                    .unwrap(),
            );
            let end = std::time::Instant::now() + LOCAL;
            while !scratch.paths().socket().exists() {
                assert!(std::time::Instant::now() < end);
                std::thread::sleep(Duration::from_millis(10));
            }
            if mode == "server-disconnect" {
                let mut s =
                    std::os::unix::net::UnixStream::connect(scratch.paths().socket()).unwrap();
                use std::io::Write;
                let mut frame = vec![1, 2, 0, 13];
                frame.extend_from_slice(b"Hello, world!");
                s.write_all(&frame).unwrap();
                let end = std::time::Instant::now() + LOCAL;
                while !scratch.0.join("public-results").exists() {
                    assert!(std::time::Instant::now() < end);
                    std::thread::sleep(Duration::from_millis(5));
                }
                drop(s);
            } else {
                let o = child("keep", &scratch.0, 23).output().unwrap();
                assert!(o.status.success());
                Command::new("kill")
                    .args([signal, &server.0.id().to_string()])
                    .status()
                    .unwrap();
            }
            let end = std::time::Instant::now() + LOCAL;
            loop {
                if let Some(status) = server.0.try_wait().unwrap() {
                    assert!(status.success());
                    break;
                }
                assert!(std::time::Instant::now() < end);
                std::thread::sleep(Duration::from_millis(10));
            }
            assert!(!scratch.paths().socket().exists());
            assert!(ProbeLock::acquire_as(&scratch.paths().lock, scratch.paths().uid).is_ok());
            assert!(
                std::fs::read_to_string(scratch.0.join("public-results"))
                    .unwrap()
                    .contains("cleanup")
            );
        }
    }
}
/// @brief Racing acquisitions converge on one kernel lock owner, without replacing its inode.
#[test]
fn g2_2_racing_owners() {
    let scratch = Scratch::new();
    let path = scratch.paths().lock;
    let uid = scratch.paths().uid;
    let ready = Arc::new(std::sync::Barrier::new(3));
    let release = Arc::new(std::sync::Barrier::new(3));
    let mut tasks = Vec::new();
    for _ in 0..2 {
        let (p, r, q) = (path.clone(), ready.clone(), release.clone());
        tasks.push(std::thread::spawn(move || {
            r.wait();
            let owner = ProbeLock::acquire_as(&p, uid);
            q.wait();
            owner.is_ok()
        }));
    }
    ready.wait();
    release.wait();
    assert_eq!(
        tasks
            .into_iter()
            .map(|t| usize::from(t.join().unwrap()))
            .sum::<usize>(),
        1
    );
}

/// @brief The actual client startup path launches a worker on demand; repeated close never starts one.
#[test]
fn g2_2_on_demand_commands() {
    let scratch = Scratch::new();
    scratch.pair();
    for (mode, expected) in [
        ("keep", "Transport"),
        ("keep", "Transport"),
        ("send", "Busy"),
        ("close", "Closed"),
        ("close", "Closed"),
    ] {
        let o = child(mode, &scratch.0, 23).output().unwrap();
        assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
        assert!(String::from_utf8_lossy(&o.stdout).contains(&format!("outcome={expected}")));
    }
    assert!(!scratch.paths().socket().exists());
    let log = std::fs::read_to_string(scratch.0.join("public-results")).unwrap();
    assert_eq!(log.matches("established").count(), 1);
    assert!(log.contains("sequence=1"));
}
/// @brief Expiry terminates idle workers and removes their socket under a controlled clock.
#[tokio::test(start_paused = true)]
async fn g2_2_idle_expiry() {
    let scratch = Scratch::new();
    let pair = scratch.pair();
    let paths = scratch.paths();
    let log = scratch.0.join("public-results");
    let task = tokio::spawn(async move {
        let owner = ProbeLock::acquire_as(&paths.lock, paths.uid).unwrap();
        serve(
            paths,
            pair,
            Native::new(23, Some(log)),
            std::future::pending(),
            &owner,
        )
        .await
    });
    tokio::task::yield_now().await;
    let mut s = UnixStream::connect(scratch.paths().socket()).await.unwrap();
    assert_eq!(
        exchange(&mut s, Op::Keep, "Hello, world!").await.unwrap(),
        Outcome::Transport
    );
    drop(s);
    tokio::time::advance(Duration::from_secs(120)).await;
    assert_eq!(task.await.unwrap(), Err("session_expired"));
    assert!(!scratch.paths().socket().exists());
    assert!(
        std::fs::read_to_string(scratch.0.join("public-results"))
            .unwrap()
            .contains("cleanup")
    );
}
/// @brief Unreachable close finishes local teardown after two seconds, without receipt waits or retry.
#[tokio::test(start_paused = true)]
async fn g2_2_unreachable_close() {
    let scratch = Scratch::new();
    let pair = scratch.pair();
    let mut b = Native::new(23, None);
    let mut kept = None;
    operation(&mut b, &mut kept, pair.clone(), Op::Keep, b"Hello, world!")
        .await
        .unwrap();
    b.delay = Duration::from_secs(100);
    let start = Instant::now();
    assert_eq!(
        operation(&mut b, &mut kept, pair, Op::Close, &[])
            .await
            .unwrap(),
        Outcome::Closed
    );
    assert_eq!(Instant::now() - start, Duration::from_secs(2));
    assert!(kept.is_none() && b.peer.is_none());
}

/// @brief A partial fragment transfer returns a phase-specific failure and never releases plaintext/retries.
#[tokio::test]
async fn g2_2_partial_transfer_failure() {
    let scratch = Scratch::new();
    let pair = scratch.pair();
    let paths = scratch.paths();
    let log = scratch.0.join("public-results");
    let task = tokio::spawn(async move {
        let owner = ProbeLock::acquire_as(&paths.lock, paths.uid).unwrap();
        let mut b = Native::new(23, Some(log));
        b.fail_fragment = Some(1);
        serve(paths, pair, b, std::future::pending(), &owner).await
    });
    tokio::task::yield_now().await;
    let mut s = UnixStream::connect(scratch.paths().socket()).await.unwrap();
    assert_eq!(
        exchange(&mut s, Op::Keep, "Hello, world!").await.unwrap(),
        Outcome::TransferFailed
    );
    assert_eq!(task.await.unwrap(), Err("partial_transfer_failed"));
    let log = std::fs::read_to_string(scratch.0.join("public-results")).unwrap();
    assert_eq!(log.matches("partial_transfer_failed").count(), 1);
    assert!(!log.contains("decrypted="));
    assert!(log.contains("cleanup"));
    assert!(!scratch.paths().socket().exists());
}

/// @brief The production startup loop and IPC reader both stop at five seconds without refreshing time.
#[tokio::test(start_paused = true)]
async fn g2_2_local_deadlines() {
    let scratch = Scratch::new();
    let start = Instant::now();
    let mut launched = 0;
    let r = client_at(scratch.paths(), Op::Keep, "Hello, world!", || {
        launched += 1;
        tokio::process::Command::new("true")
            .spawn()
            .map_err(|_| "test_launch_failed")
    })
    .await;
    assert_eq!(r, Err("busy_or_worker_startup_failed"));
    assert_eq!(launched, 1);
    assert_eq!(Instant::now() - start, LOCAL);
    let listener = UnixListener::bind(scratch.paths().socket()).unwrap();
    let _client = UnixStream::connect(scratch.paths().socket()).await.unwrap();
    let (mut s, _) = listener.accept().await.unwrap();
    let start = Instant::now();
    assert_eq!(request(&mut s).await.err(), Some("ipc_request_timeout"));
    assert_eq!(Instant::now() - start, LOCAL);
}
/// @brief A competing operation receives busy while the admitted request owns an incomplete send.
#[tokio::test]
async fn g2_2_competing_operation() {
    let scratch = Scratch::new();
    let pair = scratch.pair();
    let paths = scratch.paths();
    let log = scratch.0.join("public-results");
    let worker = tokio::spawn(async move {
        let owner = ProbeLock::acquire_as(&paths.lock, paths.uid).unwrap();
        let mut b = Native::new(23, Some(log));
        b.delay = Duration::from_secs(1);
        serve(paths, pair, b, std::future::pending(), &owner).await
    });
    tokio::task::yield_now().await;
    let mut s = UnixStream::connect(scratch.paths().socket()).await.unwrap();
    let first = tokio::spawn(async move { exchange(&mut s, Op::Keep, "Hello, world!").await });
    let until = Instant::now() + LOCAL;
    while !scratch.0.join("public-results").exists() {
        assert!(Instant::now() < until);
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    let mut other = UnixStream::connect(scratch.paths().socket()).await.unwrap();
    assert_eq!(
        exchange(&mut other, Op::Keep, "Hello, world!")
            .await
            .unwrap(),
        Outcome::Busy
    );
    first.abort();
    let _ = first.await;
    assert_eq!(worker.await.unwrap(), Err("ipc_lost_or_trailing_data"));
    assert!(!scratch.paths().socket().exists());
}
