//! @file
//! @brief Single native protocol owner; opaque generations never expose pointers to JavaScript.
use jni::{
    JNIEnv,
    objects::{JByteArray, JClass},
    sys::{jint, jlong, jstring},
};
use qtooth_ble_core::{
    Error, Result,
    framing::{self, Kind, Reassembly},
    protocol::{Handshake, Pair, Session},
};
use serde_json::{Value, json};
use std::{
    panic::{AssertUnwindSafe, catch_unwind},
    sync::{Arc, Mutex, OnceLock},
};

/// @brief One foreground attempt; all access is serialized under REGISTRY.
struct Owner {
    handshake: Option<Handshake>,
    session: Option<Session>,
    rx: Reassembly,
    expected: Kind,
    tx_id: u16,
    mtu: u16,
    enclosing: u64,
    started: u64,
    handshake_ms: Option<u64>,
    fragments: u32,
}
impl Owner {
    /// @brief Check packaged provider support before accepting private pairing state.
    fn new(now: u64, mtu: u16) -> Result<Self> {
        framing::budget(mtu)?;
        if openssl::version::number() < 0x30500000 {
            return Err(Error::Crypto);
        }
        use foreign_context::probe;
        probe()?;
        let pair = Pair::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/pair.bin")))?;
        Ok(Self {
            handshake: Some(Handshake::new(Arc::new(pair), false)),
            session: None,
            rx: Reassembly::default(),
            expected: Kind::PiOffer,
            tx_id: 0,
            mtu,
            enclosing: now.saturating_add(30_000),
            started: now,
            handshake_ms: None,
            fragments: 0,
        })
    }
    /// @brief Produce one bounded object; Kotlin awaits each ordinary write completion.
    fn outgoing(&mut self, kind: Kind, body: &[u8]) -> Result<Value> {
        let values = framing::fragments(kind, self.tx_id, body, self.mtu)?;
        self.tx_id = self.tx_id.checked_add(1).ok_or(Error::Exhausted)?;
        Ok(json!({"kind":"status", "phase":"Authenticating", "writes":values}))
    }
    /// @brief Enforce the earlier absolute object/session expiry without JavaScript scheduling.
    fn check(&self, now: u64) -> Result<()> {
        if now >= self.rx.deadline(self.enclosing) {
            Err(Error::Expired)
        } else {
            Ok(())
        }
    }
    /// @brief Advance the peer role; only Session::decrypt can release plaintext.
    fn step(&mut self, op: i32, input: &[u8], now: u64) -> Result<Value> {
        self.check(now)?;
        if op == 0 {
            let hello = self.handshake.as_mut().ok_or(Error::State)?.start()?;
            self.started = now;
            return self.outgoing(Kind::ClientHello, &hello);
        }
        if op == 2 {
            return Ok(json!({"kind":"none"}));
        }
        if op != 1 {
            return Err(Error::State);
        }
        self.fragments = self.fragments.checked_add(1).ok_or(Error::Bounds)?;
        let Some(body) = self
            .rx
            .push(input, self.mtu, self.expected, now, self.enclosing)?
        else {
            return Ok(json!({"kind":"none"}));
        };
        let fragments = std::mem::take(&mut self.fragments);
        match self.expected {
            Kind::PiOffer => {
                let (kind, reply) = self
                    .handshake
                    .as_mut()
                    .ok_or(Error::State)?
                    .receive(Kind::PiOffer, &body)?
                    .ok_or(Error::State)?;
                self.expected = Kind::PiFinish;
                self.outgoing(kind, &reply)
            }
            Kind::PiFinish => {
                let mut handshake = self.handshake.take().ok_or(Error::State)?;
                handshake.receive(Kind::PiFinish, &body)?;
                self.session = Some(handshake.into_session()?);
                self.expected = Kind::Record;
                self.handshake_ms = Some(now.saturating_sub(self.started));
                self.enclosing = now.saturating_add(120_000);
                Ok(
                    json!({"kind":"status", "phase":"Ready · authenticated", "handshakeMs":self.handshake_ms}),
                )
            }
            Kind::Record => {
                let verified = self.session.as_mut().ok_or(Error::State)?.decrypt(&body)?;
                let Some(plaintext) = verified else {
                    return Ok(json!({"kind":"closed", "phase":"Authenticated close"}));
                };
                let text = String::from_utf8(plaintext).map_err(|_| Error::Encoding)?;
                // Metadata is read only after the complete record's tag/sequence validation.
                let sequence =
                    u64::from_be_bytes(body[19..27].try_into().map_err(|_| Error::Encoding)?);
                let n = u16::from_be_bytes([body[27], body[28]]) as usize;
                let preview = body[29..29 + n.min(24)]
                    .iter()
                    .map(|b| format!("{b:02x}"))
                    .collect::<String>();
                Ok(
                    json!({"kind":"verified", "phase":"Ready · authenticated", "plaintext":text,
                    "sequence":sequence.to_string(), "ciphertext":preview, "fragments":fragments,
                    "recordBytes":body.len(), "handshakeMs":self.handshake_ms, "mtu":self.mtu}),
                )
            }
            _ => Err(Error::State),
        }
    }
}
/// @brief Provider capability probe using safe OpenSSL context bindings.
mod foreign_context {
    use super::*;
    /// @brief Name-based provider lookup/keygen verifies packaged ML-KEM availability.
    pub fn probe() -> Result<()> {
        let mut seed = zeroize::Zeroizing::new([0u8; 64]);
        openssl::rand::rand_bytes(&mut seed[..]).map_err(|_| Error::Crypto)?;
        let _key = openssl::pkey::PKey::private_key_from_seed(
            None,
            openssl::pkey::KeyType::ML_KEM_768,
            None,
            &seed[..],
        )
        .map_err(|_| Error::Crypto)?;
        Ok(())
    }
}
/// @brief At most one active owner; monotonically increasing handles reject stale callbacks.
#[derive(Default)]
struct Registry {
    generation: i64,
    owner: Option<Owner>,
}
static REGISTRY: OnceLock<Mutex<Registry>> = OnceLock::new();
/// @brief Process-local ownership registry; poisoned state never resumes a session.
fn registry() -> &'static Mutex<Registry> {
    REGISTRY.get_or_init(|| Mutex::new(Registry::default()))
}
/// @brief Allocate a fresh generation on the serialized Kotlin worker; zero means safe init failure.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_quantumtooth_proof_NativeCore_create(
    _env: JNIEnv,
    _class: JClass,
    now: jlong,
    mtu: jint,
) -> jlong {
    catch_unwind(AssertUnwindSafe(|| {
        let mut r = registry().lock().map_err(|_| Error::State)?;
        r.owner = None;
        r.generation = r.generation.checked_add(1).ok_or(Error::Exhausted)?;
        if now < 0 || !(23..=517).contains(&mtu) {
            return Err(Error::Bounds);
        }
        r.owner = Some(Owner::new(now as u64, mtu as u16)?);
        Ok::<_, Error>(r.generation)
    }))
    .ok()
    .and_then(std::result::Result::ok)
    .unwrap_or(0)
}
/// @brief Copy one JNI array (max 512 bytes); contain panics and return only sanitized JSON.
/// @note No borrowed JNI memory escapes; failures drop all owned active state before returning.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_quantumtooth_proof_NativeCore_step(
    env: JNIEnv,
    _class: JClass,
    handle: jlong,
    op: jint,
    input: JByteArray,
    now: jlong,
) -> jstring {
    let result = catch_unwind(AssertUnwindSafe(|| -> Result<Value> {
        let mut r = registry().lock().map_err(|_| Error::State)?;
        if handle <= 0 || handle != r.generation {
            return Err(Error::State);
        }
        let result = (|| {
            if now < 0 || env.get_array_length(&input).map_err(|_| Error::Encoding)? > 512 {
                return Err(Error::Bounds);
            }
            let bytes = env
                .convert_byte_array(&input)
                .map_err(|_| Error::Encoding)?;
            r.owner
                .as_mut()
                .ok_or(Error::State)?
                .step(op, &bytes, now as u64)
        })();
        if result.is_err() || result.as_ref().is_ok_and(|v| v["kind"] == "closed") {
            r.owner = None;
        }
        result
    }));
    let value = match result {
        Ok(Ok(value)) => value,
        Ok(Err(error)) => json!({"kind":"failure", "phase":match error {
            Error::Expired => "Session expired · scan again",
            Error::Crypto => "Native crypto unavailable · rebuild the app",
            _ => "Authentication/protocol failed · scan again",
        }}),
        Err(_) => {
            // Poisoned registries are permanently unusable until process restart.
            let mut r = registry()
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            r.owner = None;
            json!({"kind":"failure", "phase":"Native failure · restart the app"})
        }
    };
    env.new_string(value.to_string())
        .map(|s| s.into_raw())
        .unwrap_or(std::ptr::null_mut())
}
/// @brief Idempotently discard the matching generation; stale teardown cannot destroy a newer owner.
#[unsafe(no_mangle)]
pub extern "system" fn Java_org_quantumtooth_proof_NativeCore_destroy(
    _env: JNIEnv,
    _class: JClass,
    handle: jlong,
) {
    let _ = catch_unwind(AssertUnwindSafe(|| {
        let mut r = registry()
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if handle == r.generation {
            r.owner = None;
        }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    /// @brief Decode the adapter's outgoing fragments with the actual core receiver.
    fn collect(value: &Value, rx: &mut Reassembly, kind: Kind, mtu: u16) -> Vec<u8> {
        let mut body = None;
        for fragment in value["writes"].as_array().unwrap() {
            let bytes = fragment
                .as_array()
                .unwrap()
                .iter()
                .map(|v| v.as_u64().unwrap() as u8)
                .collect::<Vec<_>>();
            body = rx.push(&bytes, mtu, kind, 1, 30_000).unwrap().or(body);
        }
        body.unwrap()
    }
    /// @brief Feed one real peer object through the production adapter's framing/state path.
    fn receive(owner: &mut Owner, kind: Kind, id: u16, body: &[u8]) -> Result<Value> {
        let mut output = json!({});
        for fragment in framing::fragments(kind, id, body, owner.mtu)? {
            output = owner.step(1, &fragment, 2)?;
        }
        Ok(output)
    }
    /// @brief Establish the adapter against the real Pi-role core without BLE simulation in the app.
    fn establish(mtu: u16) -> (Owner, Session) {
        let mut owner = Owner::new(0, mtu).unwrap();
        let pair = Pair::from_bytes(include_bytes!(concat!(env!("OUT_DIR"), "/pair.bin"))).unwrap();
        let mut pi = Handshake::new(Arc::new(pair), true);
        let mut rx = Reassembly::default();
        let hello = collect(
            &owner.step(0, &[], 0).unwrap(),
            &mut rx,
            Kind::ClientHello,
            mtu,
        );
        let (_, offer) = pi.receive(Kind::ClientHello, &hello).unwrap().unwrap();
        let response = receive(&mut owner, Kind::PiOffer, 0, &offer).unwrap();
        let finish = collect(&response, &mut rx, Kind::PeerFinish, mtu);
        let (_, confirmation) = pi.receive(Kind::PeerFinish, &finish).unwrap().unwrap();
        pi.finish_sent().unwrap();
        assert_eq!(
            receive(&mut owner, Kind::PiFinish, 1, &confirmation).unwrap()["phase"],
            "Ready · authenticated"
        );
        (owner, pi.into_session().unwrap())
    }
    /// @brief Verify real adapter results at both MTUs, retained sequences, authenticated close and expiry.
    #[test]
    fn adapter_retained_records_and_bounds() {
        for mtu in [23, 517] {
            let (mut owner, mut pi) = establish(mtu);
            for (sequence, text) in ["Hello, world!", "Second public message"]
                .iter()
                .enumerate()
            {
                let record = pi.encrypt(text.as_bytes(), false).unwrap();
                let event =
                    receive(&mut owner, Kind::Record, 2 + sequence as u16, &record).unwrap();
                assert_eq!(event["kind"], "verified");
                assert_eq!(event["plaintext"], *text);
                assert_eq!(event["sequence"], sequence.to_string());
                assert_eq!(event["recordBytes"], record.len());
                assert_eq!(
                    event["fragments"],
                    framing::fragments(Kind::Record, 2, &record, mtu)
                        .unwrap()
                        .len()
                );
            }
            let close = pi.encrypt(&[], true).unwrap();
            assert_eq!(
                receive(&mut owner, Kind::Record, 4, &close).unwrap()["kind"],
                "closed"
            );
            let (mut expired, _) = establish(mtu);
            assert!(matches!(expired.step(2, &[], 120_002), Err(Error::Expired)));
        }
    }
    /// @brief A tampered tag never reaches the JSON verified-result boundary.
    #[test]
    fn adapter_rejects_tampered_record() {
        let (mut owner, mut pi) = establish(23);
        let mut record = pi.encrypt(b"Public rejection check", false).unwrap();
        *record.last_mut().unwrap() ^= 1;
        assert!(matches!(
            receive(&mut owner, Kind::Record, 2, &record),
            Err(Error::Authentication)
        ));
    }
}
