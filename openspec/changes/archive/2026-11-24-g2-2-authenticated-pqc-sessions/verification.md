# G2.2 verification — 2026-11-24

G2.2's native/build acceptance boundary is met. `send`, `sendkeep` and
`closekeep` are implemented; `probe` remains separate. G2.3 and overall G2
remain open. No physical Pi/Pixel operation or SD deployment was performed.

## Reproduce and observe

From the repository root, with native OpenSSL >=3.5, Rust, C tools, pkg-config
and D-Bus development libraries/daemon:

```bash
cargo test --locked --manifest-path components/ble/Cargo.toml --workspace g2_2 -- --nocapture
```

**22 focused tests pass** (17 executable, five core), with real-crypto
`decrypted=Hello, world!` at simulated MTUs 23/517. Process tests use production
worker/client IPC paths and a test-only simulated BLE backend: two separate
retained clients share one establishment and sequences 0/1; fresh send fails
busy; explicit close and repeated no-op close succeed. Auto-launch, competing
operations, racing locks, restricted paths/credentials, malformed/oversized IPC,
IPC loss, SIGINT/SIGTERM, idle expiry and expiry during send pass. Test pair
inputs are random per run and isolated from production provisioning. An additional
process-harness trial verifies standalone send establishes fresh state, sends one
record, removes its worker/socket and leaves a later close as a no-op.

Core checks reject wrong PSK/identity/transcript/version/suite/confirmation,
metadata/tag tamper, replay, wrong direction/session, exhausted counters and
malformed/oversized/overlapping/duplicate/gapped/wrong-state/expired fragments.
Independent [encoding fixtures](fixtures.json) check the transcript and record
bytes; tests also cover 512-byte data, directional records and authenticated close.
Production secure callbacks run through BlueR's real D-Bus parser on a private
bus. Controlled clocks cover 30 s establishment, immutable 120 s lifetime,
object expiry, clipped 5 s ATT/startup/IPC waits and 2 s unreachable close.
A partial transfer fails without plaintext, retry or application receipt.

Transport success reports `delivery=unconfirmed`; it does not prove decryption.
The native receiver separately demonstrates tag-verified plaintext. Successful
local IPC result handoff lets the worker retain keys after CLI exit; loss before
handoff discards state. This local handoff is unrelated to BLE application ACKs.

## Native and build evidence

- Host Rust/Cargo 1.98.0; native OpenSSL/default provider 3.5.5. Real native and
  ARM64-emulated OpenSSL 3.5.9 KEM smoke checks report public/ciphertext/secret
  lengths **1184/1088/32**, with agreement. No secret bytes are printed.
- Locked workspace tests: **41 passed**, none failed/ignored. Cargo formatting
  and Clippy for all workspace targets with warnings denied pass. Locked
  metadata identifies the nested core; its normal tree has no BlueR or platform
  IPC/Android adapter. Unsafe code is confined to the narrow KEM module;
  size queries, checked outputs, RAII ownership and constant-time authentication
  comparisons were reviewed. Existing safe bindings handle key generation.
- `tools/check-build-selection` passes; `tools/check-ble-write-type` passes all
  **256** mappings against the actual patched BlueZ source. Default selections
  and the archived G2.1 transport contract remain intact.
- Both requested `tools/build --machine raspberrypi4-64 --profile rt --image
  diagnostic --feature ble` forms (`--check` and full build) pass. Final full
  build: **3813 tasks**, 3780 already current, all successful. OE Rust/Cargo
  1.94.1 builds frozen/locked sources for ELF64 AArch64.
- The collector verifies repository/shared-source equality, package/rootfs
  executable equality, the preserved patched BlueZ daemon/IPK, and system
  `libcrypto.so.3` bytes against the exact **libcrypto3 / openssl 3.5.9** IPK.
  Dynamic dependencies include system libssl/libcrypto, libdbus, libgcc and libc.
  No vendored OpenSSL source, target compiler or boot-enabled worker is present.

| Final unprovisioned artifact | SHA-256 |
|---|---|
| `usr/bin/qtooth-ble` | `87c1adbab021c3eac3ee805c41fd7aa5e90c92b0d769f0ed67e059624c6db40c` |
| `usr/lib/libcrypto.so.3` | `adb8b30035bf7ea4f594055cee63a4eec39704f797dbb46fc2c1e6b46004f804` |

Optional provisioning was exercised with disposable ignored inputs. Changing
only pair contents changes the native executable and Yocto compile signature;
the provisioned ARM64 build passes, with intended embedding checked privately.
Removing the input triggers a successful unprovisioned rebuild. Native and
built ARM64 CLI checks reject data before advertising and preserve `probe` help;
no test/default pair becomes a production fallback. Provisioned artifacts,
input, logs and sstate remain private; build/sstate directories are mode 0700.
Embedded PSKs remain extractable, and cleansing does not promise elimination
of every compiler/library copy.

## Failures corrected and evidence limits

The first target probe used a pruned recipe sysroot, then an incomplete assembler
PATH; the retained application sysroot/full OE toolchain fixed both. Initial
registry access needed sandbox network access. A callback test initially tried
to construct BlueR's non-exhaustive metadata; the real private D-Bus parser
checks replace that assumption. An auto-launch harness inherited a captured
stderr pipe and stalled after CLI exit; its simulated-worker launch now closes
that pipe. Affected test processes were stopped and the corrected suite passes.

The first provisioned build failed with nondeterministic metadata because
optional-file existence was absent from BitBake's parse dependencies. Registering
that dependency fixes creation/removal; SRC_URI content checksums propagate key
changes to compile/cache signatures. Subsequent provisioned/unprovisioned builds
pass without changing upstream pins or vendoring crypto. Failure replies now
avoid a blocking IPC write before teardown; startup connect is explicitly bounded.

Private raw logs, disposable pair evidence, provisioned IPK, signatures and
attribution stay under ignored `local/g2-2-authenticated-pqc-sessions/` and the
selected BLE build's `evidence/`. Published files contain no pair material,
real peer addresses, workstation paths or host-key fingerprints. Both root
SoTs and all ten protected derived assets match [preflight hashes](preflight.md).
Course Bash blocks, conceptual SVG syntax, whitespace and strict OpenSpec
validation pass. Historical G2.1 evidence and frozen releases are unchanged.

Native simulation and QEMU user emulation do **not** establish physical BLE
behavior. Unperformed: foreground Android/JNI/NDK integration, Pixel decryption,
physical MTU-23/large-MTU authenticated fragmentation, handshake cost, physical
wrong-peer/tamper/reconnect cleanup and remaining BLE-5 checks. No independent
cryptographic audit or hard real-time latency guarantee is claimed.

The next experiment is G2.3's foreground Android adapter and observed Pixel
plaintext. The [live course](../../../../docs/courses/G2_Architecture_and_Practical_Guide.md#10-g22--authenticated-native-experiment)
explains fragment → authenticated exchange → tag-verified plaintext and why
worker-owned counters survive separate CLI invocations. Optional explain-back:
why can a kept worker survive CLI exit while delivery remains unconfirmed?
