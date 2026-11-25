//! @file
//! @brief BlueZ GATT callbacks, event ownership, confirmation and cleanup.

use crate::state::{Gate, TOKEN, Transfer, Write};
use bluer::{
    Adapter, AdapterEvent, Address, DeviceEvent, DeviceProperty,
    adv::{Advertisement, AdvertisementHandle},
    gatt::local::{
        Application, ApplicationHandle, Characteristic, CharacteristicNotifier,
        CharacteristicNotify, CharacteristicNotifyMethod, CharacteristicWrite,
        CharacteristicWriteMethod, ReqError, Service,
    },
};
use futures::{
    FutureExt, StreamExt,
    future::{BoxFuture, pending},
    stream::{BoxStream, SelectAll},
};
use std::{
    collections::HashSet,
    future::Future,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::mpsc,
    time::{Duration, Instant, sleep, sleep_until, timeout, timeout_at},
};
use uuid::Uuid;

/// @brief Stable diagnostic service UUID, separate from future authenticated BLE.
pub const SERVICE: Uuid = Uuid::from_u128(0xb6631000_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Phone-to-Pi write-with-response characteristic.
pub const RX: Uuid = Uuid::from_u128(0xb6631001_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Pi-to-phone indication-only characteristic.
pub const TX: Uuid = Uuid::from_u128(0xb6631002_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Bound known-device observers, including pre-existing unrelated links.
const MAX_DEVICES: usize = 64;
/// @brief Bound setup and each additional device-observer D-Bus request.
const SETUP_LIMIT: Duration = Duration::from_secs(10);
/// @brief Bound owned-link disconnection during cleanup.
const CLEANUP_LIMIT: Duration = Duration::from_secs(3);

/// @brief Use fixed phase/reason text; backend details can contain peer addresses.
#[derive(Debug, PartialEq, Eq)]
pub struct Failure {
    pub phase: &'static str,
    pub reason: &'static str,
}

impl Failure {
    /// @brief Construct a sanitized actionable diagnostic.
    fn new(phase: &'static str, reason: &'static str) -> Self {
        Self { phase, reason }
    }
}

/// @brief All BlueZ handles acquired so far, including partially completed setup.
#[derive(Default)]
struct Resources {
    session: Option<bluer::Session>,
    adapter: Option<Adapter>,
    app: Option<ApplicationHandle>,
    advertisement: Option<AdvertisementHandle>,
}

/// @brief Construct callbacks without spawning tasks or awaiting write delivery.
fn application(
    gate: Arc<Mutex<Gate>>,
    writes: mpsc::Sender<Transfer>,
    subscriptions: mpsc::Sender<CharacteristicNotifier>,
    adapter_name: String,
) -> Application {
    Application {
        services: vec![Service {
            uuid: SERVICE,
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: RX,
                    write: Some(CharacteristicWrite {
                        // BlueR 0.17.4 reverses these two field docstrings;
                        // BlueZ's "write" flag means request/response.
                        write: true,
                        write_without_response: false,
                        method: CharacteristicWriteMethod::Fun(Box::new(move |bytes, request| {
                            let result = gate
                                .try_lock()
                                .map_err(|_| ReqError::InProgress)
                                .and_then(|mut state| {
                                    state.submit(
                                        Write {
                                            peer: request.device_address,
                                            adapter_matches: request.adapter_name == adapter_name,
                                            offset: request.offset,
                                            operation: request.op_type,
                                            prepare: request.prepare_authorize,
                                            mtu: request.mtu,
                                            bytes: &bytes,
                                        },
                                        &writes,
                                        Instant::now(),
                                    )
                                });
                            if let Err(reason) = result {
                                eprintln!(
                                    "phase=write rejected={reason} operation={:?} prepare={} offset={} mtu={} bytes={}",
                                    request.op_type,
                                    request.prepare_authorize,
                                    request.offset,
                                    request.mtu,
                                    bytes.len()
                                );
                            }
                            async move { result }.boxed()
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
                Characteristic {
                    uuid: TX,
                    notify: Some(CharacteristicNotify {
                        indicate: true,
                        notify: false,
                        method: CharacteristicNotifyMethod::Fun(Box::new(move |notifier| {
                            // @note BlueZ waits for this callback before completing StartNotify.
                            // A busy handoff drops the new notifier, invalidating that session.
                            let _ = subscriptions.try_send(notifier);
                            async {}.boxed()
                        })),
                        ..Default::default()
                    }),
                    ..Default::default()
                },
            ],
            ..Default::default()
        }],
        ..Default::default()
    }
}

/// @brief Map a device property or stream termination back to its internal peer.
type DeviceChange = (Address, Option<DeviceEvent>);

/// @brief Register an observer before reading connection state to avoid lost edges.
async fn observe(
    adapter: &Adapter,
    address: Address,
    changes: &mut SelectAll<BoxStream<'static, DeviceChange>>,
) -> Result<bool, Failure> {
    let device = adapter
        .device(address)
        .map_err(|_| Failure::new("connection", "device_unavailable"))?;
    let events = device
        .events()
        .await
        .map_err(|_| Failure::new("connection", "cannot_watch_device"))?;
    changes.push(
        events
            .map(move |event| (address, Some(event)))
            .chain(futures::stream::once(async move { (address, None) }))
            .boxed(),
    );
    device
        .is_connected()
        .await
        .map_err(|_| Failure::new("connection", "cannot_read_link"))
}

/// @brief Wait for transport confirmation, failing on unsubscribe or absolute expiry.
/// @param delivery A future that completes only on actual indication confirmation.
/// @return Success only after confirmed delivery within the same phase deadline.
pub async fn confirm_by(
    delivery: impl Future<Output = Result<(), ()>>,
    stopped: impl Future<Output = ()>,
    deadline: Instant,
) -> Result<(), Failure> {
    tokio::select! {
        biased;
        _ = sleep_until(deadline) => Err(Failure::new("exchange", "timeout")),
        _ = stopped => Err(Failure::new("exchange", "subscription_lost")),
        result = delivery => result.map_err(|_| Failure::new("indication", "unconfirmed_or_failed")),
    }
}

/// @brief Event streams and bounded callback handoffs owned by the foreground loop.
struct Pump {
    adapter_events: BoxStream<'static, AdapterEvent>,
    changes: SelectAll<BoxStream<'static, DeviceChange>>,
    observed: HashSet<Address>,
    unrelated: HashSet<Address>,
    writes: mpsc::Receiver<Transfer>,
    subscriptions: mpsc::Receiver<CharacteristicNotifier>,
}

/// @brief Acquire observers and GATT before advertising; never alter adapter power.
async fn setup(resources: &mut Resources, gate: &Arc<Mutex<Gate>>) -> Result<Pump, Failure> {
    let session = bluer::Session::new()
        .await
        .map_err(|_| Failure::new("dbus", "check_system_bus_and_permissions"))?;
    resources.session = Some(session.clone());
    let adapter = session
        .default_adapter()
        .await
        .map_err(|_| Failure::new("adapter", "check_controller_and_bluetooth_service"))?;
    resources.adapter = Some(adapter.clone());
    if !adapter
        .is_powered()
        .await
        .map_err(|_| Failure::new("adapter", "cannot_read_power"))?
    {
        return Err(Failure::new(
            "adapter",
            "power_controller_with_bluetoothctl_first",
        ));
    }
    let adapter_events = adapter
        .events()
        .await
        .map_err(|_| Failure::new("adapter", "cannot_watch_adapter"))?
        .boxed();
    let mut changes = SelectAll::new();
    let mut observed = HashSet::new();
    let mut unrelated = HashSet::new();
    let addresses = adapter
        .device_addresses()
        .await
        .map_err(|_| Failure::new("connection", "cannot_list_devices"))?;
    if addresses.len() > MAX_DEVICES {
        return Err(Failure::new("setup", "too_many_known_devices"));
    }
    for address in addresses {
        observed.insert(address);
        if observe(&adapter, address, &mut changes).await? {
            unrelated.insert(address);
        }
    }
    let (write_tx, writes) = mpsc::channel(1);
    let (subscription_tx, subscriptions) = mpsc::channel(1);
    resources.app = Some(
        adapter
            .serve_gatt_application(application(
                gate.clone(),
                write_tx,
                subscription_tx,
                adapter.name().to_owned(),
            ))
            .await
            .map_err(|_| {
                Failure::new("registration", "check_bluez_gatt_support_and_permissions")
            })?,
    );
    let advertised = Instant::now();
    resources.advertisement = Some(
        adapter
            .advertise(Advertisement {
                service_uuids: [SERVICE].into_iter().collect(),
                local_name: Some("QTooth Probe".to_owned()),
                discoverable: Some(true),
                ..Default::default()
            })
            .await
            .map_err(|_| Failure::new("advertising", "check_controller_advertising_support"))?,
    );
    *gate.lock().expect("gate poisoned") = Gate::new(advertised);
    println!("phase=advertising unauthenticated=true readiness_limit_s=30");
    Ok(Pump {
        adapter_events,
        changes,
        observed,
        unrelated,
        writes,
        subscriptions,
    })
}

/// @brief Select one new connection and stop this probe's advertisement.
fn connected(
    resources: &mut Resources,
    gate: &Arc<Mutex<Gate>>,
    address: Address,
    unrelated: &HashSet<Address>,
) -> Result<(), Failure> {
    if unrelated.contains(&address) {
        return Ok(());
    }
    let mut state = gate.lock().expect("gate poisoned");
    match state.peer {
        Some(peer) if peer != address => {
            return Err(Failure::new(
                "connection",
                "multiple_new_peers_restart_with_one_phone",
            ));
        }
        Some(_) => return Ok(()),
        None => state.peer = Some(address),
    }
    drop(resources.advertisement.take());
    println!("phase=connected");
    if state.update_readiness(Instant::now()) {
        println!("phase=ready exchange_limit_s=120");
    }
    Ok(())
}

impl Pump {
    /// @brief Drive peer events and indications without detached application tasks.
    async fn exchange(
        mut self,
        resources: &mut Resources,
        gate: &Arc<Mutex<Gate>>,
    ) -> Result<(), Failure> {
        let adapter = resources
            .adapter
            .as_ref()
            .expect("adapter acquired")
            .clone();
        let mut notifier = None;
        let mut stopped: Option<BoxFuture<'static, ()>> = None;
        let mut indication: Option<BoxFuture<'static, Result<(), Failure>>> = None;
        loop {
            let deadline = gate.lock().expect("gate poisoned").deadline;
            tokio::select! {
                biased;
                _ = sleep_until(deadline) => {
                    let ready = gate.lock().expect("gate poisoned").ready_at.is_some();
                    return Err(Failure::new(if ready { "exchange" } else { "readiness" }, "timeout"));
                }
                event = self.adapter_events.next() => match event {
                    Some(AdapterEvent::DeviceAdded(address)) if !self.observed.contains(&address) => {
                        if self.observed.len() >= MAX_DEVICES { return Err(Failure::new("connection", "device_observer_limit")); }
                        self.observed.insert(address);
                        let is_connected = timeout_at(deadline.min(Instant::now() + SETUP_LIMIT), observe(&adapter, address, &mut self.changes)).await
                            .map_err(|_| Failure::new("connection", "observer_timeout"))??;
                        if is_connected { connected(resources, gate, address, &self.unrelated)?; }
                    }
                    Some(AdapterEvent::DeviceRemoved(address))
                        if gate.lock().expect("gate poisoned").lost_link(address) =>
                            return Err(Failure::new("connection", "peer_removed")),
                    None => return Err(Failure::new("adapter", "adapter_or_bus_lost")),
                    _ => {}
                },
                change = self.changes.next(), if !self.changes.is_empty() => {
                    if let Some((address, event)) = change {
                        match event {
                            Some(DeviceEvent::PropertyChanged(DeviceProperty::Connected(true))) =>
                                connected(resources, gate, address, &self.unrelated)?,
                            Some(DeviceEvent::PropertyChanged(DeviceProperty::Connected(false))) | None
                                if gate.lock().expect("gate poisoned").lost_link(address) =>
                                    return Err(Failure::new("connection", "link_lost")),
                            _ => {}
                        }
                    }
                }
                _ = async { match &mut stopped { Some(f) => f.await, None => pending().await } } =>
                    return Err(Failure::new("exchange", "subscription_lost")),
                result = async { match &mut indication { Some(f) => f.await, None => pending().await } } => {
                    result?;
                    println!("phase=indication tx_bytes=4 confirmed=true");
                    return Ok(());
                }
                subscription = self.subscriptions.recv() => {
                    let n = subscription.ok_or_else(|| Failure::new("subscription", "handoff_closed"))?;
                    if stopped.is_some() { return Err(Failure::new("subscription", "duplicate_subscription")); }
                    if !n.confirming() || n.is_stopped() { return Err(Failure::new("subscription", "indications_required")); }
                    stopped = Some(n.stopped().boxed());
                    notifier = Some(n);
                    let mut state = gate.lock().expect("gate poisoned");
                    state.subscribed = true;
                    println!("phase=subscribed confirming=true");
                    if state.update_readiness(Instant::now()) { println!("phase=ready exchange_limit_s=120"); }
                }
                transfer = self.writes.recv() => {
                    let transfer = transfer.ok_or_else(|| Failure::new("write", "handoff_closed"))?;
                    println!("phase=write rx_bytes=4 att_mtu={}", transfer.mtu);
                    let mut n = notifier.take().ok_or_else(|| Failure::new("subscription", "not_ready"))?;
                    indication = Some(async move {
                        let stopped = n.stopped();
                        confirm_by(async { n.notify(TOKEN.to_vec()).await.map_err(|_| ()) }, stopped, deadline).await
                    }.boxed());
                }
            }
        }
    }
}

/// @brief Close callback state, unregister handles, then disconnect only our peer.
/// @return Cleanup failures also prevent reporting a successful probe.
async fn cleanup(resources: &mut Resources, gate: &Arc<Mutex<Gate>>) -> Result<(), Failure> {
    let peer = {
        let mut state = gate.lock().expect("gate poisoned");
        state.close();
        state.peer
    };
    drop(resources.advertisement.take());
    drop(resources.app.take());
    let result = if let (Some(adapter), Some(peer)) = (&resources.adapter, peer) {
        timeout(CLEANUP_LIMIT, async {
            let device = adapter.device(peer).map_err(|_| ())?;
            if device.is_connected().await.map_err(|_| ())? {
                device.disconnect().await.map_err(|_| ())?;
            }
            Ok::<_, ()>(())
        })
        .await
        .map_err(|_| Failure::new("cleanup", "disconnect_timeout"))?
        .map_err(|_| Failure::new("cleanup", "cannot_release_test_link"))
    } else {
        Ok(())
    };
    // @note BlueR unregisters dropped handles asynchronously. Allow dispatch;
    // runtime shutdown closes this process's D-Bus connection as the final barrier.
    sleep(Duration::from_millis(100)).await;
    drop(resources.adapter.take());
    drop(resources.session.take());
    println!("phase=cleanup");
    result
}

/// @brief Run under the process lock through setup, cancellation and bounded cleanup.
/// @param cancelled Ctrl-C or SIGTERM future registered before acquiring resources.
/// @return Success only for confirmed exchange followed by successful cleanup.
pub async fn run(cancelled: impl Future<Output = ()>) -> Result<(), Failure> {
    let gate = Arc::new(Mutex::new(Gate::new(Instant::now())));
    let mut resources = Resources::default();
    tokio::pin!(cancelled);
    let result = async {
        let pump = tokio::select! {
            biased;
            _ = &mut cancelled => return Err(Failure::new("setup", "cancelled")),
            result = timeout(SETUP_LIMIT, setup(&mut resources, &gate)) =>
                result.map_err(|_| Failure::new("setup", "dbus_or_registration_timeout"))??,
        };
        tokio::select! {
            biased;
            _ = &mut cancelled => Err(Failure::new("probe", "cancelled")),
            result = pump.exchange(&mut resources, &gate) => result,
        }
    }
    .await;
    let cleanup_result = cleanup(&mut resources, &gate).await;
    if let Err(failure) = &cleanup_result {
        eprintln!("phase={} error={}", failure.phase, failure.reason);
    }
    result.and(cleanup_result)
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod dbus_tests;

/// @brief Authenticated adapter reuses observation/resource mechanics, with distinct callbacks.
pub(crate) mod secure;
