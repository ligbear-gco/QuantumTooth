//! @file
//! @brief Native real-crypto experiment with independent encoding and failure checks.
use super::*;
use crate::framing::{self, Reassembly};
/// @brief Disposable per-run pair input; never logged or used as a production fallback.
fn pair() -> Arc<Pair> {
    let mut b = Zeroizing::new([0; 64]);
    openssl::rand::rand_bytes(&mut b[..]).unwrap();
    Arc::new(Pair::from_bytes(&b[..]).unwrap())
}
/// @brief Pass one object through the production fragment encoder and fixed-storage receiver.
fn transfer(k: Kind, id: u16, b: &[u8], mtu: u16, rx: &mut Reassembly) -> Vec<u8> {
    let mut out = None;
    for v in framing::fragments(k, id, b, mtu).unwrap() {
        assert!(v.len() <= framing::budget(mtu).unwrap());
        out = rx.push(&v, mtu, k, 1, 30_000).unwrap();
    }
    out.unwrap()
}
/// @brief Confirm both endpoints through fragmented handshake objects.
fn establish(mtu: u16) -> (Session, Session) {
    let p = pair();
    let mut pi = Handshake::new(p.clone(), true);
    let mut peer = Handshake::new(p, false);
    let (mut pr, mut rr) = (Reassembly::default(), Reassembly::default());
    let h = transfer(Kind::ClientHello, 0, &peer.start().unwrap(), mtu, &mut pr);
    let (k, o) = pi.receive(Kind::ClientHello, &h).unwrap().unwrap();
    let o = transfer(k, 0, &o, mtu, &mut rr);
    let (k, f) = peer.receive(k, &o).unwrap().unwrap();
    let f = transfer(k, 1, &f, mtu, &mut pr);
    let (k, f) = pi.receive(k, &f).unwrap().unwrap();
    let f = transfer(k, 1, &f, mtu, &mut rr);
    peer.receive(k, &f).unwrap();
    pi.finish_sent().unwrap();
    (pi.into_session().unwrap(), peer.into_session().unwrap())
}
/// @brief Reproducible learner experiment; transport completion requires no application reply.
#[test]
fn g2_2_real_crypto_demo() {
    for mtu in [23, 517] {
        let (mut pi, mut peer) = establish(mtu);
        let mut rx = Reassembly::default();
        let r = pi.encrypt(b"Hello, world!", false).unwrap();
        let r = transfer(Kind::Record, 0, &r, mtu, &mut rx);
        assert_eq!(peer.decrypt(&r).unwrap().unwrap(), b"Hello, world!");
        println!(
            "g2_2 native simulated_mtu={mtu} decrypted=Hello, world! delivery=unconfirmed no_application_ack=true"
        );
        assert_eq!(pi.next_sequence(), 1);
        let next = pi.encrypt(&[b'x'; 512], false).unwrap();
        assert_eq!(next.len(), 557);
        assert_ne!(&r[19..27], &next[19..27]);
        assert_eq!(peer.decrypt(&next).unwrap().unwrap().len(), 512);
        let reverse = peer.encrypt(b"reverse", false).unwrap();
        assert_eq!(pi.decrypt(&reverse).unwrap().unwrap(), b"reverse");
        let close = pi.encrypt(&[], true).unwrap();
        assert_eq!(close.len(), 45);
        assert_eq!(peer.decrypt(&close).unwrap(), None);
        assert!(pi.encrypt(b"again", false).is_err());
        assert!(peer.decrypt(&r).is_err());
    }
}
/// @brief Decode independent literal hex fixtures.
fn hex(s: &str) -> Vec<u8> {
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).unwrap())
        .collect()
}
/// @brief Compare to Python fixtures rather than only testing shared-code round trips.
#[test]
fn g2_2_canonical_fixtures() {
    let mut b = [0; 64];
    b[32..48].copy_from_slice(&(0..16).collect::<Vec<u8>>());
    b[48..].copy_from_slice(&(16..32).collect::<Vec<u8>>());
    let p = Pair::from_bytes(&b).unwrap();
    let n: [u8; 32] = (32..64).collect::<Vec<u8>>().try_into().unwrap();
    let h = hello(&p, &n);
    assert_eq!(
        h,
        hex(
            "01000101000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f"
        )
    );
    assert_eq!(
        framing::fragments(Kind::ClientHello, 0, &h, 23).unwrap()[0],
        hex("0101000000440000010001010001020304050607")
    );
    let mut o = vec![2];
    o.extend(64..96);
    o.extend([0xa5; 1184]);
    let t = transcript(&h, &o, &[0x5a; 1088]);
    assert_eq!(t.len(), 2396);
    assert_eq!(
        sha256(&t).to_vec(),
        hex("2da73c3b87a5d16e9c6a6a7734921a314007c636733bbc78cd84e70b29bc2eaf")
    );
    let id: [u8; 16] = (0..16).collect::<Vec<u8>>().try_into().unwrap();
    assert_eq!(
        record_header(1, 1, &id, 0x0102030405060708, 13),
        hex("010101000102030405060708090a0b0c0d0e0f0102030405060708000d")
    );
    assert_eq!(
        record_nonce(&[0xa0, 0xa1, 0xa2, 0xa3], 0x0102030405060708).to_vec(),
        hex("a0a1a2a30102030405060708")
    );
    for (k, n, small, large) in [
        (Kind::ClientHello, 68, 6, 1),
        (Kind::PiOffer, 1249, 105, 3),
        (Kind::PeerFinish, 1152, 96, 3),
        (Kind::PiFinish, 32, 3, 1),
        (Kind::Record, 58, 5, 1),
        (Kind::Record, 557, 47, 2),
    ] {
        assert_eq!(
            framing::fragments(k, 0, &vec![0; n], 23).unwrap().len(),
            small
        );
        assert_eq!(
            framing::fragments(k, 0, &vec![0; n], 517).unwrap().len(),
            large
        );
    }
}
/// @brief Reject changed transcript/identity/version/suite and wrong PSK/confirmation.
#[test]
fn g2_2_handshake_rejections() {
    for i in [0, 1, 2, 3, 4, 20, 36, 67] {
        let p = pair();
        let mut pi = Handshake::new(p.clone(), true);
        let mut peer = Handshake::new(p, false);
        let mut h = peer.start().unwrap();
        h[i] ^= 1;
        if let Ok(Some((k, o))) = pi.receive(Kind::ClientHello, &h) {
            assert!(peer.receive(k, &o).is_err());
        }
        assert!(pi.into_session().is_err());
    }
    let a = pair();
    let mut b = Zeroizing::new([0; 64]);
    b[..32].copy_from_slice(&a.psk[..]);
    b[0] ^= 1;
    b[32..48].copy_from_slice(&a.peer);
    b[48..].copy_from_slice(&a.pi);
    let mut pi = Handshake::new(a, true);
    let mut peer = Handshake::new(Arc::new(Pair::from_bytes(&b[..]).unwrap()), false);
    let h = peer.start().unwrap();
    let (k, o) = pi.receive(Kind::ClientHello, &h).unwrap().unwrap();
    assert!(peer.receive(k, &o).is_err());
    for stage in 0..4 {
        let p = pair();
        let mut pi = Handshake::new(p.clone(), true);
        let mut peer = Handshake::new(p, false);
        let h = peer.start().unwrap();
        let (k, mut o) = pi.receive(Kind::ClientHello, &h).unwrap().unwrap();
        if stage == 0 {
            o[100] ^= 1;
            assert!(peer.receive(k, &o).is_err());
            continue;
        }
        let (k, mut f) = peer.receive(k, &o).unwrap().unwrap();
        if stage == 1 {
            f[1088] ^= 1;
            assert!(pi.receive(k, &f).is_err());
            continue;
        }
        if stage == 2 {
            f[1120] ^= 1;
            assert!(pi.receive(k, &f).is_err());
            continue;
        }
        let (k, mut f) = pi.receive(k, &f).unwrap().unwrap();
        f[0] ^= 1;
        assert!(peer.receive(k, &f).is_err());
        assert!(peer.into_session().is_err());
        assert!(pi.into_session().is_err());
    }
}
/// @brief Tamper, replay, wrong direction/session and exhaustion always destroy keys.
#[test]
fn g2_2_record_rejections() {
    for i in [0, 1, 2, 3, 18, 19, 26, 27, 28, 29, 57] {
        let (mut pi, mut peer) = establish(517);
        let mut r = pi.encrypt(b"Hello, world!", false).unwrap();
        r[i] ^= 1;
        assert!(peer.decrypt(&r).is_err());
        assert!(peer.encrypt(b"closed", false).is_err());
    }
    let (mut pi, mut peer) = establish(517);
    let r = pi.encrypt(b"replay", false).unwrap();
    assert!(peer.decrypt(&r).is_ok());
    assert!(peer.decrypt(&r).is_err());
    let (mut pi, mut peer) = establish(517);
    let r = pi.encrypt(b"direction", false).unwrap();
    assert!(pi.decrypt(&r).is_err());
    let (_, mut other) = establish(517);
    assert!(other.decrypt(&r).is_err());
    peer.send = u64::MAX;
    assert_eq!(
        peer.encrypt(b"exhaust", false).err(),
        Some(Error::Exhausted)
    );
    assert!(peer.material.is_none());
}
/// @brief Check immutable object deadlines and malformed fragment rejection without storage growth.
#[test]
fn g2_2_framing_rejections_and_deadlines() {
    let f = framing::fragments(Kind::ClientHello, 0, &[0; 68], 23).unwrap();
    for mode in 0..10 {
        let mut rx = Reassembly::default();
        let mut bad = f[0].clone();
        match mode {
            0 => {
                bad[4] = 0xff;
                bad[5] = 0xff;
            }
            1 => bad[7] = 1,
            2 => bad[3] = 1,
            3 => bad[1] = 3,
            4 => bad[0] = 2,
            5 => bad.truncate(8),
            6 => bad.extend([0; 30]),
            7 => {
                rx.push(&f[0], 23, Kind::ClientHello, 0, 30_000).unwrap();
            }
            8 => {
                rx.push(&f[0], 23, Kind::ClientHello, 0, 30_000).unwrap();
                bad = f[2].clone();
            }
            _ => {
                rx.push(&f[0], 23, Kind::ClientHello, 0, 30_000).unwrap();
                bad = f[1].clone();
                bad[4] = 1;
            }
        }
        assert!(rx.push(&bad, 23, Kind::ClientHello, 1, 30_000).is_err());
        assert!(rx.push(&f[0], 23, Kind::ClientHello, 2, 30_000).is_err());
    }
    let mut rx = Reassembly::default();
    rx.push(&f[0], 23, Kind::ClientHello, 100, 30_000).unwrap();
    assert_eq!(rx.deadline(30_000), 10_100);
    rx.push(&f[1], 23, Kind::ClientHello, 10_099, 30_000)
        .unwrap();
    assert_eq!(rx.deadline(30_000), 10_100);
    assert_eq!(
        rx.push(&f[2], 23, Kind::ClientHello, 10_100, 30_000),
        Err(Error::Expired)
    );
    let mut rx = Reassembly::default();
    assert_eq!(
        rx.push(&f[0], 23, Kind::ClientHello, 30_000, 30_000),
        Err(Error::Expired)
    );
}
