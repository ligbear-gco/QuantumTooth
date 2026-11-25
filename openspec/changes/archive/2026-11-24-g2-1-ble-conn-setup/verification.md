# G2.1 verification — 2026-11-24

**G2.1 accepted — 13/13 tasks complete.** The physical Pi/Pixel public-token
round trip and failure suite pass on the evidence basis below. Cases 10/11
have operator-reported passes without saved output. Physical MTU 23 is
unperformed. G2.2/G2.3, BLE-2/4/5 and overall G2 acceptance remain open.

## Verified software behavior

- The standalone `components/ble/` crate exposes only `probe`, with BlueR 0.17.4
  and a locked Cargo.lock. Host Rust/Cargo 1.98.0 and system D-Bus 1.16.2
  support locked native checks; no service is installed at boot.
- `cargo check --locked`, CLI help, 19 tests (including real D-Bus parsing), formatting and Clippy with
  warnings denied pass. Unsupported `send` is rejected by argument parsing.
  Tests cover token/length/operation/offset/subscription/peer validation,
  capacity-one handoff, confirmation failure/expiry, cancellation, link loss,
  deadline non-refresh, exclusive ownership and fresh state after release.
- RX uses writes with response; TX uses indications only. The write callback
  returns without awaiting acknowledgment; success requires confirmed delivery.
  Events expose counts, observed request MTU and elapsed time without addresses.
- The process lock spans runtime shutdown. Setup, readiness, exchange and
  owned-link cleanup are bounded; prior adapter power and unrelated links are
  preserved. BlueR unregisters handles asynchronously; process runtime shutdown
  closes its own bus connection as the final release mechanism.
- Selector checks preserve QEMU and all four Pi default paths/configurations,
  support both Pi diagnostic profiles with BLE, preserve forwarded arguments,
  and reject base/QEMU/unknown BLE selections. All retained G1 manifests and
  layer configurations exclude the BLE feature.
- The exact BLE RT diagnostic `--check` command passes: 997 recipes parsed,
  zero errors. Resolved Kas includes the feature layer/package. The corrected
  full build passes all 3813 BitBake tasks, rerunning 42 with cached tooling. OE Rust/Cargo 1.94.1 compiles the application with
  `--frozen` from 90 checksum-identified crate inputs.
- Package/image executable hashes match. The collector verifies ELF64 AArch64,
  dependencies `libdbus-1.so.3`, `libgcc_s.so.1`, `libc.so.6`, and exact
  recipe/repository source equality. The image includes `qtooth-ble 0.1.0`,
  BlueZ 5.86, `dbus-1` and `libdbus-1-3` 1.16.2, with no target compiler or
  application crypto/vendored D-Bus linkage. The retained kernel/config still
  match G1 RT 6.18.33-v8. Patched BlueZ is packaged as
  `5.86-r0.qtoothble1`; its installed daemon matches the exact emitted IPK.
- The self-contained [G2 course](../../../../docs/courses/G2_Architecture_and_Practical_Guide.md) covers
  verified-SD deployment, generic inspector actions and failure/restart trials.
  Its shell examples pass syntax checks. Protected SoT/diagram hashes,
  whitespace/privacy review and strict OpenSpec validation pass.

## Evidence and limitations

The verified raw SD image is frozen in ignored
`build/releases/rt-diagnostic-ble-20261124T161909Z/`, with source archive/patch,
boot-file hashes, binary/source attribution and a successfully checked
`SHA256SUMS`. Both binaries were also extracted read-only from the actual raw
SD image with `debugfs`; their hashes match package/rootfs attribution. Compressed WIC is 60.84 MiB; raw image is 364.80 MiB. These are
file sizes, not installed-package size, RAM or boot measurements.

| Artifact | SHA-256 |
|---|---|
| Frozen `image.img` | `22e183d329d30941035e3a47ab2e8a1c61f0e22ca7034e828b844dd0fc0486e5` |
| Built WIC.bz2 | `8beb8049348c2e950339338c969ce36148433c4679d2554b7bc2d3edfe6b3b14` |
| Installed `usr/bin/qtooth-ble` | `5342941294b5b4a93b26630b3cecb77cb8a4db232b53c577a42a83f901847cbd` |
| Installed `usr/libexec/bluetooth/bluetoothd` | `38f91c7e66cee3ad9541f17a38684ca12e591c499a32d10df9f41d948977366b` |
| Feature-local BlueZ patch | `f947d173e1f1bc1d98f1caf8ebe362be534044800c460815dc47bb20d424c7de` |
| `components/ble/Cargo.lock` | `886e09db705a588230933a4e612c65dea95b7d6bbb856f3cc12bbd3e57c73508` |

Private evidence stays in ignored `local/g2-1-ble-conn-setup/` and
`build/raspberrypi4-64-rt-diagnostic-ble/evidence/`. It includes source/protected
hashes, native checks, dependency features, metadata and full build logs.
Initial sandbox registry DNS failure was resolved using approved network access;
two initial Clippy match warnings were corrected. Failures are retained privately.
The first post-build collector incorrectly expected recipe name `dbus` in the
manifest. The pinned recipe explicitly renames it to `dbus-1`; the corrected
collector verifies the daemon and shared-library package names. Its failed log
is retained. The first build also reports the existing access recipe's unpack
directory warning; no G1 recipe was changed. Final source rebuild/collection pass.

The assistant did not operate the physical Pi/Pixel or write a card. The
operator supplied the phone screenshot and case logs, confirmed matching step 3
hashes, and states that Imager verified every write without issues. The client
is **nRF Connect 4.29.1**, on **Google Pixel 7a / Android 17**. Deployment hash
matching/read-back and cases 10/11 rely on operator testimony; no raw baseline
log or terminal output for 10/11 was supplied. Separate confirmed round-trip
logs after every failure are unavailable; repeated fresh invocations appear in
the logs and the owner trial's successful first round trip is operator-reported.
Physical MTU-23 behavior is **unperformed**; native MTU-23 checks do not establish
a negotiated target MTU or BLE-5 fragmentation/decryption evidence.

The probe is an unauthenticated single-phone bench diagnostic. BlueR's
subscription callback supplies no peer identity; this is not multi-client
isolation or authenticated receipt. Cached-device observers are capped at 64.
Native tests cannot establish Pi BlueZ callback timing, radio connectivity,
phone display/confirmation or post-disconnect restart behavior.

## Operator-reported physical attempts

Two attempts supplied by the operator reached `connected`, then reported
`phase=connection error=link_lost`; neither reached `subscribed`, `ready`,
write or indication confirmation. Both reported cleanup, probe exit 1 and
active D-Bus/Bluetooth services (`services_exit=0`).

| Attempt | Total probe elapsed ms | Result |
|---|---:|---|
| `01-round-trip` | 13695 | Link lost before observed subscription |
| `01-round-trip_2` | 3561 | Link lost before observed subscription |

The operator confirms using the Pixel 7a's built-in Bluetooth tile, which shows
an error and disconnects. These attempts did not exercise the required GATT
inspector actions. Logs are retained privately as operator-supplied evidence;
runtime/source attribution and phone OS/client versions were not supplied for
these attempts; the later successful trial is recorded below. The logs alone do not isolate the low-level disconnect
reason or prove the server implementation correct. These initial logs justified switching to a GATT inspector; the separate
write-type defect below subsequently required the compatibility correction. Those early attempts did not establish acceptance.

### Write-operation compatibility defect

A further operator-reported attempt, `01-round-trip_4`, connected but expired
at readiness without an observed subscription (total probe elapsed 32447 ms;
probe exit 1, shared services active). Later supplied output reaches
`subscribed confirming=true` and `ready exchange_limit_s=120`, then reports
`Bluetooth request not supported` on RX. The operator reports the public token
as byte-array `0x51544231`, with the same rejection for Request and Command.
These failed attempts established subscription/readiness but no confirmed
round trip. Full runtime/client attribution was not supplied.

Pinned-source inspection identifies the interoperability defect: BlueZ 5.86's
`src/gatt-database.c` forwards device/offset/link/MTU/prepare metadata but no
write `type`. BlueR 0.17.4 parses absent `type` through `WriteOp::default()`,
which is `Command`. The validator rejects it, including a genuine ATT Request.
The 18 unit tests supplied `Request` directly and missed this D-Bus boundary.

The operator later supplied the same rejection after readiness, then link loss
at 86097 ms; exit 1 and shared services still active. Inspection of release
`build/releases/rt-diagnostic-ble-20261124T155606Z/` confirms its `bluez5.env`
lacks the patch and its source archive contains no BlueZ adaptation. Rebuilding
those unfixed inputs reproduced the defect. Runtime hashes for that physical
attempt were not supplied, so exact deployment attribution remains unconfirmed.
Both earlier releases ending `114539Z` and `155606Z` are retained as unfixed.

The correction now lives in `meta-qtooth-ble`'s BlueZ 5.86 append and patch.
It preserves the original characteristic ATT opcode through ordinary and
acquired-write fallback paths: Request → `request`, Command/signed Command →
`command`, Prepare/Execute → `reliable`. Descriptor/read metadata and upstream
pins remain unchanged; omitted or unknown types remain rejected. Rejection logs
now include only operation, prepare status, offset, MTU and length, without
payloads or peer addresses. G1 selections exclude the BLE layer/patch.

Regression coverage now starts a private D-Bus daemon and fake GATT manager,
registers the production application with actual BlueR 0.17.4, discovers RX and
calls its real `WriteValue` method. Missing type reproduces the original failure;
Command, reliable and prepare are rejected without a handoff; ordinary Request
accepts the exact token at MTU 23. This tests the real parser and callback, with
synthetic peer/readiness metadata and no radio. Its environment is confined to a
child process and never changes the host system bus. The initial sandbox denied
the private socket; the approved run outside that restriction passes all 19 tests.

`tools/check-ble-write-type <patched-BlueZ-source>` extracts the actual patched
helper, compiles it against the pinned ATT constants and checks all 256 opcode
values. Mapping, Clippy, formatting and selector checks pass. The first ARM64
compile caught an additional `pending_write_new` caller in the acquired-write
path; its opcode propagation was corrected before the successful full build.
Failed logs remain private. The collector also verifies installed `bluetoothd`
bytes against the exact patched IPK and records daemon/package/patch hashes.
A compiled boundary check still does not prove a confirmed physical exchange.

## First successful physical round trip

The operator reports `01-round-trip` passing after the corrected-image
instructions. Its log records four RX bytes, observed ATT MTU **517**, four TX
bytes with `confirmed=true`, cleanup and `transport_success=true`, with probe
exit **0** and both shared services active (`services_exit=0`). Total elapsed
**32823 ms** covers the interactive probe run, not isolated radio latency.

The supplied phone screenshot independently displays `51-54-42-31` (`QTB1`)
on both documented RX (`WRITE`) and TX (`INDICATE`) characteristics, with CCCD
indications enabled. Its final disconnected state is consistent with successful
probe cleanup. The screenshot does not show versions; the operator separately confirms
nRF Connect 4.29.1 and Android 17 on the Pixel 7a. `authenticated=false` remains correct: this is a
public-token transport test, with no authentication or decryption.

The original screenshot contains an operator device address and stays in ignored
local/build storage. The exact supplied log, copied screenshot, SHA-256 manifest
and context are retained under an ignored `physical-success-*` directory.
No published screenshot or peer identifier is added, and the frozen release is
unchanged. The operator confirms matching baseline hashes for the corrected
release above and successful Imager verification; the raw baseline log was not
supplied.

**Round-trip scenario: PASS (operator log plus phone observation).**

## Failure, cancellation and repeatability checks

The operator reports all tests successful, with incorrect phone manipulation on
the first case 08 attempt and a successful retry. Every supplied negative-case
log records cleanup, `transport_success=false`, probe exit 1, both shared
services active and `services_exit=0`. No successful exchange is reported in
these negative traces. Preserve the first case 08 attempt as a procedure issue,
not a successful unsubscribe test or an application failure.

| Case | Observed result | Whole-run elapsed ms | Evidence |
|---|---|---:|---|
| 02 no phone | PASS: readiness timeout | 30125 | Supplied log |
| 03 subscribed/no write | PASS: exchange timeout | 133760 | Supplied log |
| 04 invalid token/length | PASS: four-byte token denied; three-byte value rejected | 45367 | Supplied log |
| 05 before subscription | PASS: Request denied before TX subscription | 24716 | Supplied log |
| 06 early disconnect | PASS: link lost before readiness | 18840 | Supplied log |
| 07 ready disconnect | PASS: link lost after readiness | 18892 | Supplied log |
| 08 first attempt | Procedure issue: no observed subscription; readiness timeout | 32107 | Supplied log; operator explains incorrect manipulation |
| 08 retry | PASS: subscription lost after readiness | 16575 | Supplied log |
| 09 Ctrl-C | PASS: cancelled and cleaned up | 21248 | Supplied log |
| 10 SIGTERM | PASS, operator-reported | Unavailable | No saved output |
| 11 exclusive ownership / owner round trip | PASS, operator-reported | Unavailable | No saved output |

Case 03's 133.760 s covers setup, connection/subscription, the 120 s exchange
window and cleanup. It is not a measured 133.760 s exchange timeout. Logs do
not include per-phase timestamps. The 02 no-phone observation is 30.125 s for
the complete run. Neither observation is a hard real-time guarantee.

The exact pasted negative logs, both case 08 attempts, operator statements and
later client/hash/Imager confirmations are retained with checksums under ignored
`physical-failure-checks-*` storage. A separate final archive in
`local/g2-readme-evidence/` includes these confirmations; the earlier archive and
original successful-round-trip screenshot/log remain retained for future G2
README visuals. No timing, exit code or phase trace is invented for 10/11.
Additional physical offset/prepare/reliable/no-response write variants are not
attributably logged for the corrected image; native validator/parser checks
cover their rejection. Physical MTU 23 remains unperformed.

## G2.1 acceptance

Accepted on 2026-11-24 from attributable build/package/raw-image checks, the
operator's deployment hash/read-back confirmations, identified client/phone,
matching phone/Pi round-trip observations, supplied timeout/rejection/link-loss/
Ctrl-C logs, repeated fresh probe invocations and reported SIGTERM/ownership/
owner-round-trip success. Cases 10/11 and per-failure confirmed restart traces
have weaker evidence than the saved cases; this acceptance makes that limit
explicit. No rebuild/reflash or code change is required by these results.
The published summary contains no operator addresses or workstation paths;
protected SoT/diagram hashes, whitespace checks and strict OpenSpec validation
pass. This accepts **G2.1 transport only**. G2.2/G2.3, BLE-2/4/5 and the G2 gate
remain open; there is no authentication/decryption claim.

Learning demonstrated: distinguish callback acceptance from confirmed transport;
bound asynchronous state through ownership and monotonic deadlines; separate
metadata/build attribution from physical delivery. Next experiment after the
physical G2.1 gate: bounded application framing, before authenticated PQC/AEAD.

## Documentation after archive

The two original `docs/labs/g2-1-ble-*` guides were consolidated into the draft
[G2 Architecture and Practical Guide](../../../../docs/courses/G2_Architecture_and_Practical_Guide.md).
Its reproducible labs, implementation explanation and result table replace those
guides; archived planning/task paths record their original creation. Original
images, screenshots, logs and checksum archives remain private and unchanged.
A separately supplied redacted phone screenshot is included in that course;
the original unredacted evidence remains private.
