//! @file
//! @brief Canonical PSK-authenticated KEM establishment and one-way AEAD records.
use crate::{Error, Result, crypto, framing::Kind, kem};
use openssl::{
    pkey::{PKey, Private},
    sha::sha256,
    symm::{Cipher, Crypter, Mode},
};
use std::sync::Arc;
use zeroize::Zeroizing;
/// @brief Frozen transcript domain, without NUL or separators beyond those written.
pub const DOMAIN: &[u8] = b"QTOOTH/G2.2/v1";
/// @brief Private pairing material, deliberately neither Debug nor Clone.
pub struct Pair {
    psk: Zeroizing<[u8; 32]>,
    pub peer: [u8; 16],
    pub pi: [u8; 16],
}
impl Pair {
    /// @brief Take ownership of exactly 64 provisioned bytes: PSK, peer identity, Pi identity.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 {
            return Err(Error::Bounds);
        }
        let mut psk = Zeroizing::new([0; 32]);
        psk.copy_from_slice(&bytes[..32]);
        Ok(Self {
            psk,
            peer: bytes[32..48].try_into().map_err(|_| Error::Encoding)?,
            pi: bytes[48..64].try_into().map_err(|_| Error::Encoding)?,
        })
    }
}
/// @brief Canonical kind/length envelope, also used for the untagged transcript cores.
pub fn envelope(kind: Kind, body: &[u8]) -> Vec<u8> {
    let mut v = vec![kind as u8];
    v.extend_from_slice(&(body.len() as u16).to_be_bytes());
    v.extend_from_slice(body);
    v
}
/// @brief Bind both identities, version, suite and peer role in fixed order.
pub fn hello(pair: &Pair, nonce: &[u8; 32]) -> Vec<u8> {
    let mut v = vec![1, 0, 1, 1];
    v.extend_from_slice(&pair.peer);
    v.extend_from_slice(&pair.pi);
    v.extend_from_slice(nonce);
    v
}
/// @brief Encode the complete authenticated exchange, excluding HMACs and confirmations.
pub fn transcript(hello: &[u8], offer: &[u8], response: &[u8]) -> Vec<u8> {
    let mut t = DOMAIN.to_vec();
    t.extend(envelope(Kind::ClientHello, hello));
    t.extend(envelope(Kind::PiOffer, offer));
    t.extend(envelope(Kind::PeerFinish, response));
    t
}
/// @brief Directional secrets; all owned outputs zeroize when discarded.
struct Material {
    pi_key: Zeroizing<[u8; 32]>,
    peer_key: Zeroizing<[u8; 32]>,
    pi_nonce: Zeroizing<[u8; 4]>,
    peer_nonce: Zeroizing<[u8; 4]>,
    pi_confirm: Zeroizing<[u8; 32]>,
    peer_confirm: Zeroizing<[u8; 32]>,
    id: Zeroizing<[u8; 16]>,
}
impl Material {
    /// @brief Derive independent outputs using the registry's exact role/direction labels.
    fn new(secret: &[u8], hash: &[u8; 32]) -> Result<Self> {
        Ok(Self {
            pi_key: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/pi-to-peer/key")?,
            peer_key: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/peer-to-pi/key")?,
            pi_nonce: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/pi-to-peer/nonce")?,
            peer_nonce: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/peer-to-pi/nonce")?,
            pi_confirm: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/pi-confirm")?,
            peer_confirm: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/peer-confirm")?,
            id: crypto::derive(secret, hash, b"QTOOTH/G2.2/v1/session-id")?,
        })
    }
}
/// @brief Ephemeral key lifetime follows the handshake; errors consume and discard state.
enum State {
    Initial,
    WaitingOffer {
        hello: Vec<u8>,
    },
    WaitingPeer {
        hello: Vec<u8>,
        offer: Vec<u8>,
        key: PKey<Private>,
    },
    WaitingPi {
        material: Material,
        hash: [u8; 32],
        confirmation: [u8; 32],
    },
    FinishPending(Material),
    Ready(Material),
    Closed,
}
/// @brief One configured endpoint; plaintext is impossible before mutual confirmation.
pub struct Handshake {
    pair: Arc<Pair>,
    pi: bool,
    state: State,
}
impl Handshake {
    /// @brief Create a fresh role-specific handshake; caller enforces its absolute 30-second limit.
    pub fn new(pair: Arc<Pair>, pi: bool) -> Self {
        Self {
            pair,
            pi,
            state: State::Initial,
        }
    }
    /// @brief Peer starts with fresh nonce and the configured identities.
    pub fn start(&mut self) -> Result<Vec<u8>> {
        let old = std::mem::replace(&mut self.state, State::Closed);
        if self.pi || !matches!(old, State::Initial) {
            return Err(Error::State);
        }
        let h = hello(&self.pair, &crypto::nonce()?);
        self.state = State::WaitingOffer { hello: h.clone() };
        Ok(h)
    }
    /// @brief Process one exact-length object; every error leaves a closed, secret-free handshake.
    pub fn receive(&mut self, kind: Kind, body: &[u8]) -> Result<Option<(Kind, Vec<u8>)>> {
        let old = std::mem::replace(&mut self.state, State::Closed);
        if !kind.permits(body.len()) {
            return Err(Error::Encoding);
        }
        match (old, kind) {
            (State::Initial, Kind::ClientHello) if self.pi => {
                if body[..36] != hello(&self.pair, &[0; 32])[..36] {
                    return Err(Error::Authentication);
                }
                let key = kem::generate()?;
                let mut offer = vec![2];
                offer.extend_from_slice(&crypto::nonce()?);
                offer.extend(kem::public(&key)?);
                let tag = crypto::mac(
                    &self.pair.psk[..],
                    &[
                        DOMAIN,
                        b"pi-offer",
                        &envelope(Kind::ClientHello, body),
                        &envelope(Kind::PiOffer, &offer),
                    ],
                )?;
                let mut out = offer.clone();
                out.extend(tag);
                self.state = State::WaitingPeer {
                    hello: body.to_vec(),
                    offer,
                    key,
                };
                Ok(Some((Kind::PiOffer, out)))
            }
            (State::WaitingOffer { hello }, Kind::PiOffer) => {
                if body[0] != 2 {
                    return Err(Error::Encoding);
                }
                let offer = &body[..1217];
                crypto::verify(
                    &crypto::mac(
                        &self.pair.psk[..],
                        &[
                            DOMAIN,
                            b"pi-offer",
                            &envelope(Kind::ClientHello, &hello),
                            &envelope(Kind::PiOffer, offer),
                        ],
                    )?,
                    &body[1217..],
                )?;
                let (ct, secret) = kem::encapsulate(&offer[33..])?;
                let t = transcript(&hello, offer, &ct);
                let hash = sha256(&t);
                let material = Material::new(&secret[..], &hash)?;
                let confirmation =
                    crypto::mac(&material.peer_confirm[..], &[b"peer-finished", &hash])?;
                let mut out = ct;
                out.extend(crypto::mac(&self.pair.psk[..], &[b"peer-auth", &t])?);
                out.extend(confirmation);
                self.state = State::WaitingPi {
                    material,
                    hash,
                    confirmation,
                };
                Ok(Some((Kind::PeerFinish, out)))
            }
            (State::WaitingPeer { hello, offer, key }, Kind::PeerFinish) => {
                let t = transcript(&hello, &offer, &body[..1088]);
                let hash = sha256(&t);
                crypto::verify(
                    &crypto::mac(&self.pair.psk[..], &[b"peer-auth", &t])?,
                    &body[1088..1120],
                )?;
                let secret = kem::decapsulate(&key, &body[..1088])?;
                let material = Material::new(&secret[..], &hash)?;
                crypto::verify(
                    &crypto::mac(&material.peer_confirm[..], &[b"peer-finished", &hash])?,
                    &body[1120..],
                )?;
                let finish = crypto::mac(
                    &material.pi_confirm[..],
                    &[b"pi-finished", &hash, &body[1120..]],
                )?;
                self.state = State::FinishPending(material);
                Ok(Some((Kind::PiFinish, finish.to_vec())))
            }
            (
                State::WaitingPi {
                    material,
                    hash,
                    confirmation,
                },
                Kind::PiFinish,
            ) => {
                crypto::verify(
                    &crypto::mac(
                        &material.pi_confirm[..],
                        &[b"pi-finished", &hash, &confirmation],
                    )?,
                    body,
                )?;
                self.state = State::Ready(material);
                Ok(None)
            }
            _ => Err(Error::State),
        }
    }
    /// @brief Pi commits keys only after its finish indication's transport completion.
    pub fn finish_sent(&mut self) -> Result<()> {
        match std::mem::replace(&mut self.state, State::Closed) {
            State::FinishPending(m) => {
                self.state = State::Ready(m);
                Ok(())
            }
            _ => Err(Error::State),
        }
    }
    /// @brief Consume confirmed handshake, dropping ephemeral keys and confirmation keys.
    pub fn into_session(mut self) -> Result<Session> {
        match std::mem::replace(&mut self.state, State::Closed) {
            State::Ready(mut m) => {
                use zeroize::Zeroize;
                m.pi_confirm.zeroize();
                m.peer_confirm.zeroize();
                Ok(Session {
                    material: Some(m),
                    pi: self.pi,
                    send: 0,
                    receive: 0,
                })
            }
            _ => Err(Error::State),
        }
    }
}
/// @brief Public authenticated metadata; this complete header is AEAD associated data.
pub fn record_header(
    kind: u8,
    direction: u8,
    id: &[u8; 16],
    sequence: u64,
    length: usize,
) -> Vec<u8> {
    let mut v = vec![1, kind, direction];
    v.extend_from_slice(id);
    v.extend_from_slice(&sequence.to_be_bytes());
    v.extend_from_slice(&(length as u16).to_be_bytes());
    v
}
/// @brief Nonce is the directional prefix followed by a nonrepeating big-endian sequence.
pub fn record_nonce(prefix: &[u8; 4], sequence: u64) -> [u8; 12] {
    let mut n = [0; 12];
    n[..4].copy_from_slice(prefix);
    n[4..].copy_from_slice(&sequence.to_be_bytes());
    n
}
/// @brief Confirmed directional traffic state; errors/close irreversibly discard all key material.
pub struct Session {
    material: Option<Material>,
    pi: bool,
    send: u64,
    receive: u64,
}
impl Session {
    /// @brief Public local sequence observation, safe for tests and count-only reporting.
    pub fn next_sequence(&self) -> u64 {
        self.send
    }
    /// @brief Immediately destroy all owned keys/counters on any external failure.
    pub fn abort(&mut self) {
        self.material = None;
        self.send = 0;
        self.receive = 0;
    }
    /// @brief Encrypt complete UTF-8 data or an empty close; reserve sequence before EVP work.
    pub fn encrypt(&mut self, text: &[u8], close: bool) -> Result<Vec<u8>> {
        let result = self.seal(text, close);
        if result.is_err() || close {
            self.abort()
        };
        result
    }
    /// @brief Internal sealing; outer method invalidates on error and never retries.
    fn seal(&mut self, text: &[u8], close: bool) -> Result<Vec<u8>> {
        if (close && !text.is_empty())
            || (!close && (!(1..=512).contains(&text.len()) || std::str::from_utf8(text).is_err()))
        {
            return Err(Error::Bounds);
        }
        let m = self.material.as_ref().ok_or(Error::State)?;
        if self.send == u64::MAX {
            return Err(Error::Exhausted);
        }
        let seq = self.send;
        self.send += 1;
        let direction = if self.pi { 1 } else { 2 };
        let (key, prefix) = if self.pi {
            (&m.pi_key, &m.pi_nonce)
        } else {
            (&m.peer_key, &m.peer_nonce)
        };
        let mut out = record_header(if close { 2 } else { 1 }, direction, &m.id, seq, text.len());
        let nonce = record_nonce(prefix, seq);
        let mut tag = [0; 16];
        let ct = openssl::symm::encrypt_aead(
            Cipher::aes_256_gcm(),
            &key[..],
            Some(&nonce),
            &out,
            text,
            &mut tag,
        )
        .map_err(|_| Error::Crypto)?;
        out.extend(ct);
        out.extend(tag);
        Ok(out)
    }
    /// @brief Verify metadata, strict sequence and GCM tag before releasing any plaintext.
    /// @return None for a valid close; all rejection paths destroy this session.
    pub fn decrypt(&mut self, record: &[u8]) -> Result<Option<Vec<u8>>> {
        let result = self.open(record);
        if result.is_err() || matches!(result, Ok(None)) {
            self.abort()
        };
        result
    }
    /// @brief Internal opening uses zeroizing scratch; EVP may write before final tag verification.
    fn open(&mut self, r: &[u8]) -> Result<Option<Vec<u8>>> {
        let m = self.material.as_ref().ok_or(Error::State)?;
        if !(45..=557).contains(&r.len()) {
            return Err(Error::Bounds);
        }
        let n = u16::from_be_bytes([r[27], r[28]]) as usize;
        let seq = u64::from_be_bytes(r[19..27].try_into().map_err(|_| Error::Encoding)?);
        let direction = if self.pi { 2 } else { 1 };
        if r[0] != 1
            || r[2] != direction
            || r[3..19] != m.id[..]
            || seq != self.receive
            || self.receive == u64::MAX
            || r.len() != 45 + n
            || !matches!((r[1], n), (1, 1..=512) | (2, 0))
        {
            return Err(Error::Authentication);
        }
        let (key, prefix) = if self.pi {
            (&m.peer_key, &m.peer_nonce)
        } else {
            (&m.pi_key, &m.pi_nonce)
        };
        let nonce = record_nonce(prefix, seq);
        let mut c = Crypter::new(Cipher::aes_256_gcm(), Mode::Decrypt, &key[..], Some(&nonce))
            .map_err(|_| Error::Crypto)?;
        c.aad_update(&r[..29]).map_err(|_| Error::Crypto)?;
        c.set_tag(&r[29 + n..]).map_err(|_| Error::Crypto)?;
        let mut scratch = Zeroizing::new([0u8; 528]);
        let used = c
            .update(&r[29..29 + n], &mut scratch[..])
            .map_err(|_| Error::Crypto)?;
        let last = c
            .finalize(&mut scratch[used..])
            .map_err(|_| Error::Authentication)?;
        if used + last != n {
            return Err(Error::Crypto);
        }
        if r[1] == 2 {
            return Ok(None);
        }
        std::str::from_utf8(&scratch[..n]).map_err(|_| Error::Encoding)?;
        self.receive += 1;
        Ok(Some(scratch[..n].to_vec()))
    }
}

#[cfg(test)]
mod tests;
