//! @file
//! @brief Authenticated GATT adapter with bounded callback handoff and owned indications.
use super::*;
use qtooth_ble_core::{
    framing::{self, Kind, Reassembly},
    protocol::{Handshake, Pair, Session},
};
use tokio::sync::{Notify, watch};
/// @brief Distinct version-1 authenticated service.
pub const SERVICE: Uuid = Uuid::from_u128(0xb6632000_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Authenticated write-with-response characteristic.
const RX: Uuid = Uuid::from_u128(0xb6632001_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Authenticated indication-only characteristic.
const TX: Uuid = Uuid::from_u128(0xb6632002_7e7b_4fb5_9dc4_93f4152a0e7a);
/// @brief Safe adapter failure without backend identifiers.
type Result<T> = std::result::Result<T, &'static str>;
/// @brief One complete bounded object handed off after synchronous fragment validation.
struct Object {
    kind: Kind,
    body: Vec<u8>,
}
/// @brief Callback-owned bounds and protocol state; crypto runs only outside callbacks.
struct SecureGate {
    peer: Option<Address>,
    subscribed: bool,
    expected: Option<Kind>,
    rx: Reassembly,
    epoch: Instant,
    deadline: Instant,
    mtu: u16,
    failed: watch::Sender<bool>,
    incoming: Arc<Notify>,
}
impl SecureGate {
    /// @brief Fail closed and wake any idle/active protocol operation.
    fn abort(&mut self) {
        self.expected = None;
        self.rx.abort();
        self.failed.send_replace(true);
    }
    /// @brief Convert platform monotonic time for the platform-independent core.
    fn ms(&self, t: Instant) -> u64 {
        t.saturating_duration_since(self.epoch)
            .as_millis()
            .min(u64::MAX as u128) as u64
    }
    /// @brief Validate the actual BlueR request, then copy only contiguous bounded fragments.
    fn submit(
        &mut self,
        w: Write<'_>,
        tx: &mpsc::Sender<Object>,
        now: Instant,
    ) -> std::result::Result<(), ReqError> {
        let result = (|| {
            if *self.failed.borrow() || now >= self.deadline {
                return Err(ReqError::NotPermitted);
            }
            if w.operation != bluer::gatt::WriteOp::Request || w.prepare {
                return Err(ReqError::NotSupported);
            }
            if w.offset != 0 {
                return Err(ReqError::InvalidOffset);
            }
            if self.peer != Some(w.peer) || !w.adapter_matches || !self.subscribed {
                return Err(ReqError::NotPermitted);
            }
            let k = self.expected.ok_or(ReqError::NotPermitted)?;
            let body = self
                .rx
                .push(w.bytes, w.mtu, k, self.ms(now), self.ms(self.deadline))
                .map_err(|_| ReqError::NotPermitted)?;
            self.mtu = w.mtu;
            if let Some(body) = body {
                self.expected = None;
                tx.try_send(Object { kind: k, body })
                    .map_err(|_| ReqError::InProgress)?;
            }
            self.incoming.notify_one();
            Ok(())
        })();
        if result.is_err() {
            self.abort()
        };
        result
    }
}
/// @brief Construct production authenticated callbacks, separate from the public probe parser.
fn application(
    gate: Arc<Mutex<SecureGate>>,
    tx: mpsc::Sender<Object>,
    sub: mpsc::Sender<CharacteristicNotifier>,
    adapter: String,
) -> Application {
    Application {
        services: vec![Service {
            uuid: SERVICE,
            primary: true,
            characteristics: vec![
                Characteristic {
                    uuid: RX,
                    write: Some(CharacteristicWrite {
                        write: true,
                        write_without_response: false,
                        method: CharacteristicWriteMethod::Fun(Box::new(move |bytes, r| {
                            let result = gate
                                .try_lock()
                                .map_err(|_| ReqError::InProgress)
                                .and_then(|mut g| {
                                    g.submit(
                                        Write {
                                            peer: r.device_address,
                                            adapter_matches: r.adapter_name == adapter,
                                            offset: r.offset,
                                            operation: r.op_type,
                                            prepare: r.prepare_authorize,
                                            mtu: r.mtu,
                                            bytes: &bytes,
                                        },
                                        &tx,
                                        Instant::now(),
                                    )
                                });
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
                        method: CharacteristicNotifyMethod::Fun(Box::new(move |n| {
                            let _ = sub.try_send(n);
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
/// @brief Connection/event state owned by one worker; no application task is detached.
pub struct Link {
    resources: Resources,
    gate: Arc<Mutex<SecureGate>>,
    objects: mpsc::Receiver<Object>,
    subscriptions: mpsc::Receiver<CharacteristicNotifier>,
    failed: watch::Receiver<bool>,
    events: BoxStream<'static, AdapterEvent>,
    changes: SelectAll<BoxStream<'static, DeviceChange>>,
    observed: HashSet<Address>,
    unrelated: HashSet<Address>,
    notifier: Option<CharacteristicNotifier>,
    stopped: Option<BoxFuture<'static, ()>>,
    sequence: u32,
    incoming: Arc<Notify>,
}
impl Link {
    /// @brief Register secure GATT before advertising, preserving adapter power and unrelated links.
    pub async fn open() -> Result<Self> {
        let epoch = Instant::now();
        let (failed, rx_failed) = watch::channel(false);
        let incoming = Arc::new(Notify::new());
        let gate = Arc::new(Mutex::new(SecureGate {
            peer: None,
            subscribed: false,
            expected: Some(Kind::ClientHello),
            rx: Reassembly::default(),
            epoch,
            deadline: epoch + Duration::from_secs(30),
            mtu: 23,
            failed,
            incoming: incoming.clone(),
        }));
        let session = bluer::Session::new()
            .await
            .map_err(|_| "dbus_unavailable")?;
        let adapter = session
            .default_adapter()
            .await
            .map_err(|_| "adapter_unavailable")?;
        if !adapter.is_powered().await.map_err(|_| "power_unknown")? {
            return Err("controller_unpowered");
        }
        let events = adapter
            .events()
            .await
            .map_err(|_| "adapter_watch_failed")?
            .boxed();
        let mut changes = SelectAll::new();
        let mut observed = HashSet::new();
        let mut unrelated = HashSet::new();
        let addresses = adapter
            .device_addresses()
            .await
            .map_err(|_| "device_list_failed")?;
        if addresses.len() > MAX_DEVICES {
            return Err("observer_limit");
        }
        for a in addresses {
            observed.insert(a);
            if observe(&adapter, a, &mut changes)
                .await
                .map_err(|_| "observer_failed")?
            {
                unrelated.insert(a);
            }
        }
        let (tx, objects) = mpsc::channel(1);
        let (sub, subscriptions) = mpsc::channel(1);
        let app = adapter
            .serve_gatt_application(application(
                gate.clone(),
                tx,
                sub,
                adapter.name().to_owned(),
            ))
            .await
            .map_err(|_| "registration_failed")?;
        let start = Instant::now();
        gate.lock().unwrap().deadline = start + Duration::from_secs(30);
        let advertisement = adapter
            .advertise(Advertisement {
                service_uuids: [SERVICE].into_iter().collect(),
                local_name: Some("QTooth Secure".into()),
                discoverable: Some(true),
                ..Default::default()
            })
            .await
            .map_err(|_| "advertising_failed")?;
        Ok(Self {
            resources: Resources {
                session: Some(session),
                adapter: Some(adapter),
                app: Some(app),
                advertisement: Some(advertisement),
            },
            gate,
            objects,
            subscriptions,
            failed: rx_failed,
            events,
            changes,
            observed,
            unrelated,
            notifier: None,
            stopped: None,
            sequence: 0,
            incoming,
        })
    }
    /// @brief Set the phase/session deadline without extending an incomplete object's deadline.
    pub fn deadline(&mut self, d: Instant) {
        self.gate.lock().unwrap().deadline = d;
    }
    /// @brief Observe the original advertising-to-confirmation deadline.
    pub fn phase_deadline(&self) -> Instant {
        self.gate.lock().unwrap().deadline
    }
    /// @brief Select only one newly connected peer and stop owned advertising.
    fn connected(&mut self, a: Address) -> Result<()> {
        if self.unrelated.contains(&a) {
            return Ok(());
        }
        let mut g = self.gate.lock().unwrap();
        if g.peer.is_some_and(|p| p != a) {
            g.abort();
            return Err("multiple_peers");
        }
        g.peer = Some(a);
        drop(self.resources.advertisement.take());
        Ok(())
    }
    /// @brief Continuously monitor link/subscription/invalid traffic, also while an indication is pending.
    async fn event(&mut self) -> Result<()> {
        if *self.failed.borrow() {
            return Err("invalid_fragment_or_state");
        }
        let deadline = {
            let g = self.gate.lock().unwrap();
            g.epoch + Duration::from_millis(g.rx.deadline(g.ms(g.deadline)))
        };
        tokio::select! {biased;
            _=sleep_until(deadline)=>Err("object_or_phase_timeout"),
            _=self.failed.changed()=>Err("invalid_fragment_or_state"),
            _=self.incoming.notified()=>Ok(()),
            _=async{match &mut self.stopped{Some(f)=>f.await,None=>pending().await}}=>Err("subscription_lost"),
            n=self.subscriptions.recv()=>{
                let n=n.ok_or("subscription_handoff_closed")?;
                if self.stopped.is_some() || !n.confirming() || n.is_stopped(){return Err("invalid_subscription")}
                self.stopped=Some(n.stopped().boxed());self.notifier=Some(n);self.gate.lock().unwrap().subscribed=true;Ok(())
            },
            event=self.events.next()=>match event{
                Some(AdapterEvent::DeviceAdded(a))if !self.observed.contains(&a)=>{
                    if self.observed.len()>=MAX_DEVICES{return Err("observer_limit")}
                    self.observed.insert(a);let adapter=self.resources.adapter.as_ref().unwrap();
                    if timeout_at(deadline.min(Instant::now()+Duration::from_secs(5)),observe(adapter,a,&mut self.changes)).await.map_err(|_|"observer_timeout")?.map_err(|_|"observer_failed")?{self.connected(a)?;}Ok(())
                },
                Some(AdapterEvent::DeviceRemoved(a))if self.gate.lock().unwrap().peer==Some(a)=>Err("peer_removed"),None=>Err("adapter_or_bus_lost"),_=>Ok(())
            },
            change=self.changes.next(),if !self.changes.is_empty()=>{
                if let Some((a,e))=change{match e{Some(DeviceEvent::PropertyChanged(DeviceProperty::Connected(true)))=>self.connected(a)?,Some(DeviceEvent::PropertyChanged(DeviceProperty::Connected(false)))|None if self.gate.lock().unwrap().peer==Some(a)=>return Err("link_lost"),_=>{}}}Ok(())
            }
        }
    }
    /// @brief Receive a complete expected object while pumping all failure sources.
    async fn object(&mut self) -> Result<Object> {
        loop {
            let deadline = {
                let g = self.gate.lock().unwrap();
                g.epoch + Duration::from_millis(g.rx.deadline(g.ms(g.deadline)))
            };
            let objects = &mut self.objects;
            // @note Keep event and object borrows disjoint using a single guarded wait below.
            if let Ok(o) = objects.try_recv() {
                return Ok(o);
            }
            tokio::select! {biased;_=sleep_until(deadline)=>return Err("object_or_phase_timeout"),e=self.event()=>e?}
        }
    }
    /// @brief Send ordinary indications once, awaiting owned/cancellable futures and all failure sources.
    pub async fn send_object(&mut self, k: Kind, b: &[u8], outer: Instant) -> Result<()> {
        if self.sequence > u16::MAX as u32 {
            return Err("object_counter_exhausted");
        }
        let deadline = outer
            .min(self.phase_deadline())
            .min(Instant::now() + Duration::from_secs(10));
        let mtu = self.gate.lock().unwrap().mtu;
        let values =
            framing::fragments(k, self.sequence as u16, b, mtu).map_err(|_| "framing_failed")?;
        self.sequence += 1;
        for v in values {
            let mut n = self.notifier.take().ok_or("not_subscribed")?;
            let result = {
                let operation = att_transfer(n.notify(v), deadline);
                tokio::pin!(operation);
                loop {
                    tokio::select! {biased;e=self.event()=>if let Err(e)=e{break Err(e)},r=&mut operation=>break r}
                }
            };
            self.notifier = Some(n);
            result?;
        }
        Ok(())
    }
    /// @brief Complete authenticated establishment; no session is returned before finish transport completion.
    pub async fn establish(&mut self, pair: Arc<Pair>) -> Result<Session> {
        let deadline = self.phase_deadline();
        let mut h = Handshake::new(pair, true);
        for expected in [Kind::ClientHello, Kind::PeerFinish] {
            let o = self.object().await?;
            if o.kind != expected {
                return Err("wrong_state");
            }
            let (k, b) = h
                .receive(o.kind, &o.body)
                .map_err(|_| "authentication_failed")?
                .ok_or("missing_finish")?;
            if k == Kind::PiOffer {
                self.gate.lock().unwrap().expected = Some(Kind::PeerFinish);
            }
            self.send_object(k, &b, deadline).await?;
            if k == Kind::PiFinish {
                self.gate.lock().unwrap().expected = Some(Kind::Record);
            }
        }
        h.finish_sent().map_err(|_| "confirmation_failed")?;
        h.into_session().map_err(|_| "confirmation_failed")
    }
    /// @brief Idle retained worker still handles authenticated peer close and any malformed traffic.
    pub async fn idle(&mut self, session: &mut Session) -> Result<()> {
        loop {
            if let Ok(o) = self.objects.try_recv() {
                session.decrypt(&o.body).map_err(|_| "record_failed")?;
                return Err("peer_closed_or_unexpected_data");
            }
            self.event().await?;
        }
    }
    /// @brief Stop protocol work first, drop owned registrations and disconnect only the selected link.
    pub async fn cleanup(&mut self) -> Result<()> {
        let peer = {
            let mut g = self.gate.lock().unwrap();
            g.abort();
            g.peer
        };
        self.notifier = None;
        self.stopped = None;
        drop(self.resources.advertisement.take());
        drop(self.resources.app.take());
        let result = timeout(Duration::from_secs(2), async {
            if let Some(peer) = peer {
                let device = self
                    .resources
                    .adapter
                    .as_ref()
                    .unwrap()
                    .device(peer)
                    .map_err(|_| "cleanup_device_failed")?;
                if device
                    .is_connected()
                    .await
                    .map_err(|_| "cleanup_state_failed")?
                {
                    device
                        .disconnect()
                        .await
                        .map_err(|_| "cleanup_disconnect_failed")?;
                }
            }
            Ok(())
        })
        .await
        .map_err(|_| "cleanup_timeout")?;
        self.resources.adapter = None;
        self.resources.session = None;
        result
    }
}

/// @brief Bound each ordinary ATT operation to five seconds and the enclosing absolute deadline.
async fn att_transfer(
    delivery: impl Future<Output = bluer::Result<()>>,
    until: Instant,
) -> Result<()> {
    timeout_at(until.min(Instant::now() + Duration::from_secs(5)), delivery)
        .await
        .map_err(|_| "att_timeout")?
        .map_err(|_| "indication_failed")
}

#[cfg(test)]
mod tests {
    use super::*;
    use bluer::gatt::WriteOp;
    /// @brief Fresh production callback state with one synthetic selected peer.
    fn gate() -> (
        Arc<Mutex<SecureGate>>,
        mpsc::Sender<Object>,
        mpsc::Receiver<Object>,
    ) {
        let (failed, _) = watch::channel(false);
        let epoch = Instant::now();
        let (tx, rx) = mpsc::channel(1);
        (
            Arc::new(Mutex::new(SecureGate {
                peer: Some(Address::new([0, 0, 0, 0, 0, 1])),
                subscribed: true,
                expected: Some(Kind::ClientHello),
                rx: Reassembly::default(),
                epoch,
                deadline: epoch + Duration::from_secs(30),
                mtu: 23,
                failed,
                incoming: Arc::new(Notify::new()),
            })),
            tx,
            rx,
        )
    }
    /// @brief Construct ordinary request metadata consumed by the actual registered callback.
    fn request(bytes: &[u8], mtu: u16) -> Write<'_> {
        Write {
            peer: Address::new([0, 0, 0, 0, 0, 1]),
            adapter_matches: true,
            offset: 0,
            operation: WriteOp::Request,
            mtu,
            prepare: false,
            bytes,
        }
    }
    /// @brief Production callbacks enforce ATT metadata, protocol states, MTU and capacity-one completion.
    #[tokio::test]
    async fn g2_2_production_secure_callbacks() {
        for mtu in [23, 517] {
            let (g, tx, mut rx) = gate();
            for v in framing::fragments(Kind::ClientHello, 0, &[0; 68], mtu).unwrap() {
                g.lock()
                    .unwrap()
                    .submit(request(&v, mtu), &tx, Instant::now())
                    .unwrap();
            }
            let o = rx.recv().await.unwrap();
            assert_eq!(o.body.len(), 68);
            assert!(rx.try_recv().is_err());
            assert!(g.lock().unwrap().expected.is_none());
            assert!(
                g.lock()
                    .unwrap()
                    .submit(request(b"QTB1", mtu), &tx, Instant::now())
                    .is_err()
            );
            assert!(*g.lock().unwrap().failed.borrow());
        }
        for case in 0..9 {
            let (g, tx, _rx) = gate();
            let mut v = framing::fragments(Kind::ClientHello, 0, &[0; 68], 23)
                .unwrap()
                .remove(0);
            if case == 7 {
                v.extend([0; 20]);
            }
            if case == 8 {
                v = b"QTB1".to_vec();
            }
            let mut r = request(&v, 23);
            match case {
                0 => r.operation = WriteOp::Command,
                1 => r.operation = WriteOp::Reliable,
                2 => r.prepare = true,
                3 => r.offset = 1,
                4 => r.adapter_matches = false,
                5 => r.peer = Address::new([0; 6]),
                6 => g.lock().unwrap().subscribed = false,
                _ => {}
            }
            assert!(g.lock().unwrap().submit(r, &tx, Instant::now()).is_err());
            assert!(*g.lock().unwrap().failed.borrow());
        }
    }

    /// @brief Real transfer limiter honors both its five-second bound and earlier object/session expiry.
    #[tokio::test(start_paused = true)]
    async fn g2_2_att_nested_deadlines() {
        for (outer, expected) in [(20, 5), (3, 3), (0, 0)] {
            let start = Instant::now();
            assert_eq!(
                att_transfer(std::future::pending(), start + Duration::from_secs(outer)).await,
                Err("att_timeout")
            );
            assert_eq!(Instant::now() - start, Duration::from_secs(expected));
        }
    }
    /// @brief First-fragment expiry is clipped to advertising's original absolute 30-second limit.
    #[tokio::test(start_paused = true)]
    async fn g2_2_establishment_deadline() {
        let (g, tx, _rx) = gate();
        let epoch = g.lock().unwrap().epoch;
        let f = framing::fragments(Kind::ClientHello, 0, &[0; 68], 23).unwrap();
        tokio::time::advance(Duration::from_secs(29)).await;
        {
            let mut g = g.lock().unwrap();
            g.submit(
                Write {
                    peer: request(&[], 23).peer,
                    adapter_matches: true,
                    offset: 0,
                    operation: WriteOp::Request,
                    prepare: false,
                    mtu: 23,
                    bytes: &f[0],
                },
                &tx,
                Instant::now(),
            )
            .unwrap();
            assert_eq!(g.rx.deadline(g.ms(g.deadline)), 30_000);
        }
        tokio::time::advance(Duration::from_secs(1)).await;
        let mut g = g.lock().unwrap();
        assert_eq!(g.deadline, epoch + Duration::from_secs(30));
        assert!(
            g.submit(
                Write {
                    peer: request(&[], 23).peer,
                    adapter_matches: true,
                    offset: 0,
                    operation: WriteOp::Request,
                    prepare: false,
                    mtu: 23,
                    bytes: &f[1]
                },
                &tx,
                Instant::now()
            )
            .is_err()
        );
    }
}

#[cfg(test)]
mod dbus_tests;
