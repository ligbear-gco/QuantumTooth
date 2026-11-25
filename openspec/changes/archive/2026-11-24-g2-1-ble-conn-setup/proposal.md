## Why

G1 established the Pi Bluetooth controller's readiness, but not a connection to
the Pixel. Validate the real GATT transport with a generic foreground phone app
before adding PQC or a custom Android client, so transport and crypto failures
can be investigated separately.

## What Changes

- Establish the G2 sequence below; only G2.1 is implemented by this change.
- Add an explicitly unencrypted `qtooth-ble probe` diagnostic: advertise, accept
  one phone, receive a known test write with response, and return an indication.
- Introduce the Rust application and `meta-qtooth-ble` Cargo packaging, with an
  opt-in BLE diagnostic build selection and separate build/evidence directory.
- Adapt the pinned BlueZ server in the BLE layer to preserve actual ATT write
  types for strict BlueR validation; default G1 builds exclude this patch.
- Document a learner-run Pixel experiment covering subscription, observed MTU,
  bounded waiting, disconnect/interrupt cleanup and a fresh subsequent run.

| Milestone | Deliverable and acceptance boundary |
|---|---|
| G2.1 — this change | Physical Pi/Pixel connection and bidirectional GATT test bytes using a generic BLE inspector. No application authentication or decryption claim. |
| G2.2 — later change | Pi/shared Rust framing, PSK-authenticated ML-KEM exchange, AEAD records/receipts and `send`/`sendkeep`/`closekeep`; automated peer tests before the custom phone app. |
| G2.3 — later change | Minimal Kotlin BLE/UI adapter with shared Rust core through JNI; physical Pixel decryption and all BLE-5 evidence. Only then accept G2. |

Future command decisions: `send` always establishes a fresh session and rejects
while a kept session exists; only `sendkeep` reuses it; `closekeep` closes it.
Handshake availability is bounded to 30 s. The 120 s open-session timeout is
currently interpreted as an absolute lifetime, not refreshed by sends. The demo
pair's unique 256-bit PSK will be embedded from ignored local build inputs;
no actual key is generated or included here. Final plaintext cap, wire encoding
and post-handshake operation deadlines belong to G2.2.

## Capabilities

### New Capabilities

- `ble-connection-setup`: Opt-in Pi GATT transport probe, reproducible Yocto
  packaging, bounded lifecycle and attributable generic-phone verification.

### Modified Capabilities

None. Existing platform images retain their default package/access policy.

## Impact

Affects `components/ble/`, `layers/meta-qtooth-ble/`, an optional Kas feature
fragment, build selectors and a focused transport lab. Uses Rust/clap/Tokio/BlueR
and existing BlueZ/D-Bus; target Cargo/compiler, crypto protocol, persistent
worker, Android project and FPGA work are outside this milestone.

Conforms to DESIGN SCOPE, BASE-1..3, BLE-1/BLE-3 and staged G2, and HARDWARE HW-1 /
HW-3's Bluetooth UART reservation. BLE-2/4/5 remain later gates. No source-of-truth
documents, architecture diagrams, kernel configuration or hardware wiring change.

Prerequisites: completed G1, Linux Docker/kas build environment, diagnostic Pi
shell access and the HW-1 Pixel with a generic BLE inspector in the foreground.
Learning outcomes: trace Rust → D-Bus → BlueZ → controller → phone; distinguish
GATT values, ATT MTU and indications; trace a Rust binary from Cargo recipe to
image manifest and inspect resource cleanup. The assistant implements in small increments, explains consequential choices,
and adds standard Doxygen documentation, as requested by the operator. No implementation or physical result is claimed
by these planning artifacts.
