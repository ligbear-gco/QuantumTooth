## Context

See [proposal.md](proposal.md) for the G2 split and
[the transport contract](specs/ble-connection-setup/spec.md). G1 provides BlueZ
5.86, system D-Bus, firmware and diagnostic SSH. At planning time the BLE layer
was a placeholder, with no application/recipe; selectors chose only
machine/profile/image. BLE therefore needs an explicit additional choice.

Design included because the new Rust dependency chain, Yocto integration and
asynchronous resource lifetime cross several boundaries. This remains within
DESIGN BASE-1..3/BLE-1/BLE-3 and HARDWARE HW-1/HW-3; no kernel/UART changes.

## Goals / Non-Goals

**Goals:** make one physical transport round trip observable and repeatable;
establish locked Rust packaging without changing default platform images.

**Non-Goals:** no production plaintext transport, KEM/framing/AEAD, daemon/local
socket, Android project, PSK material or final G2 acceptance. The probe's small
test token does not define the future application's message limit.

## Decisions

### 1. Separate foreground probe, not an insecure implementation of send

Place a standalone Cargo crate in `components/ble/`, producing `qtooth-ble` with
the `probe` subcommand. Use clap, Tokio and BlueR's callback-based local GATT
API. Lock all versions in Cargo.lock; start compatibility checking with BlueR
0.17.4, whose documented APIs cover writes, request MTU and confirmed indications.
Do not silently substitute a different stack if the pinned build fails.

One foreground process owns a D-Bus session, advertisement, GATT registration
and tasks. A local exclusive runtime lock prevents two probes. Owned handles
and explicit asynchronous cleanup release resources on success, Ctrl-C/SIGTERM,
disconnect or timeout; no detached task may outlive the process. Run manually
from the existing diagnostic shell; no boot-enabled service is added.

Alternative: build the retained-session worker first. Rejected for this milestone:
it mixes transport debugging with IPC, session ownership and crypto state.

### 2. Fixed, distinct test service and public token

Use name `QTooth Probe`, with these project-owned 128-bit UUIDs, distinct from
any later authenticated service:

| Role | UUID | Operation |
|---|---|---|
| Service | `b6631000-7e7b-4fb5-9dc4-93f4152a0e7a` | Primary GATT service |
| RX (phone → Pi) | `b6631001-7e7b-4fb5-9dc4-93f4152a0e7a` | Write with response only |
| TX (Pi → phone) | `b6631002-7e7b-4fb5-9dc4-93f4152a0e7a` | Indications only |

Register GATT before advertising. BlueZ manages the CCCD (Client Characteristic
Configuration Descriptor) subscription; do not hand-build another CCCD. Accept
only the public token `51 54 42 31` (`QTB1`), offset zero, after subscription.
Reject wrong input/operation through an ATT error. A capacity-one handoff joins
the write callback to the indication task; a busy transfer rejects extra writes.
Do not wait for indication confirmation inside the write callback.

BlueR's `CharacteristicNotify` supports `indicate=true, notify=false`; check
`CharacteristicNotifier::confirming()` and await `notify()` to distinguish a
confirmed indication from merely queuing bytes. Log transport events, elapsed
times, request MTU and counts; never debug-print entire request structs containing
peer addresses. Report success only after valid write and confirmed indication.

```text
Phone -- ATT write request --> BlueZ -- D-Bus callback --> Rust
Phone <-- ATT write response - BlueZ <-- accept/reject --- Rust
Phone <-- ATT indication ----- BlueZ <-- fixed token ----- Rust
Phone -- ATT confirmation ---> BlueZ --> indication future completes
```

Alternative: use notifications or generic Bluetooth utilities as the server.
Rejected: they would not validate the selected Rust API and indication path.

### 3. Small bounded lifecycle before protocol state machines

Use `register → advertise/wait (30 s) → connected/subscribed (120 s) → exchange
→ cleanup`. Deadlines use a monotonic clock; these are userspace timeouts, not
hard real-time guarantees. Readiness begins the absolute 120 s window, which
invalid traffic cannot refresh. The first successful round trip ends the probe;
disconnect, loss of subscription or indication failure ends it unsuccessfully.

Stop advertising after selecting the test peer. Associate connection events and
writes with that peer, reject another peer's writes, and disconnect only the
owned test link. Single-phone bench operation is the acceptance setup; this is
not a multi-client server or proof of peer identity. Bound outstanding D-Bus work
and cleanup; preserve shared services and the adapter's prior power policy.

The four-byte token fits at MTU 23 without application fragmentation. Report the
MTU actually supplied by the write request; never infer MTU 23 from a short write.
Physical MTU-23/large-object fragmentation proof remains in BLE-5, after framing.

### 4. Feature-owned packaging and isolated build selection

Add `layers/meta-qtooth-ble/conf/layer.conf`, a `qtooth-ble` Cargo recipe and its
locked crate-source include. Use OE's cargo classes and checksum-identified
`crate://` inputs; compilation must remain frozen/offline after fetching. Source
the repository-owned crate without absolute workstation paths. Include the
system D-Bus development dependency needed by BlueR's bindings; do not vendor
libdbus or OpenSSL. Inspect actual dynamic linkage and runtime package closure.
This phase performs no crypto linkage work and does not update upstream pins.

Extend `tools/build-selection` with `--feature none|ble` (default `none`), propagated
through existing wrapper selectors. Support `ble` only for the Pi diagnostic
selection, keep the selected RT/non-RT profile explicit, append
`kas/features/ble.yml`, and use `build/raspberrypi4-64-<profile>-diagnostic-ble`.
The fragment adds the BLE layer and package to the diagnostic image. Defaults
retain existing paths and packages; base/QEMU BLE selections fail visibly.

Alternative: globally install the feature in the base packagegroup. Rejected:
it changes G1 defaults and breaks independent feature selection.

### 5. Preserve ATT operation metadata at the BlueZ boundary

Physical validation found that BlueZ 5.86 omits `type` in server-side
`WriteValue`, which BlueR 0.17.4 defaults to `Command`. Preserve the actual
characteristic ATT opcode in a feature-local BlueZ patch: Request → `request`,
Command/signed Command → `command`, Prepare/Execute → `reliable`. Missing or
unknown metadata remains rejected; never infer Request from advertised flags.
Descriptor/read behavior and pinned versions stay unchanged. Only selecting the
BLE layer applies this append. Exercise BlueR's real D-Bus parsing on a private
bus and record both the probe and packaged `bluetoothd` hashes in build evidence.
A boundary test does not establish phone/radio behavior.

### 6. Implementation and evidence

Implement crate/CLI, GATT callbacks, lifecycle and packaging in that order,
with concise explanations, useful comments and standard Doxygen tags. Use focused
tests for invalid writes, deadline transitions, pending confirmation/cancellation
and exclusive ownership; mocks cannot establish physical radio behavior.

Create `docs/labs/g2-1-ble-connection.md` with exact commands and phone actions,
the UUID/token table, expected events and failure/restart checks. Keep raw logs
under ignored `build/` or `local/`; a reviewed `verification.md` in this change
will identify image/source hashes, actual peer/client versions, observed MTU,
results and limitations without addresses, secrets or workstation paths. It
must be self-contained and must not depend on ignored `LOCAL_DESIGN.md`.

## Risks / Trade-offs

- BlueR/BlueZ indication or peer-event behavior differs on target → validate it
  on the physical Pi early; log the exact failed phase and stop dependent work.
- Inspector auto-negotiates MTU or caches GATT → record actual MTU and inspector
  version, refresh discovery, and disclose inability to force MTU 23.
- Unauthenticated diagnostic can be reached by nearby peers → use only public
  fixed test bytes, short lifetime and distinct service; no application secrets.
- A dependency/build failure is mistaken for a BLE failure → inspect the locked
  Cargo/Yocto build and ELF dependencies before deploying; retain failed evidence.

## Migration Plan

Build/check the opt-in diagnostic variant, retain its hashes and matching evidence,
then deploy through the established G1 verified-SD procedure. Run physical checks;
do not close G2 on a build or simulated result. Roll back by booting the retained
G1 image or rebuilding the default selection without the BLE feature.

References: [BlueR request MTU](https://docs.rs/bluer/0.17.4/bluer/gatt/local/struct.CharacteristicWriteRequest.html),
[indication flags](https://docs.rs/bluer/0.17.4/bluer/gatt/local/struct.CharacteristicNotify.html),
[confirmation semantics](https://docs.rs/bluer/0.17.4/bluer/gatt/local/struct.CharacteristicNotifier.html).
Build authority is the pinned OE `cargo.bbclass` and
`cargo-update-recipe-crates.bbclass`, inspected during planning.
