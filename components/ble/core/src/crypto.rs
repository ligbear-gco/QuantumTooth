//! @file
//! @brief Safe EVP HMAC/HKDF and bounded secret ownership; no secret Debug or Clone.
use crate::{Error, Result};
use openssl::{
    hash::MessageDigest,
    md::Md,
    pkey::{Id, PKey},
    pkey_ctx::{HkdfMode, PkeyCtx},
    sign::Signer,
};
use zeroize::Zeroizing;
/// @brief Generate fresh nonce bytes with system OpenSSL randomness.
pub fn nonce() -> Result<[u8; 32]> {
    let mut n = [0; 32];
    openssl::rand::rand_bytes(&mut n).map_err(|_| Error::Crypto)?;
    Ok(n)
}
/// @brief HMAC-SHA-256 of ordered byte slices, with checked digest length.
pub fn mac(key: &[u8], parts: &[&[u8]]) -> Result<[u8; 32]> {
    let key = PKey::hmac(key).map_err(|_| Error::Crypto)?;
    let mut signer = Signer::new(MessageDigest::sha256(), &key).map_err(|_| Error::Crypto)?;
    for p in parts {
        signer.update(p).map_err(|_| Error::Crypto)?;
    }
    let mut out = [0; 32];
    if signer.sign(&mut out).map_err(|_| Error::Crypto)? != 32 {
        return Err(Error::Crypto);
    }
    Ok(out)
}
/// @brief Verify public-length authentication values with constant-time comparison.
pub fn verify(a: &[u8], b: &[u8]) -> Result<()> {
    if a.len() == b.len() && openssl::memcmp::eq(a, b) {
        Ok(())
    } else {
        Err(Error::Authentication)
    }
}
/// @brief Derive one independent role-separated output under the frozen info label.
pub fn derive<const N: usize>(
    secret: &[u8],
    hash: &[u8; 32],
    label: &[u8],
) -> Result<Zeroizing<[u8; N]>> {
    let mut ctx = PkeyCtx::new_id(Id::HKDF).map_err(|_| Error::Crypto)?;
    ctx.derive_init().map_err(|_| Error::Crypto)?;
    ctx.set_hkdf_mode(HkdfMode::EXTRACT_THEN_EXPAND)
        .map_err(|_| Error::Crypto)?;
    ctx.set_hkdf_md(Md::sha256()).map_err(|_| Error::Crypto)?;
    ctx.set_hkdf_salt(hash).map_err(|_| Error::Crypto)?;
    ctx.set_hkdf_key(secret).map_err(|_| Error::Crypto)?;
    ctx.add_hkdf_info(label).map_err(|_| Error::Crypto)?;
    let mut out = Zeroizing::new([0; N]);
    if ctx.derive(Some(&mut out[..])).map_err(|_| Error::Crypto)? != N {
        return Err(Error::Crypto);
    }
    Ok(out)
}
