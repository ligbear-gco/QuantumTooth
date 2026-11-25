//! @file
//! @brief Platform-independent bounded protocol; adapters supply monotonic milliseconds.
mod crypto;
pub mod framing;
mod kem;
pub mod protocol;

/// @brief Sanitized failure categories; no backend errors or key-bearing Debug values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    Encoding,
    State,
    Bounds,
    Expired,
    Authentication,
    Crypto,
    Exhausted,
}
/// @brief Fallible core operation.
pub type Result<T> = std::result::Result<T, Error>;

/// @brief Transport boundary; adapters await normal transport completion for every value.
/// @note No application acknowledgment, retry or platform types belong in the core.
pub trait Transport {
    /// @brief Send one fragment under the supplied absolute monotonic deadline.
    fn send(
        &mut self,
        value: &[u8],
        deadline_ms: u64,
    ) -> impl std::future::Future<Output = Result<()>>;
    /// @brief Receive one bounded fragment under the supplied deadline.
    fn receive(&mut self, deadline_ms: u64) -> impl std::future::Future<Output = Result<Vec<u8>>>;
}
