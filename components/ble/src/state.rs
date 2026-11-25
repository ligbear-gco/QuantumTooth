//! @file
//! @brief Bounded probe validation and monotonic phase deadlines.

use bluer::{
    Address,
    gatt::{WriteOp, local::ReqError},
};
use tokio::{
    sync::mpsc,
    time::{Duration, Instant},
};

/// @brief Public diagnostic bytes; this is neither plaintext delivery nor a key.
pub const TOKEN: [u8; 4] = *b"QTB1";
/// @brief Maximum advertising-to-readiness interval.
pub const READY_LIMIT: Duration = Duration::from_secs(30);
/// @brief Absolute readiness-to-confirmation interval.
pub const EXCHANGE_LIMIT: Duration = Duration::from_secs(120);

/// @brief Address-free metadata handed off once to the indication future.
#[derive(Debug)]
pub struct Transfer {
    pub mtu: u16,
}

/// @brief Borrow only fields needed for validation; never log the BlueR request.
pub struct Write<'a> {
    pub peer: Address,
    pub adapter_matches: bool,
    pub offset: u16,
    pub operation: WriteOp,
    pub prepare: bool,
    pub mtu: u16,
    pub bytes: &'a [u8],
}

/// @brief One peer, one subscription and one outstanding transfer per process.
pub struct Gate {
    pub peer: Option<Address>,
    pub subscribed: bool,
    pub busy: bool,
    pub closed: bool,
    pub ready_at: Option<Instant>,
    pub deadline: Instant,
}

impl Gate {
    /// @brief Start the non-refreshing readiness deadline after advertising.
    pub fn new(now: Instant) -> Self {
        Self {
            peer: None,
            subscribed: false,
            busy: false,
            closed: false,
            ready_at: None,
            deadline: now + READY_LIMIT,
        }
    }

    /// @brief Start exchange time once both connection and subscription exist.
    /// @return True exactly on the transition to readiness.
    pub fn update_readiness(&mut self, now: Instant) -> bool {
        if !self.closed
            && self.peer.is_some()
            && self.subscribed
            && self.ready_at.is_none()
            && now < self.deadline
        {
            self.ready_at = Some(now);
            self.deadline = now + EXCHANGE_LIMIT;
            true
        } else {
            false
        }
    }

    /// @brief Validate and reserve the single handoff without awaiting delivery.
    /// @return An ATT error for invalid traffic, busy state or closed handoff.
    pub fn submit(
        &mut self,
        write: Write<'_>,
        tx: &mpsc::Sender<Transfer>,
        now: Instant,
    ) -> Result<(), ReqError> {
        if self.closed || now >= self.deadline {
            return Err(ReqError::NotPermitted);
        }
        if write.operation != WriteOp::Request || write.prepare {
            return Err(ReqError::NotSupported);
        }
        if write.offset != 0 {
            return Err(ReqError::InvalidOffset);
        }
        if write.bytes.len() != TOKEN.len() {
            return Err(ReqError::InvalidValueLength);
        }
        if write.bytes != TOKEN {
            return Err(ReqError::NotPermitted);
        }
        if !write.adapter_matches
            || self.peer != Some(write.peer)
            || !self.subscribed
            || self.ready_at.is_none()
            || write.mtu < 23
        {
            return Err(ReqError::NotPermitted);
        }
        if self.busy {
            return Err(ReqError::InProgress);
        }
        tx.try_send(Transfer { mtu: write.mtu })
            .map_err(|error| match error {
                mpsc::error::TrySendError::Full(_) => ReqError::InProgress,
                mpsc::error::TrySendError::Closed(_) => ReqError::NotPermitted,
            })?;
        self.busy = true;
        Ok(())
    }

    /// @brief Invalidate callbacks before releasing external resources.
    pub fn close(&mut self) {
        self.closed = true;
        self.subscribed = false;
    }

    /// @brief Invalidate only a lost owned link; leave unrelated connections alone.
    /// @return True when the lost connection terminates this probe.
    pub fn lost_link(&mut self, peer: Address) -> bool {
        if self.peer == Some(peer) {
            self.close();
            true
        } else {
            false
        }
    }
}
