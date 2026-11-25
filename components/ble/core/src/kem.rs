//! @file
//! @brief The only unsafe module: missing EVP KEM/name-keygen wrappers and their ownership contracts.
use crate::{Error, Result};
use foreign_types::ForeignType;
use openssl::{
    pkey::{KeyType, PKey, Private, Public},
    pkey_ctx::PkeyCtx,
};
use openssl_sys as ffi;
use std::ptr;
use zeroize::Zeroizing;

// OpenSSL >=3.5 ABI; function-level contracts below describe borrowed synchronous calls.
unsafe extern "C" {
    /// @brief Initialize a borrowed provider context without optional parameters.
    fn EVP_PKEY_encapsulate_init(
        ctx: *mut ffi::EVP_PKEY_CTX,
        params: *const ffi::OSSL_PARAM,
    ) -> i32;
    /// @brief Query sizes or write distinct bounded ciphertext/secret buffers.
    fn EVP_PKEY_encapsulate(
        ctx: *mut ffi::EVP_PKEY_CTX,
        out: *mut u8,
        outlen: *mut usize,
        secret: *mut u8,
        secretlen: *mut usize,
    ) -> i32;
    /// @brief Initialize a borrowed private-key context without optional parameters.
    fn EVP_PKEY_decapsulate_init(
        ctx: *mut ffi::EVP_PKEY_CTX,
        params: *const ffi::OSSL_PARAM,
    ) -> i32;
    /// @brief Query the secret size or decapsulate borrowed ciphertext into owned scratch.
    fn EVP_PKEY_decapsulate(
        ctx: *mut ffi::EVP_PKEY_CTX,
        out: *mut u8,
        outlen: *mut usize,
        input: *const u8,
        inlen: usize,
    ) -> i32;
}
/// @brief Generate one ephemeral provider-owned ML-KEM key; RAII promptly frees private material.
pub fn generate() -> Result<PKey<Private>> {
    if openssl::version::number() < 0x30500000 {
        return Err(Error::Crypto);
    }
    // SAFETY: the new context pointer is checked and transferred once to its RAII owner.
    // The existing safe key-generation bindings handle private-key output ownership.
    let mut ctx = unsafe {
        let raw =
            ffi::EVP_PKEY_CTX_new_from_name(ptr::null_mut(), c"ML-KEM-768".as_ptr(), ptr::null());
        if raw.is_null() {
            return Err(Error::Crypto);
        }
        PkeyCtx::<()>::from_ptr(raw)
    };
    ctx.keygen_init().map_err(|_| Error::Crypto)?;
    ctx.keygen().map_err(|_| Error::Crypto)
}
/// @brief Export exactly the ML-KEM-768 public bytes; never export a private seed.
pub fn public(key: &PKey<Private>) -> Result<Vec<u8>> {
    let raw = key.raw_public_key().map_err(|_| Error::Crypto)?;
    if raw.len() != 1184 {
        return Err(Error::Crypto);
    }
    Ok(raw)
}
/// @brief Import checked public bytes and encapsulate into fixed maximum buffers.
pub fn encapsulate(raw: &[u8]) -> Result<(Vec<u8>, Zeroizing<[u8; 32]>)> {
    if raw.len() != 1184 {
        return Err(Error::Bounds);
    }
    let key: PKey<Public> =
        PKey::public_key_from_raw_bytes_ex(None, KeyType::ML_KEM_768, None, raw)
            .map_err(|_| Error::Crypto)?;
    let ctx = PkeyCtx::new(&key).map_err(|_| Error::Crypto)?;
    let mut ct = vec![0; 1088];
    let mut secret = Zeroizing::new([0; 32]);
    let (mut cn, mut sn) = (ct.len(), secret.len());
    // SAFETY: ctx owns a valid key reference for this scope. Both distinct output buffers
    // have their stated capacities; EVP's in/out size contract is checked before use.
    unsafe {
        if EVP_PKEY_encapsulate_init(ctx.as_ptr(), ptr::null()) <= 0
            || EVP_PKEY_encapsulate(
                ctx.as_ptr(),
                ptr::null_mut(),
                &mut cn,
                ptr::null_mut(),
                &mut sn,
            ) <= 0
            || cn != ct.len()
            || sn != secret.len()
            || EVP_PKEY_encapsulate(
                ctx.as_ptr(),
                ct.as_mut_ptr(),
                &mut cn,
                secret.as_mut_ptr(),
                &mut sn,
            ) <= 0
            || cn != 1088
            || sn != 32
        {
            return Err(Error::Crypto);
        }
    }
    Ok((ct, secret))
}
/// @brief Decapsulate exactly one ciphertext; implicit rejection is checked by key confirmation.
pub fn decapsulate(key: &PKey<Private>, ct: &[u8]) -> Result<Zeroizing<[u8; 32]>> {
    if ct.len() != 1088 {
        return Err(Error::Bounds);
    }
    let ctx = PkeyCtx::new(key).map_err(|_| Error::Crypto)?;
    let mut secret = Zeroizing::new([0; 32]);
    let mut n = secret.len();
    // SAFETY: Borrowed ct/key live through the call; secret is a distinct writable
    // 32-byte allocation. RAII frees context and zeroizes output on every return path.
    unsafe {
        if EVP_PKEY_decapsulate_init(ctx.as_ptr(), ptr::null()) <= 0
            || EVP_PKEY_decapsulate(ctx.as_ptr(), ptr::null_mut(), &mut n, ct.as_ptr(), ct.len())
                <= 0
            || n != secret.len()
            || EVP_PKEY_decapsulate(
                ctx.as_ptr(),
                secret.as_mut_ptr(),
                &mut n,
                ct.as_ptr(),
                ct.len(),
            ) <= 0
            || n != 32
        {
            return Err(Error::Crypto);
        }
    }
    Ok(secret)
}
