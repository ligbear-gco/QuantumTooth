## Why

G2.2 demonstrates authenticated PQC sessions natively, but no Android app yet
receives and visibly decrypts the Pi's BLE messages. G2.3 supplies a minimal
Pixel debug APK whose screen makes that proof understandable in a GitHub recording.

## What Changes

- Add one portrait TypeScript/React Native screen: scan/connect, protocol status,
  ciphertext preview, prominent verified plaintext and bounded diagnostics.
- Add a small Kotlin BLE/native module and Rust JNI adapter reusing the existing
  version-1 `qtooth-ble-core`; package NDK-built OpenSSL >=3.5 for ARM64.
- Integrate native compilation and JavaScript/assets bundling into Gradle's
  `:app:assembleDebug`, producing a privately provisioned APK that runs without Metro.
- Document setup, exact build/output paths and a short operator-run Pixel smoke
  experiment. The user installs with ADB and records the screen personally.
- Reuse the README UML and existing G2 evidence links; distinguish the protocol
  illustration, APK build result and later physical evidence.

G2.3 implementation acceptance is the successful paired ARM64 debug APK build
and package inspection. Validation stays functional; no exhaustive test campaign
or release pipeline. Overall G2 remains open until observed Pixel decryption and
remaining BLE-5 evidence are recorded. No application ACK, retry, automatic
reconnect, background service, message composer or FPGA work is introduced.

## Capabilities

### New Capabilities

- `ble-android-proof-app`: standalone debug APK, foreground native BLE/Rust
  integration, recording-friendly verified-result display and reproducible handoff.

### Modified Capabilities

None. The existing authenticated protocol and Pi command contracts are preserved.

## Impact

Conforms to [DESIGN](../../../DESIGN.md) SCOPE, BASE-1..3, BLE-1..5 and G2, including
the user-authorized React Native UI revision, and [HARDWARE](../../../HARDWARE.md)
HW-1/HW-3. Neither authority nor its protected figures needs another revision.

The app lives in top-level `app/`: React Native sources, `app/android/` for
Gradle, and `app/native/` for the Rust JNI adapter. Reuse `components/ble/core/`
through a path dependency and keep an independent Android Cargo workspace/lockfile
to preserve Yocto source collection, Pi system-libcrypto linkage and defaults.
Integration adds Android build support and ignore rules.
Dependencies include React Native/React/TypeScript, Node/npm, Android SDK/NDK,
JDK, Rust Android target, JNI bindings and pinned OpenSSL sources. Pin compatible
versions during the toolchain preflight rather than assuming installed SDKs.

Prerequisites: Android Studio (already installed), native toolchains, the G2.2 Pi
image and the same ignored `local/ble/pair.bin` for both peers. Provisioned APKs
and raw evidence remain private.

Learning outcomes: trace GATT callbacks through JNI into Rust and back to React
Native; distinguish ATT confirmation from authenticated decryption; explain why
the debug APK needs an embedded JavaScript bundle. The visible mechanism is
fragment reassembly → authenticated session → tag-verified plaintext → UI result.
