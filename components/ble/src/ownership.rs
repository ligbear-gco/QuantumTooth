//! @file
//! @brief Kernel-released exclusive ownership of the foreground probe.

use std::{
    fs::{File, OpenOptions},
    io,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::Path,
};

/// @brief Hold an advisory file lock; closing the descriptor releases it.
pub struct ProbeLock {
    _file: File,
}

impl ProbeLock {
    /// @brief Acquire the shared runtime lock without following a symlink.
    /// @param path Root-controlled runtime file; it is deliberately never unlinked.
    /// @return An error if another probe holds it or the file cannot be opened.
    pub fn acquire(path: &Path) -> io::Result<Self> {
        Self::acquire_as(path, std::fs::metadata("/proc/self")?.uid())
    }

    /// @brief Validate ownership/type/link count before locking; never unlink a live lock inode.
    pub fn acquire_as(path: &Path, uid: u32) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let m = file.metadata()?;
        if !m.is_file()
            || m.uid() != uid
            || m.nlink() != 1
            || m.permissions().mode() & 0o777 != 0o600
        {
            return Err(io::Error::other("unsafe lock metadata"));
        }
        file.try_lock()
            .map_err(|_| io::Error::other("probe already running or lock unavailable"))?;
        Ok(Self { _file: file })
    }
}
