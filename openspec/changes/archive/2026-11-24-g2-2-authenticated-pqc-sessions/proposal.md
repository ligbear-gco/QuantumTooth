## Why

G2.1 established Pi/Pixel BLE transport, but the application still has no
authentication, key establishment or encrypted messaging. G2.2 adds those
mechanisms and all three session commands, with focused automated verification
before the Android adapter and physical decryption work in G2.3.

## What Changes

- Add a transport-independent shared Rust core for bounded framing,
  PSK-authenticated ML-KEM-768 establishment, mutual key confirmation, HKDF and
  directional AES-256-GCM records (DESIGN BLE-2..4).
- Implement `qtooth-ble send "text"`, `sendkeep "text"` and `closekeep`, backed
  by one on-demand worker retaining the live connection, keys and counters.
- Send application data once, best effort: **no application ACK, receipt,
  automatic retry or delivery guarantee**. Keep BLE responses/indication
  confirmations and handshake key confirmation. Close is best effort too.
- Use `Hello, world!` as the demo plaintext, without restricting the CLI to that
  literal. Adopt a 512-byte UTF-8 input cap as a planning default.
- Carry forward the 30 s establishment deadline and absolute 120 s session
  lifetime; bound fragment, transfer and cleanup waits within them.
- Add system OpenSSL integration, ignored pair-PSK build inputs, focused Rust
  peer/worker tests and an executable learner experiment. Update the live G2
  course/design draft to remove superseded receipt assumptions during apply.
- Keep `probe` available and its transport contract intact. This change adds an
  authenticated service with separate UUIDs and exclusive runtime ownership.

Earlier course/draft references to authenticated data receipts and close
exchanges are superseded for this change. Neither the frozen architecture nor
the accepted G2.1 evidence is rewritten.

## Capabilities

### New Capabilities

- `ble-authenticated-protocol`: Bounded framing, pair authentication, ephemeral
  key establishment, mutual confirmation and encrypted best-effort records.
- `ble-session-commands`: Three CLI commands, retained-session worker,
  restricted local IPC, finite lifecycle and honest completion reporting.

### Modified Capabilities

None. The existing `ble-connection-setup` probe requirements remain valid.

## Impact

Touches `components/ble/`, `layers/meta-qtooth-ble/`, BLE build/evidence tooling
and G2 teaching documents. Introduces locked Rust crypto bindings and links the
Pi executable to system OpenSSL >=3.5 `libcrypto`, without a vendored duplicate.
The exact shared-crate packaging and KEM binding route are feasibility checks.
Operator-specific PSKs, provisioned binaries/images and raw evidence remain in
ignored `local/` or build directories.

Conforms to DESIGN SCOPE, BASE-1..3, BLE-1..5/staged G2 and HARDWARE HW-1/HW-3.
No hardware, kernel, FPGA, root SoT or protected architecture-figure changes.
G2.3 owns Kotlin/JNI/NDK integration, physical Pixel decryption and remaining
BLE-5 measurements; overall G2 stays open. G2.2 acceptance is focused native
peer/worker checks plus attributable Pi cross-build/linkage inspection, with
any unavailable target checks explicitly reported rather than claimed passed.

Prerequisites: accepted G1/G2.1, existing Cargo/kas build environment, OpenSSL
>=3.5 for native crypto checks and ignored provisioning inputs for a real pair.
No custom phone app is required for this increment's automated experiment.

Learning outcomes:
1. Trace fragments through authenticated establishment to decrypted plaintext.
2. Explain why a worker owns keys/counters across separate shell commands.
3. Distinguish transport completion, key confirmation and observed decryption.

The learner experiment will show `Hello, world!` decrypted by a native peer,
session reuse across two `sendkeep` invocations, explicit close, and rejection
of wrong-PSK/tampered traffic, without claiming a physical phone result.
