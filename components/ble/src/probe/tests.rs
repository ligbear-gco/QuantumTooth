//! @file
//! @brief Exercise transport failure boundaries without a controller or phone.

use super::*;
use crate::ownership::ProbeLock;
use crate::state::{EXCHANGE_LIMIT, READY_LIMIT};
use bluer::gatt::WriteOp;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    task::Poll,
};

/// @brief Stable synthetic peer identity used only inside unit tests.
fn peer() -> Address {
    Address::new([0, 0, 0, 0, 0, 1])
}

/// @brief Ready one test state and return its bounded callback handoff.
fn ready() -> (Gate, mpsc::Sender<Transfer>, mpsc::Receiver<Transfer>) {
    let mut gate = Gate::new(Instant::now());
    gate.peer = Some(peer());
    gate.subscribed = true;
    assert!(gate.update_readiness(Instant::now()));
    let (tx, rx) = mpsc::channel(1);
    (gate, tx, rx)
}

/// @brief Build an ordinary offset-zero request at the minimum ATT MTU.
fn write(bytes: &[u8]) -> Write<'_> {
    Write {
        peer: peer(),
        adapter_matches: true,
        offset: 0,
        operation: WriteOp::Request,
        prepare: false,
        mtu: 23,
        bytes,
    }
}

/// @brief Only the exact public token reaches the handoff, including at MTU 23.
#[tokio::test]
async fn accepts_only_exact_token_and_reports_request_mtu() {
    for bytes in [&b""[..], &b"QTB"[..], &b"QTB12"[..], &b"NOPE"[..]] {
        let (mut gate, tx, mut rx) = ready();
        assert!(gate.submit(write(bytes), &tx, Instant::now()).is_err());
        assert!(rx.try_recv().is_err());
        assert!(!gate.busy);
    }
    for mtu in [23, 247, 517] {
        let (mut gate, tx, mut rx) = ready();
        let mut request = write(&TOKEN);
        request.mtu = mtu;
        assert_eq!(gate.submit(request, &tx, Instant::now()), Ok(()));
        assert_eq!(rx.recv().await.unwrap().mtu, mtu);
    }
}

/// @brief Reject unsupported ATT operations, offsets, peers and invalid MTUs.
#[tokio::test]
async fn rejects_wrong_operation_offset_adapter_peer_and_prepare() {
    for case in 0..7 {
        let (mut gate, tx, mut rx) = ready();
        let mut request = write(&TOKEN);
        match case {
            0 => request.operation = WriteOp::Command,
            1 => request.operation = WriteOp::Reliable,
            2 => request.offset = 1,
            3 => request.prepare = true,
            4 => request.peer = Address::new([0, 0, 0, 0, 0, 2]),
            5 => request.adapter_matches = false,
            _ => request.mtu = 22,
        }
        assert!(gate.submit(request, &tx, Instant::now()).is_err());
        assert!(rx.try_recv().is_err());
    }
}

/// @brief A connected phone cannot write before TX subscription/readiness.
#[tokio::test]
async fn rejects_pre_subscription_writes() {
    let (mut gate, tx, mut rx) = ready();
    gate.subscribed = false;
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::NotPermitted)
    );
    assert!(rx.try_recv().is_err());
}

/// @brief Reserve the transfer until confirmation, even after the queue is drained.
#[tokio::test]
async fn busy_handoff_never_buffers_a_second_write() {
    let (mut gate, tx, mut rx) = ready();
    assert!(gate.submit(write(&TOKEN), &tx, Instant::now()).is_ok());
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::InProgress)
    );
    rx.recv().await.unwrap();
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::InProgress)
    );
    assert!(rx.try_recv().is_err());
}

/// @brief Queue saturation and a closed consumer reject without reserving state.
#[tokio::test]
async fn full_and_closed_handoffs_fail() {
    let (mut gate, tx, rx) = ready();
    tx.try_send(Transfer { mtu: 23 }).unwrap();
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::InProgress)
    );
    assert!(!gate.busy);
    drop(rx);
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::NotPermitted)
    );
}

/// @brief Readiness requires both observations and never starts after expiry.
#[tokio::test(start_paused = true)]
async fn readiness_requires_connection_and_subscription_before_thirty_seconds() {
    for subscription_first in [true, false] {
        let start = Instant::now();
        let mut gate = Gate::new(start);
        if subscription_first {
            gate.subscribed = true;
        } else {
            gate.peer = Some(peer());
        }
        assert!(!gate.update_readiness(start));
        tokio::time::advance(Duration::from_secs(29)).await;
        gate.subscribed = true;
        gate.peer = Some(peer());
        assert!(gate.update_readiness(Instant::now()));
        assert_eq!(gate.deadline, start + Duration::from_secs(149));
        assert!(!gate.update_readiness(Instant::now()));
    }
    let mut gate = Gate::new(Instant::now());
    tokio::time::advance(READY_LIMIT).await;
    gate.subscribed = true;
    gate.peer = Some(peer());
    assert!(!gate.update_readiness(Instant::now()));
}

/// @brief Rejected traffic and duplicate readiness never extend the exchange window.
#[tokio::test(start_paused = true)]
async fn absolute_exchange_deadline_does_not_refresh() {
    let (mut gate, tx, mut rx) = ready();
    let deadline = gate.deadline;
    for _ in 0..119 {
        tokio::time::advance(Duration::from_secs(1)).await;
        assert!(gate.submit(write(b"NOPE"), &tx, Instant::now()).is_err());
        assert!(!gate.update_readiness(Instant::now()));
        assert_eq!(gate.deadline, deadline);
    }
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::NotPermitted)
    );
    assert!(rx.try_recv().is_err());
}

/// @brief Pending delivery cannot report success and expires without wall-clock sleep.
#[tokio::test(start_paused = true)]
async fn pending_confirmation_cannot_succeed() {
    let deadline = Instant::now() + EXCHANGE_LIMIT;
    let result = confirm_by(pending(), pending(), deadline).await;
    assert_eq!(result, Err(Failure::new("exchange", "timeout")));
    assert_eq!(Instant::now(), deadline);
}

/// @brief Failed delivery and subscription loss fail before the deadline.
#[tokio::test(start_paused = true)]
async fn failed_or_lost_indication_cannot_succeed() {
    let deadline = Instant::now() + EXCHANGE_LIMIT;
    assert_eq!(
        confirm_by(async { Err(()) }, pending(), deadline).await,
        Err(Failure::new("indication", "unconfirmed_or_failed"))
    );
    assert_eq!(
        confirm_by(pending(), async {}, deadline).await,
        Err(Failure::new("exchange", "subscription_lost"))
    );
    assert!(Instant::now() < deadline);
}

/// @brief An expired deadline or already-lost subscription wins over a ready result.
#[tokio::test(start_paused = true)]
async fn confirmation_must_arrive_before_expiry_and_link_loss() {
    assert!(
        confirm_by(async { Ok(()) }, pending(), Instant::now())
            .await
            .is_err()
    );
    let deadline = Instant::now() + EXCHANGE_LIMIT;
    assert!(
        confirm_by(async { Ok(()) }, async {}, deadline)
            .await
            .is_err()
    );
    assert_eq!(
        confirm_by(async { Ok(()) }, pending(), deadline).await,
        Ok(())
    );
}

/// @brief Cancellation drops a pending exchange rather than leaving a background task.
#[tokio::test]
async fn cancellation_drops_pending_confirmation() {
    let dropped = Arc::new(AtomicBool::new(false));
    /// @brief Observe destruction of the pending delivery future.
    struct DropNotice(Arc<AtomicBool>);
    impl Drop for DropNotice {
        /// @brief Signal that no worker retains this transfer.
        fn drop(&mut self) {
            self.0.store(true, Ordering::SeqCst);
        }
    }
    let notice = DropNotice(dropped.clone());
    let delivery = async move {
        let _notice = notice;
        pending::<Result<(), ()>>().await
    };
    let mut future = Box::pin(confirm_by(
        delivery,
        pending(),
        Instant::now() + EXCHANGE_LIMIT,
    ));
    assert!(matches!(futures::poll!(&mut future), Poll::Pending));
    drop(future);
    assert!(dropped.load(Ordering::SeqCst));
}

/// @brief Callback handoff completes even while indication confirmation is pending.
#[tokio::test]
async fn callback_acceptance_does_not_wait_for_confirmation() {
    let (mut gate, tx, mut rx) = ready();
    let mut confirmation = Box::pin(confirm_by(pending(), pending(), gate.deadline));
    assert!(matches!(futures::poll!(&mut confirmation), Poll::Pending));
    assert_eq!(gate.submit(write(&TOKEN), &tx, Instant::now()), Ok(()));
    assert_eq!(rx.recv().await.unwrap().mtu, 23);
    assert!(matches!(futures::poll!(&mut confirmation), Poll::Pending));
}

/// @brief Cleanup invalidates stale callbacks and a fresh run starts empty.
#[tokio::test]
async fn cleanup_discards_subscription_transfer_and_peer_state() {
    let (mut gate, tx, mut rx) = ready();
    gate.close();
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::NotPermitted)
    );
    assert!(rx.try_recv().is_err());
    let fresh = Gate::new(Instant::now());
    assert!(fresh.peer.is_none() && !fresh.subscribed && !fresh.busy && fresh.ready_at.is_none());
}

/// @brief The actual kernel lock excludes a second open and permits a fresh run.
#[test]
fn concurrent_ownership_and_fresh_restart() {
    let path = std::env::temp_dir().join(format!("qtooth-lock-{}", Uuid::new_v4()));
    let first = ProbeLock::acquire(&path).unwrap();
    assert!(ProbeLock::acquire(&path).is_err());
    drop(first);
    let fresh = ProbeLock::acquire(&path).unwrap();
    assert!(ProbeLock::acquire(&path).is_err());
    drop(fresh);
    std::fs::remove_file(path).unwrap();
}

/// @brief Runtime ownership refuses symlink redirection.
#[test]
fn lock_does_not_follow_symlinks() {
    let path = std::env::temp_dir().join(format!("qtooth-symlink-{}", Uuid::new_v4()));
    std::os::unix::fs::symlink("/dev/null", &path).unwrap();
    assert!(ProbeLock::acquire(&path).is_err());
    std::fs::remove_file(path).unwrap();
}

/// @brief The GATT definition has fixed UUIDs, request-only RX and indicate-only TX.
#[test]
fn service_definition_matches_transport_contract() {
    let (writes, _) = mpsc::channel(1);
    let (subscriptions, _) = mpsc::channel(1);
    let app = application(
        Arc::new(Mutex::new(Gate::new(Instant::now()))),
        writes,
        subscriptions,
        "hci0".into(),
    );
    let service = &app.services[0];
    assert_eq!(
        service.uuid.to_string(),
        "b6631000-7e7b-4fb5-9dc4-93f4152a0e7a"
    );
    assert!(service.primary);
    let rx = &service.characteristics[0];
    assert_eq!(rx.uuid, RX);
    let write = rx.write.as_ref().unwrap();
    assert!(write.write && !write.write_without_response && !write.reliable_write);
    assert!(!write.authenticated_signed_writes && rx.read.is_none() && rx.notify.is_none());
    let tx = &service.characteristics[1];
    assert_eq!(tx.uuid, TX);
    let notify = tx.notify.as_ref().unwrap();
    assert!(notify.indicate && !notify.notify);
    assert!(tx.write.is_none() && tx.read.is_none());
    assert!(rx.descriptors.is_empty() && tx.descriptors.is_empty());
}

/// @brief Unrelated links cannot become owned, and a second new peer cannot replace the first.
#[tokio::test]
async fn connection_selection_preserves_existing_links_and_first_peer() {
    let gate = Arc::new(Mutex::new(Gate::new(Instant::now())));
    let mut resources = Resources::default();
    let existing = Address::new([0, 0, 0, 0, 0, 2]);
    let second = Address::new([0, 0, 0, 0, 0, 3]);
    let unrelated = [existing].into_iter().collect();
    connected(&mut resources, &gate, existing, &unrelated).unwrap();
    assert!(gate.lock().unwrap().peer.is_none());
    connected(&mut resources, &gate, peer(), &unrelated).unwrap();
    let deadline = gate.lock().unwrap().deadline;
    connected(&mut resources, &gate, peer(), &unrelated).unwrap();
    assert_eq!(gate.lock().unwrap().deadline, deadline);
    assert!(connected(&mut resources, &gate, second, &unrelated).is_err());
    assert_eq!(gate.lock().unwrap().peer, Some(peer()));
}

/// @brief Owned link loss closes the real write gate without disturbing another peer.
#[tokio::test]
async fn link_loss_invalidates_writes_and_preserves_cleanup_identity() {
    let (mut gate, tx, mut rx) = ready();
    assert!(!gate.lost_link(Address::new([0, 0, 0, 0, 0, 2])));
    assert!(!gate.closed);
    assert!(gate.lost_link(peer()));
    assert_eq!(gate.peer, Some(peer()));
    assert_eq!(
        gate.submit(write(&TOKEN), &tx, Instant::now()),
        Err(ReqError::NotPermitted)
    );
    assert!(rx.try_recv().is_err());
}
