## 1. Learn and establish the transport boundary

- [x] 1.1 Re-read both root SoTs and G1 acceptance, retain protected-file hashes privately, and explain the G2 milestone split; verify a short preflight records the existing Pi/BlueZ/D-Bus baseline and labels BLE-2/4/5 as pending.
- [x] 1.2 Inspect the pinned OE Rust/cargo classes, BlueR callback/indication APIs and D-Bus dependency; verify the selected versions support request MTU and confirmed indications, document host prerequisites and stop dependent work if the selected stack is infeasible.

## 2. Build, inspect and explain the Rust probe

Before each increment, explain its boundary and expected observation. The assistant
implements and verifies the code, adding useful comments and standard Doxygen tags. Optional explain-back questions never gate work.

- [x] 2.1 Create the standalone `components/ble` Cargo crate, locked dependencies and explicit `probe` CLI; verify `cargo check --locked --manifest-path components/ble/Cargo.toml` and CLI help, with no plaintext `send` implementation or service started at boot.
- [x] 2.2 Implement the documented service/RX/TX UUIDs, registration-before-advertising and fixed-token write validation; verify focused tests accept only `51 54 42 31`, offset-zero ordinary writes after subscription and reject wrong lengths/token, unsupported operations, pre-subscription writes and a busy handoff without unbounded queues.
- [x] 2.3 Implement indication-only response, confirmation-aware success and address-free phase/MTU/count reporting; verify a pending/failed indication cannot produce success and that callback completion does not wait for the indication acknowledgment.
- [x] 2.4 Implement the exclusive probe lock, 30 s readiness/120 s exchange limits and owned-resource cleanup; verify focused tests cover non-refreshing deadlines, cancellation/link loss, second invocation and fresh restart, then run `cargo test --locked --manifest-path components/ble/Cargo.toml` and formatting/lint checks. Explain why these tests do not prove radio connectivity.

- [x] 2.5 Preserve the actual characteristic ATT opcode as D-Bus write type using a BLE-layer-only BlueZ 5.86 patch; exercise the real BlueR D-Bus parser for ordinary Request acceptance and missing/Command/reliable/prepare rejection, then attribute the rebuilt daemon and verify default selections exclude the patch.

## 3. Build and inspect the opt-in Yocto feature

- [x] 3.1 Add BLE layer metadata and the Cargo recipe with locked crate URI/checksum inputs, repository-owned sources and system D-Bus linkage; verify BitBake parsing, frozen/offline compilation after fetch, ARM64 ELF dependencies and package ownership without vendored libdbus/OpenSSL or a target compiler.
- [x] 3.2 Add `kas/features/ble.yml` and propagate `--feature none|ble` through build selectors/wrappers with a separate BLE diagnostic build root; verify selector checks preserve every existing default path/configuration, support Pi diagnostic profiles and reject base/QEMU BLE combinations without modifying generated build configuration.
- [x] 3.3 Run `./tools/build --machine raspberrypi4-64 --profile rt --image diagnostic --feature ble --check`, then the same command without `--check`; inspect resolved Kas input, manifest, source/image hashes and package/dependency evidence, retaining failures privately and explaining recipe → binary → image attribution. Verify the original diagnostic/base defaults still exclude the feature.

## 4. Run, inspect and explain the physical experiment

- [x] 4.1 Write `docs/labs/g2-1-ble-connection.md` with prerequisites, exact build/verified-SD deployment steps, probe command, UUID/token table and generic inspector actions; verify the lab is self-contained, defines GATT/ATT/MTU/CCCD, states expected observations and offers the optional question about confirmation versus decryption.
- [x] 4.2 Deploy the attributable BLE diagnostic image using the G1 procedure and run `qtooth-ble probe` on the HW-1 Pi; on the foreground Pixel inspector scan/connect/discover/subscribe and write the public token with response. Verify identical indicated bytes, observed MTU, confirmation and successful cleanup; repeat no-phone timeout, subscribed/no-write expiry, unsupported write, disconnect, Ctrl-C and fresh-run checks, retaining raw logs and actual client/version privately. Report any unperformed physical MTU-23 check rather than inferring it.
  Evidence: phone/log round trip at MTU 517; cases 02–09 logged, including the valid case 08 retry. Cases 10/11 and owner round trip pass by operator report without saved output. Operator confirms nRF Connect 4.29.1, Pixel 7a/Android 17, matching baseline hashes and Imager verification. Physical MTU 23 and separately captured per-failure round trips remain unperformed/unavailable as disclosed in verification.

- [x] 4.3 Review measurements and publish a sanitized change-local `verification.md` with image/source attribution, transport evidence, failures/limitations and learned outcomes; verify protected hashes, repository privacy, `git diff --check` and `openspec validate g2-1-ble-conn-setup --strict`. Accept only G2.1 when its physical scenarios pass; explain the next experiment (bounded application framing), leaving G2.2/G2.3 and G2 acceptance open.
