//! @file
//! @brief Fixed-storage, contiguous version-1 framing with absolute deadlines.
use crate::{Error, Result};
/// @brief Largest allowed logical object storage per direction.
pub const CAP: usize = 2048;
/// @brief One application header per ordinary ATT value.
pub const HEADER: usize = 8;
/// @brief Frozen object kinds; response core uses PeerFinish's transcript kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Kind {
    ClientHello = 1,
    PiOffer = 2,
    PeerFinish = 3,
    PiFinish = 4,
    Record = 5,
}
impl Kind {
    /// @brief Enforce type-specific wire lengths before copying a fragment.
    pub fn permits(self, n: usize) -> bool {
        match self {
            Self::ClientHello => n == 68,
            Self::PiOffer => n == 1249,
            Self::PeerFinish => n == 1152,
            Self::PiFinish => n == 32,
            Self::Record => (45..=557).contains(&n),
        }
    }
}
/// @brief Compute ordinary ATT value budget; no long-write fallback.
pub fn budget(mtu: u16) -> Result<usize> {
    if mtu < 23 {
        Err(Error::Bounds)
    } else {
        Ok(usize::from(mtu - 3).min(512))
    }
}
/// @brief Encode one complete object into bounded ATT values; caller encrypts records first.
pub fn fragments(kind: Kind, id: u16, body: &[u8], mtu: u16) -> Result<Vec<Vec<u8>>> {
    if !kind.permits(body.len()) {
        return Err(Error::Bounds);
    }
    let size = budget(mtu)? - HEADER;
    Ok(body
        .chunks(size)
        .enumerate()
        .map(|(i, b)| {
            let mut v = vec![1, kind as u8];
            v.extend_from_slice(&id.to_be_bytes());
            v.extend_from_slice(&(body.len() as u16).to_be_bytes());
            v.extend_from_slice(&((i * size) as u16).to_be_bytes());
            v.extend_from_slice(b);
            v
        })
        .collect())
}
/// @brief One permanently fail-closed receiver; no fragment can allocate storage or refresh time.
pub struct Reassembly {
    data: Box<[u8; CAP]>,
    next: u32,
    used: usize,
    total: usize,
    until: u64,
    kind: Option<Kind>,
    failed: bool,
}
impl Default for Reassembly {
    /// @brief Allocate the single fixed-capacity buffer once.
    fn default() -> Self {
        Self {
            data: Box::new([0; CAP]),
            next: 0,
            used: 0,
            total: 0,
            until: 0,
            kind: None,
            failed: false,
        }
    }
}
impl Reassembly {
    /// @brief Discard incomplete bytes and prevent revival after any protocol failure.
    pub fn abort(&mut self) {
        self.data.fill(0);
        self.failed = true;
        self.used = 0;
    }
    /// @brief Earliest incomplete-object expiry, or enclosing phase/session deadline.
    pub fn deadline(&self, enclosing: u64) -> u64 {
        if self.kind.is_some() {
            enclosing.min(self.until)
        } else {
            enclosing
        }
    }
    /// @brief Validate state, budget, headers and deadline before copying; return only complete objects.
    /// @param now Monotonic milliseconds supplied by the platform adapter.
    pub fn push(
        &mut self,
        v: &[u8],
        mtu: u16,
        expected: Kind,
        now: u64,
        enclosing: u64,
    ) -> Result<Option<Vec<u8>>> {
        let result = self.accept(v, mtu, expected, now, enclosing);
        if result.is_err() {
            self.abort()
        };
        result
    }
    /// @brief Internal validation; caller invalidates the receiver on every error.
    fn accept(
        &mut self,
        v: &[u8],
        mtu: u16,
        expected: Kind,
        now: u64,
        enclosing: u64,
    ) -> Result<Option<Vec<u8>>> {
        if self.failed {
            return Err(Error::State);
        }
        if now >= self.deadline(enclosing) {
            return Err(Error::Expired);
        }
        if v.len() <= HEADER || v.len() > budget(mtu)? {
            return Err(Error::Bounds);
        }
        if v[0] != 1 || v[1] != expected as u8 {
            return Err(Error::State);
        }
        let id = u16::from_be_bytes([v[2], v[3]]) as u32;
        let total = u16::from_be_bytes([v[4], v[5]]) as usize;
        let offset = u16::from_be_bytes([v[6], v[7]]) as usize;
        if id != self.next
            || !expected.permits(total)
            || total > CAP
            || offset != self.used
            || offset + v.len() - HEADER > total
        {
            return Err(Error::Bounds);
        }
        if let Some(kind) = self.kind {
            if kind != expected || total != self.total {
                return Err(Error::State);
            }
        } else {
            self.kind = Some(expected);
            self.total = total;
            self.until = now.saturating_add(10_000).min(enclosing)
        }
        let end = offset + v.len() - HEADER;
        self.data[offset..end].copy_from_slice(&v[HEADER..]);
        self.used = end;
        if end == total {
            let body = self.data[..end].to_vec();
            self.data[..end].fill(0);
            self.used = 0;
            self.kind = None;
            self.next += 1;
            Ok(Some(body))
        } else {
            Ok(None)
        }
    }
}
