//! @file
//! @brief Embed exactly the user-provisioned pair; Cargo tracks path and contents.
use std::{env, fs, path::PathBuf};
/// @brief Stage private bytes only inside ignored Cargo output, failing before linking.
fn main() {
    println!("cargo:rerun-if-env-changed=QTOOTH_PAIR_FILE");
    let input = env::var_os("QTOOTH_PAIR_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../local/ble/pair.bin")
        });
    println!("cargo:rerun-if-changed={}", input.display());
    let bytes = fs::read(input).expect("private pair input missing; restore local/ble/pair.bin");
    assert_eq!(
        bytes.len(),
        64,
        "private pair input must be exactly 64 bytes"
    );
    fs::write(
        PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("pair.bin"),
        bytes,
    )
    .expect("cannot stage private pair input");
}
