//! @file
//! @brief Exercise production GATT callbacks through BlueR's real D-Bus parser.

use super::*;
use dbus::{
    arg::{PropMap, Variant},
    channel::MatchingReceiver,
    message::MatchRule,
};
use dbus_crossroads::Crossroads;
use std::{
    collections::HashMap,
    io::{BufRead, BufReader},
    process::{Child, Command, Stdio},
};

/// @brief Kill and reap the private test bus on success or assertion failure.
struct PrivateBus(Child);

impl Drop for PrivateBus {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// @brief Isolate bus environment in a child process, avoiding global env mutation.
#[test]
fn write_value_metadata_reaches_gate() {
    if std::env::var_os("QTOOTH_PRIVATE_DBUS_TEST").is_some() {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(async {
                timeout(Duration::from_secs(10), exercise_write_value())
                    .await
                    .unwrap();
            });
        return;
    }
    let mut bus = PrivateBus(
        Command::new("dbus-daemon")
            .args(["--session", "--nofork", "--print-address=1"])
            .stdout(Stdio::piped())
            .spawn()
            .expect("install dbus-daemon for boundary tests"),
    );
    let mut address = String::new();
    BufReader::new(bus.0.stdout.take().unwrap())
        .read_line(&mut address)
        .unwrap();
    assert!(!address.trim().is_empty(), "private bus failed to start");
    let child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "probe::dbus_tests::write_value_metadata_reaches_gate",
            "--nocapture",
        ])
        .env("QTOOTH_PRIVATE_DBUS_TEST", "1")
        .env("DBUS_SYSTEM_BUS_ADDRESS", address.trim())
        .env("DBUS_SESSION_BUS_ADDRESS", address.trim())
        .output()
        .unwrap();
    assert!(
        child.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&child.stdout),
        String::from_utf8_lossy(&child.stderr)
    );
}

/// @brief Register the real application against a fake manager, then call RX over D-Bus.
async fn exercise_write_value() {
    let (io, connection) = dbus_tokio::connection::new_session_sync().unwrap();
    let io_task = tokio::spawn(io);
    connection
        .request_name("org.bluez", false, true, false)
        .await
        .unwrap();
    let (registered, mut registrations) = mpsc::channel(1);
    let mut cr = Crossroads::new();
    let manager = cr.register("org.bluez.GattManager1", |b| {
        b.method(
            "RegisterApplication",
            ("application", "options"),
            (),
            |ctx,
             sender: &mut mpsc::Sender<(String, dbus::Path<'static>)>,
             (path, _): (dbus::Path<'static>, PropMap)| {
                sender
                    .try_send((ctx.message().sender().unwrap().to_string(), path))
                    .unwrap();
                Ok(())
            },
        );
        b.method(
            "UnregisterApplication",
            ("application",),
            (),
            |_,
             _: &mut mpsc::Sender<(String, dbus::Path<'static>)>,
             (_,): (dbus::Path<'static>,)| Ok(()),
        );
    });
    cr.insert("/org/bluez/hci0", &[manager], registered);
    connection.start_receive(
        MatchRule::new_method_call(),
        Box::new(move |msg, conn| {
            cr.handle_message(msg, conn).unwrap();
            true
        }),
    );
    let session = bluer::Session::new().await.unwrap();
    let peer = Address::new([0, 0, 0, 0, 0, 1]);
    let mut state = Gate::new(Instant::now());
    state.peer = Some(peer);
    state.subscribed = true;
    assert!(state.update_readiness(Instant::now()));
    let gate = Arc::new(Mutex::new(state));
    let (writes, mut received) = mpsc::channel(1);
    let (subscriptions, _subscribed) = mpsc::channel(1);
    let app = session
        .adapter("hci0")
        .unwrap()
        .serve_gatt_application(application(gate, writes, subscriptions, "hci0".into()))
        .await
        .unwrap();
    let (destination, path) = registrations.recv().await.unwrap();
    let proxy = dbus::nonblock::Proxy::new(
        destination,
        path,
        Duration::from_secs(2),
        connection.clone(),
    );
    let (objects,): (HashMap<dbus::Path<'static>, HashMap<String, PropMap>>,) = proxy
        .method_call(
            "org.freedesktop.DBus.ObjectManager",
            "GetManagedObjects",
            (),
        )
        .await
        .unwrap();
    let rx_path = objects
        .iter()
        .find_map(|(path, interfaces)| {
            let props = interfaces.get("org.bluez.GattCharacteristic1")?;
            (dbus::arg::prop_cast::<String>(props, "UUID")? == &RX.to_string())
                .then(|| path.clone())
        })
        .unwrap();
    let rx = dbus::nonblock::Proxy::new(
        proxy.destination.clone(),
        rx_path,
        Duration::from_secs(2),
        connection.clone(),
    );
    for (kind, prepare) in [
        (None, false),
        (Some("command"), false),
        (Some("reliable"), false),
        (Some("request"), true),
    ] {
        let result: Result<(), dbus::Error> = rx
            .method_call(
                "org.bluez.GattCharacteristic1",
                "WriteValue",
                (TOKEN.to_vec(), options(kind, prepare)),
            )
            .await;
        assert_eq!(
            result.unwrap_err().name(),
            Some("org.bluez.Error.NotSupported")
        );
        assert!(received.try_recv().is_err());
    }
    let result: Result<(), dbus::Error> = rx
        .method_call(
            "org.bluez.GattCharacteristic1",
            "WriteValue",
            (TOKEN.to_vec(), options(Some("request"), false)),
        )
        .await;
    result.unwrap();
    assert_eq!(received.recv().await.unwrap().mtu, 23);
    assert!(received.try_recv().is_err());
    drop(app);
    drop(session);
    io_task.abort();
    let _ = io_task.await;
}

/// @brief Supply the pinned BlueZ server's options, optionally adding its missing type.
fn options(kind: Option<&str>, prepare: bool) -> PropMap {
    let mut options = PropMap::new();
    options.insert(
        "device".into(),
        Variant(Box::new(
            dbus::Path::new("/org/bluez/hci0/dev_00_00_00_00_00_01").unwrap(),
        )),
    );
    options.insert("link".into(), Variant(Box::new("LE".to_string())));
    options.insert("mtu".into(), Variant(Box::new(23_u16)));
    options.insert("prepare-authorize".into(), Variant(Box::new(prepare)));
    if let Some(kind) = kind {
        options.insert("type".into(), Variant(Box::new(kind.to_string())));
    }
    options
}
