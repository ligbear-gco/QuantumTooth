//! @file
//! @brief Optional private pair input; generated artifacts stay under Cargo's output directory.
use std::{env, fs, path::PathBuf};
/// @brief Track pair contents through Cargo rebuilds; absent input never installs a default key.
fn main() {
    println!("cargo:rerun-if-env-changed=QTOOTH_PAIR_FILE");
    let bytes = if let Some(path) = env::var_os("QTOOTH_PAIR_FILE") {
        println!("cargo:rerun-if-changed={}", PathBuf::from(&path).display());
        let bytes = fs::read(path).expect("cannot read private pair input");
        assert_eq!(
            bytes.len(),
            64,
            "private pair input must contain exactly 64 bytes"
        );
        bytes
    } else {
        Vec::new()
    };
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("pair.bin"),
        bytes,
    )
    .expect("cannot stage private pair input");
}
