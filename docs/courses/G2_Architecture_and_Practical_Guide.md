# G2 Architecture and Practical Guide

**QuantumTooth — a practical work class in authenticated post-quantum BLE**

**Evidence reviewed: 2026-11-25**

**G2 complete — accepted on the physical PQC-over-BLE demonstration. G3 is next.**

G2 adds a Rust BLE application to G1's Yocto Raspberry Pi platform and a
React Native Android receiver. The physical demonstration now shows encrypted
records becoming verified phone plaintext, including two messages under one
retained session. The first standalone attempt decrypts on the phone but ends
with a Pi error, retained as a follow-up issue. The user-authorized DESIGN BLE-5
boundary accepts the completed demonstration without requiring every additional
physical robustness check. This course consolidates the
transport lab, protocol design, validation procedure and capture instructions.

Read the [G1 course](G1_Architecture_and_Practical_Guide.md) first for Linux,
Kas/Yocto installation, kernel/firmware, SD deployment and verified SSH access.
Here the objective is to understand the new application and reproduce its labs.
The authorities remain [DESIGN BLE-1..5/G2](../../openspec/DESIGN.md),
[HARDWARE HW-1/HW-3](../../openspec/HARDWARE.md) and the synced
[BLE specifications](../../openspec/specs/ble-authenticated-protocol/spec.md).

## Laboratory map

1. [Milestones, equipment and execution contexts](#1-milestones-equipment-and-execution-contexts)
2. [Architecture and implementation map](#2-architecture-and-implementation-map)
3. [How BLE and authenticated PQC work](#3-how-ble-and-authenticated-pqc-work)
4. [Lab A — native checks and private provisioning](#4-lab-a--native-checks-and-private-provisioning)
5. [Lab B — build, verify and deploy the Pi image](#5-lab-b--build-verify-and-deploy-the-pi-image)
6. [Lab C — reproduce the transport experiment](#6-lab-c--reproduce-the-transport-experiment)
7. [Lab D — build the Android receiver and exchange encrypted messages](#7-lab-d--build-the-android-receiver-and-exchange-encrypted-messages)
8. [Lab E — capture and interpret the exchange](#8-lab-e--capture-and-interpret-the-exchange)
9. [Recorded physical and software results](#9-recorded-physical-and-software-results)
10. [Remaining physical validation and troubleshooting](#10-remaining-physical-validation-and-troubleshooting)

Afterward, you should be able to trace source → image → GATT → shared crypto
core → verified UI, explain fragmentation and retained ownership, and distinguish
build evidence, transport completion and actual receiver verification.

## 1. Milestones, equipment and execution contexts

| Stage | Implemented and demonstrated | Boundary |
|---|---|---|
| G2.1 | Public-token BLE round trip; rejection, timeout and cleanup cases | Accepted at physical MTU 517; two cases have operator reports only |
| G2.2 | Authenticated ML-KEM-768 establishment, AES-GCM records, bounded framing, retained worker | Native real-crypto tests and Yocto builds accepted |
| G2.3 | Standalone ARM64 Android debug app with shared Rust core | APK build accepted; supplied GIF now demonstrates physical phone decryption and retained sequences 0/1 |
| Overall G2 — complete | Pi → BLE ciphertext → verified Android plaintext; retained sequences 0/1 | Accepted under revised DESIGN BLE-5; additional robustness checks are follow-up work |
| G3 — next | Board/pinout qualification, representative FPGA synthesis/fit/timing and Rust kernel API feasibility | To be established before further FPGA integration |

Equipment is G1's Raspberry Pi 4 Model B (4 GB), approved supply/cooling,
microSD, Ethernet/SSH access, and Pixel 7a running Android 17. The initial
transport inspector was nRF Connect 4.29.1. No FPGA or wiring change is needed;
keep the Bluetooth UART available and SPI CE0 reserved for later FPGA work.

| Label | Execution context |
|---|---|
| HOST | Linux/WSL Bash at the repository root; Docker/Kas, Rust and Android builds |
| CONTAINER | Pinned Kas shell; `/repo`, `/work`, `/build` are fixed build mounts |
| PI A / PI B | Two root SSH terminals on the diagnostic Pi |
| WINDOWS | PowerShell and Windows ADB when Windows owns the phone's USB connection |
| PHONE | Foreground nRF Connect for Lab C; QuantumTooth app for Lab D |

Keep workstation locations in environment variables, not documentation. Raw
logs, traces, provisioned images/APKs and signing inputs belong in ignored
`local/` or `build/`. Verify SSH keys through G1's procedure; do not publish
key material, fingerprints, device addresses or provisioned identities.

## 2. Architecture and implementation map

```text
Kas machine + RT profile + diagnostic image + BLE feature
  → BitBake recipes → ARM64 qtooth-ble + patched BlueZ + system libcrypto
  → root filesystem → WIC → SD → Raspberry Pi

Pi CLI → restricted Unix socket → Rust session worker → BlueR → D-Bus → BlueZ
                                      ↕ shared Rust core               ↕ BLE
Android UI ← typed events ← Kotlin GATT module ← JNI Rust adapter ← GATT
```

A Yocto layer is build metadata. BlueZ is the Linux Bluetooth daemon; BlueR is
the Rust D-Bus client. Our Rust application owns the custom protocol, not the
kernel radio stack. The phone is central/GATT client; the Pi is peripheral/GATT
server. The BLE feature is opt-in for Pi diagnostic RT/non-RT images; unsupported
base/QEMU BLE combinations fail visibly. There is no boot-started BLE worker.

| Files, relative to the repository | Responsibility and reason |
|---|---|
| `kas/base{,.lock}.yml`, `kas/machines/raspberrypi4-64{,.lock}.yml` | G1's pinned distro, OE/BitBake and Pi BSP inputs |
| `kas/profiles/{rt,nonrt}.yml`, `kas/images/diagnostic.yml` | Kernel policy and diagnostic image selection |
| `kas/features/ble.yml`, `layers/meta-qtooth-ble/conf/layer.conf` | Register the BLE layer and append `qtooth-ble` to the diagnostic image |
| `layers/meta-qtooth-ble/recipes-connectivity/qtooth-ble/` | Cargo recipe and checksummed crate inputs; frozen ARM64 build, install and runtime dependencies |
| `layers/meta-qtooth-ble/recipes-connectivity/bluez5/` | Feature-local BlueZ 5.86 write-type compatibility patch |
| `components/ble/Cargo.toml`, `Cargo.lock`, `build.rs` | Pi workspace/dependency lock and optional private-pair embedding |
| `components/ble/src/main.rs`, `ownership.rs`, `state.rs` | CLI/signals, exclusive lock and original transport validation/deadlines |
| `components/ble/src/probe.rs`, `probe/{tests,dbus_tests}.rs` | Probe GATT callbacks, confirmation/cleanup and real D-Bus parsing regression |
| `components/ble/src/probe/secure.rs`, `probe/secure/dbus_tests.rs` | Secure GATT transport and parser-boundary checks |
| `components/ble/src/worker.rs`, `worker/tests.rs` | Worker-owned retained session, local IPC, command lifecycle and process checks |
| `components/ble/core/src/framing.rs` | Fixed-capacity fragment encoding/reassembly and object deadlines |
| `components/ble/core/src/{protocol,crypto,kem}.rs`, `protocol/tests.rs` | Canonical handshake/records, HMAC/HKDF and narrow OpenSSL KEM boundary |
| `app/src/App.tsx`, `app/src/specs/NativeQTooth.ts` | Screen and typed native commands/events |
| `app/android/app/src/main/java/org/quantumtooth/proof/` | Kotlin GATT, permissions, lifecycle, ordered callbacks and native ownership |
| `app/native/{Cargo.toml,Cargo.lock,build.rs,src/lib.rs}` | Independent JNI workspace reusing the shared core, without Linux BlueR/D-Bus |
| `app/scripts/build-native.py`, `app/native/rust-toolchain.toml` | Pinned NDK/OpenSSL/Rust build integration and input tracking |
| `app/package{,-lock}.json`, `app/{metro,babel}.config.js`, `app/tsconfig.json` | Locked JS dependencies, bundle transforms and TypeScript checking |
| `app/android/` Gradle files, wrapper, manifest and resources | Codegen, ABI/SDK selection, native alignment, packaged JS, permissions and debug APK |
| `tools/build-selection`, `tools/{build,kas}`, `tools/build-tool.env`, `tools/vendor/kas-container` | Validate selection, mount caches and execute the pinned build environment |
| `tools/collect-{build,ble}-evidence` | Attribute source, boot files, executable/package/rootfs, system crypto and patched BlueZ |
| `tools/check-{build-selection,ble-write-type}` | Selector regressions and all 256 ATT opcode mappings |
| `docs/diagrams/{g2-pqc-over-ble,pqc-over-ble-*}.{puml,svg}` | Editable sequences and rendered illustrations |
| `docs/experiments/` | Reviewed public screenshot, annotated trace and phone demonstration GIF |
| `openspec/specs/ble-*/`, `openspec/changes/archive/*g2*/` | Requirements and dated implementation/acceptance history |

The recipe resolves repository sources through `FILESEXTRAPATHS`/`SRC_URI`,
uses OE's Cargo classes with frozen dependency inputs, and installs
`/usr/bin/qtooth-ble`. `DEPENDS` supplies cross-build headers/libraries;
runtime dependencies install BlueZ/D-Bus. The Pi links system
`libdbus-1.so.3` and OpenSSL `libcrypto.so.3`; Android statically packages
NDK-built OpenSSL 3.5.9 into its JNI library because Android's system crypto
is not the selected provider. G1's kernel, firmware and boot policy are reused.

### Why the BlueZ patch was necessary

BlueZ 5.86 omitted `type` from server D-Bus `WriteValue` options. BlueR 0.17.4
interpreted the missing field as Command, so strict validation rejected real
Write Requests. The first 18 unit tests constructed Request directly and missed
this boundary. The patch forwards actual ATT opcodes as `request`, `command`
or `reliable`, including acquired-write fallback. Unknown/missing type still
fails; the fix does not relax the validator.

A nineteenth test uses BlueR's actual parser on a private D-Bus. A helper checks
all 256 opcode values. The target compile also caught an additional acquired-write
caller. Inspect the patched package revision `r0.qtoothble1` and daemon hash:
the displayed version remains 5.86. Reflashing rebuilt but unfixed inputs had
reproduced the error; a new image checksum alone could not fix it.

## 3. How BLE and authenticated PQC work

**GATT** organizes services/characteristics. **ATT** carries reads, writes and
indications. A **UUID** identifies each service/characteristic. The phone enables
Pi TX indications by writing the **CCCD** subscription descriptor. RX is phone
→ Pi Write Request with response; TX is Pi → phone indication with confirmation.
Connection alone does not establish subscription, authentication or decryption.

| Interface | Advertised name | Service / RX / TX UUID prefixes |
|---|---|---|
| Public probe | `QTooth Probe` | `b6631000` / `b6631001` / `b6631002` |
| Authenticated protocol | `QTooth Secure` | `b6632000` / `b6632001` / `b6632002` |

All prefixes use suffix `-7e7b-4fb5-9dc4-93f4152a0e7a`. A generic inspector
can perform the probe, but cannot implement the secure handshake.

![BLE roles and subscription](../diagrams/pqc-over-ble-transport.svg)

### From a key exchange to verified plaintext

1. Phone sends **ClientHello**: protocol/suite, configured identities and a fresh nonce.
2. Pi returns **PiOffer**: fresh nonce, ephemeral ML-KEM-768 public key and PSK HMAC.
3. Phone authenticates Pi, encapsulates a shared secret and sends **PeerFinish**:
   KEM ciphertext, transcript authentication and derived-key confirmation.
4. Pi decapsulates, verifies the transcript/confirmation and sends **PiFinish**.
   Phone verifies it; both now have confirmed session keys.
5. Pi encrypts the complete application record with **AES-256-GCM**, fragments
   it for ATT and sends it. Phone reassembles and verifies the tag before releasing
   UTF-8 plaintext through JNI to the screen.

ML-KEM's 1184-byte public key and 1088-byte ciphertext establish a 32-byte shared
secret. That KEM ciphertext is different from the encrypted user message. The
pair-specific 256-bit PSK authenticates the canonical transcript with HMAC-SHA-256;
ML-KEM by itself does not identify the intended peer. HKDF-SHA-256 derives separate
directional traffic/confirmation keys, nonce prefixes and a session identifier.
Version, roles, identities, nonces and KEM objects are bound to the exchange.

AES-GCM authenticates 29 bytes of record metadata (session, direction, type,
sequence and length), encrypts the message, and appends a 16-byte tag. A record
is plaintext length + **45** bytes. Each direction has its own sequence/nonce;
replayed sequences and wrong session/direction fail. Temporary plaintext remains
private until tag verification succeeds. This is an experimental application
protocol; the repository contains no independent cryptographic audit.

![Authenticated PQC exchange](../diagrams/pqc-over-ble-pqc.svg)

### Why application fragmentation is required

Ordinary ATT value budget is `min(MTU - 3, 512)`. Each value includes this
8-byte application header, with multibyte integers in big-endian order:

```text
version:u8 | kind:u8 | object-id:u16 | total:u16 | offset:u16 | chunk bytes
```

Object bytes per fragment are therefore **12 at MTU 23**, **504 at MTU 517**.
Fragment count is `ceil(object_bytes / object_budget)`. Fixed 2048-byte receive
storage rejects gaps, overlap, duplicates, unexpected kinds/IDs and excessive
sizes before copying; invalid traffic does not refresh its deadline. Encrypt
first, fragment second; only the complete authenticated record can release text.

| Logical object | Kind | Bytes | MTU 23 fragments | MTU 517 fragments |
|---|---:|---:|---:|---:|
| ClientHello | 1 | 68 | 6 | 1 |
| PiOffer | 2 | 1249 | 105 | 3 |
| PeerFinish | 3 | 1152 | 96 | 3 |
| PiFinish | 4 | 32 | 3 | 1 |
| `Hello, world!` record | 5 | 58 | 5 | 1 |
| Maximum 512-byte plaintext record | 5 | 557 | 47 | 2 |
| Authenticated close record | 5 | 45 | 4 | 1 |

These are application fragments carried in ATT values, not radio packet counts.
The [version-1 registry](../../openspec/changes/archive/2026-11-24-g2-2-authenticated-pqc-sessions/registry.md)
and [public fixtures](../../openspec/changes/archive/2026-11-24-g2-2-authenticated-pqc-sessions/fixtures.json)
retain exact encodings and explicitly disposable test inputs.

### Commands, ownership and lifetime

| Command | Behavior |
|---|---|
| `qtooth-ble probe` | Unauthenticated four-byte `QTB1` write/indication, then cleanup |
| `qtooth-ble send 'text'` | Fresh handshake, one encrypted record, cleanup; rejected if a kept session exists |
| `qtooth-ble sendkeep 'text'` | Establish once or reuse worker-owned keys with a fresh sequence/nonce; retain after successful local result handoff |
| `qtooth-ble closekeep` | One authenticated close attempt, unconditional local teardown; absent session is a successful no-op |

The root-only on-demand worker retains connection, keys and counters after a
`sendkeep` CLI exits. A restricted Unix socket serializes local commands; a
kernel file lock excludes competing owners. Keys do not return through IPC or
persist to disk. Losing IPC before result handoff discards the session. Never
unlink the lock to force a restart: that can create independent lock owners.

| Secure protocol bound | Limit and meaning |
|---|---|
| Establishment | 30 s from advertising through mutual key confirmation |
| Session | Absolute 120 s from handshake completion; traffic never extends it |
| Logical object | 10 s, clipped by establishment/session expiry |
| Individual ATT operation | 5 s, clipped by enclosing deadline |
| Courtesy close / cleanup | 2 s each; close cannot extend session expiry |
| Local startup / IPC operations | 5 s, with bounded frames/queues |

The legacy probe instead uses 30 s readiness, 120 s exchange **after readiness**,
up to 10 s setup operations and 3 s link cleanup. Both paths use monotonic time
and leave shared BlueZ/D-Bus running. These are userspace bounds, not measured
hard real-time radio guarantees.

Success on Pi is `transport_success=true delivery=unconfirmed`: completed ATT
operations do not prove remote decryption. There is no application ACK, retry or
automatic reconnect. Only the phone's tag-verified result demonstrates decryption.

![Command and session ownership](../diagrams/pqc-over-ble-commands.svg)

## 4. Lab A — native checks and private provisioning

Complete G1's host setup first. Native checks need Rust/Cargo, a C toolchain,
pkg-config, D-Bus development files/daemon and OpenSSL >=3.5 with ML-KEM-768.
The recorded host used Rust 1.98.0 and OpenSSL 3.5.5; OE used Rust 1.94.1 and
OpenSSL 3.5.9. Follow [Android setup](../../app/README.md#setup-linux-x86_64)
for Node/npm, JDK 21, SDK/NDK and the pinned Rust Android target.

**HOST**, retain raw output in a fresh private directory:

```bash
set -euo pipefail
umask 077
g2_evidence="local/g2-validation/$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$g2_evidence"
pkg-config --modversion openssl dbus-1
cargo test --locked --manifest-path components/ble/Cargo.toml --workspace > "$g2_evidence/pi-host-tests.log" 2>&1
cargo test --locked --manifest-path components/ble/Cargo.toml --workspace g2_2 -- --nocapture > "$g2_evidence/native-g2-2.log" 2>&1
cargo test --locked --manifest-path app/native/Cargo.toml > "$g2_evidence/android-adapter-host.log" 2>&1
cargo fmt --check --manifest-path components/ble/Cargo.toml
cargo fmt --check --manifest-path app/native/Cargo.toml
cargo clippy --locked --manifest-path components/ble/Cargo.toml --workspace --all-targets -- -D warnings
cargo clippy --locked --manifest-path app/native/Cargo.toml --all-targets -- -D warnings
./tools/check-build-selection
```

Expect all checks to pass. The focused G2.2 test prints `decrypted=Hello, world!`
at **simulated** MTUs 23/517 using real ML-KEM/HMAC/HKDF/AES-GCM. Process checks
exercise one retained handshake, sequences 0/1, busy rejection, close, IPC loss,
expiry and signals. Core tests reject wrong PSK/identity/transcript/confirmation,
metadata/tag changes, replay, direction/session mismatch and malformed fragments.
The two Android adapter tests cover fragmented establishment, retained records,
close/expiry and tag tampering. Native simulation is not a physical radio test.

<a id="10-g22--authenticated-native-experiment"></a>
<a id="private-provisioning-and-image-build"></a>
### Private provisioning and image build

Both builds consume ignored `local/ble/pair.bin`: **64 bytes**, comprising PSK:32,
peer identity:16 and Pi identity:16. Reuse the established pair; never overwrite
it to resolve a mismatch. For a new, unpaired lab only, create it exclusively:

```bash
mkdir -p local/ble
chmod 700 local/ble
(umask 077; python3 - <<'PYPAIR'
from pathlib import Path
import secrets
with Path('local/ble/pair.bin').open('xb') as pair:
    pair.write(secrets.token_bytes(64))
PYPAIR
)
```

For an existing pair, check presence/size without displaying its contents:

```bash
python3 - <<'PYCHECK'
from pathlib import Path
pair = Path('local/ble/pair.bin')
assert pair.is_file() and pair.stat().st_size == 64, 'Restore the existing private pair'
print('Private pairing input: size valid')
PYCHECK
git check-ignore local/ble/pair.bin
```

Provision diagnostic SSH through ignored `local/ssh/authorized_keys` as in G1.
Compare the intended public key with `ssh-keygen -lf` locally, retaining no real
output in published docs. Without pairing input the Pi still supports `probe`
but data commands fail before advertising; Android rejects missing/invalid input.
BitBake tracks optional-file existence and content changes; Cargo/Gradle track
pair changes too. Embedded credentials are extractable, so provisioned binaries,
APKs, images, caches and evidence remain private.

## 5. Lab B — build, verify and deploy the Pi image

Build, export and verify a separate release on **HOST**:

```bash
# @brief Build the opt-in image and freeze its complete deployment/evidence set.
set -euo pipefail
./tools/build --machine raspberrypi4-64 --profile rt --image diagnostic --feature ble --check
./tools/build --machine raspberrypi4-64 --profile rt --image diagnostic --feature ble
selection=build/raspberrypi4-64-rt-diagnostic-ble
deploy="$selection/tmp/deploy/images/raspberrypi4-64"
release="build/releases/rt-diagnostic-ble-$(date -u +%Y%m%dT%H%M%SZ)"
(cd "$selection" && sed 's|  /build/|  |' evidence/boot.sha256 | sha256sum -c -)
mkdir -p build/releases
mkdir "$release"
bzip2 -dc "$deploy/qtooth-image-diagnostic-raspberrypi4-64.rootfs.wic.bz2" > "$release/image.img"
cp -R "$selection/evidence/." "$release/"
(cd "$deploy" && sha256sum Image bcm2711-rpi-4-b.dtb qtooth-platform-buses.dtbo bootfiles/config.txt bootfiles/start4.elf bootfiles/fixup4.dat) > "$release/BUILD_BOOT_SHA256SUMS"
git diff --binary > "$release/source.patch"
tar -czf "$release/sources.tar.gz" components/ble/{Cargo.toml,Cargo.lock,build.rs,core,src} layers/meta-qtooth-ble/{conf,recipes-connectivity} kas tools LICENSE
(cd "$release" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS && sha256sum -c SHA256SUMS)
printf 'Verified release: %s\nImage to flash: %s/image.img\n' "$release" "$release"
```

BitBake creates images in `tmp/deploy/images/`; `tools/build` records evidence
in `evidence/`. The explicit export commands create `build/releases/…`; neither
Kas nor BitBake automatically makes that frozen release. `image.env` and
`image.manifest` are metadata, not flashable images. `sources.tar.gz` covers
untracked component/layer files that `git diff` alone would omit.

`SHA256SUMS` records a digest of each exact file. Verification recomputes and
compares it, detecting changed/copied/corrupted bytes relative to a trusted
manifest. It does not prove the software correct, authenticate a replaced
manifest or verify SD sectors. Imager read-back checks the written card;
runtime hashes identify what booted. Preserve all three kinds of evidence.

### 4.1 Inspect the automation's underlying operations

The resolved stack is
`kas/machines/raspberrypi4-64.yml:kas/profiles/rt.yml:kas/images/diagnostic.yml:kas/features/ble.yml`.
`tools/build` uses the pinned Kas container to dump it, parse BitBake, build
`qtooth-image-diagnostic`, dump expanded environments and run both collectors.
To inspect manually without editing generated configuration, on **HOST**:

```bash
# @brief Enter the same selected build environment for recipe/package inspection.
g2_config=kas/machines/raspberrypi4-64.yml:kas/profiles/rt.yml:kas/images/diagnostic.yml:kas/features/ble.yml
./tools/kas --machine raspberrypi4-64 --profile rt --image diagnostic --feature ble shell "$g2_config"
```

Then in **CONTAINER**, after the image build:

```sh
# @brief Inspect the selected layer, expanded patch inputs and package-owned files.
bitbake-layers show-layers
bitbake -e bluez5 | sed -n '/^PR=/p; /^SRC_URI=/p'
oe-pkgdata-util list-pkg-files qtooth-ble
exit
```

Inspect `kas-resolved.yml`, `parse.log`, `image.manifest`, `qtooth-ble.env`,
`bluez5.env`, `ble-elf.txt`, `ble-attribution.json`, `project-inputs.sha256` and
`boot.sha256` in the selected evidence directory. Expect ELF64 AArch64, system
`libdbus-1.so.3`/libgcc/glibc linkage, repository/recipe/package/rootfs identity,
patched BlueZ revision `r0.qtoothble1`, no target compiler and unchanged G1 RT
kernel/configuration. The collector also checks installed `bluetoothd` against
the exact patched IPK. During closeout both executables were independently
extracted read-only from the raw WIC rootfs using `debugfs` and matched.

For the patched source already unpacked by that build, on **HOST**:

```bash
# @brief Check the actual BlueZ helper against every possible ATT opcode byte.
./tools/check-ble-write-type build/raspberrypi4-64-rt-diagnostic-ble/tmp/work/cortexa72-oe-linux/bluez5/5.86/sources/bluez-5.86
```

Source directories may be absent after cache restoration/cleanup; that absence
does not establish a failed patch. Use retained recipe/package attribution;
run the helper check when patched sources are present. Non-RT BLE uses a
separate `build/raspberrypi4-64-nonrt-diagnostic-ble` root. Default selections
retain their old directories, package policy and unpatched BlueZ revision `r0`.


<a id="g2-physical-lab"></a>
### Flash, connect and attribute

Flash the printed `image.img` with Raspberry Pi Imager, apply no OS customization
and complete read-back verification. Reuse G1's verified SSH route. One encountered
failure was a host/Pi direct-Ethernet subnet mismatch: check the host interface,
route/ARP and TCP port 22 before changing keys or rebuilding the image.

**PI A**:

```sh
umask 077
mkdir -p /tmp/qtooth-g2-validation
uname -m
uname -r
zcat /proc/config.gz | sed -n '/^CONFIG_PREEMPT_RT=/p'
systemctl is-active dbus bluetooth
qtooth-ble --help
sha256sum /usr/bin/qtooth-ble /usr/lib/libcrypto.so.3 /usr/libexec/bluetooth/bluetoothd > /tmp/qtooth-g2-validation/runtime.sha256
bluetoothctl show > /tmp/qtooth-g2-validation/controller.txt
```

Expect AArch64, the selected RT kernel, active services and all four CLI commands.
Compare runtime hashes with this release's `ble-attribution.json`, including
provisioning and patched BlueZ attribution. Controller output must show LE/power
readiness; use `bluetoothctl power on` if necessary. Keep raw identifiers private.

Define this helper on **PI A**; it records the tested command's actual status
and refuses to overwrite a prior trial. A case is successful only if its recorded
outcome matches the expected behavior; the helper itself merely records it.

```sh
g2_case() {
    g2_case_name=$1
    shift
    case "$g2_case_name" in
        ''|*[!a-zA-Z0-9_-]*) printf 'Use a simple case label.\n'; return 2 ;;
    esac
    g2_case_log="/tmp/qtooth-g2-validation/$g2_case_name.log"
    if [ -e "$g2_case_log" ]; then printf 'Choose a new label.\n'; return 2; fi
    if "$@" > "$g2_case_log" 2>&1; then g2_command_exit=0; else g2_command_exit=$?; fi
    printf 'command_exit=%s\n' "$g2_command_exit" >> "$g2_case_log"
    if systemctl is-active dbus bluetooth >> "$g2_case_log" 2>&1; then g2_services_exit=0; else g2_services_exit=$?; fi
    printf 'services_exit=%s\n' "$g2_services_exit" >> "$g2_case_log"
    cat "$g2_case_log"
}
```

## 6. Lab C — reproduce the transport experiment

Disconnect the QuantumTooth app and run `qtooth-ble closekeep` before using
nRF Connect. **PI A**: `g2_case probe-01 qtooth-ble probe`.
Within 30 s, in the phone's foreground **CLIENT** view:

1. Scan/connect to **QTooth Probe**; discover service `b6631000-…`.
2. Enable **TX indications** on `b6631002-…`; expect the Pi to become ready.
3. Write Byte Array `51 54 42 31` to RX `b6631001-…` with **Write Request /
   response**, offset zero. This is public ASCII `QTB1`, not a key.
4. Confirm the identical TX indication; the Pi should exit zero after cleanup,
   reporting `transport_success=true authenticated=false`. Shared services stay active.

The write callback validates then hands off without blocking; a separate owned
future awaits indication confirmation. Merely queueing the echo cannot report
success. A four-byte value fits either MTU and proves no fragmentation.

<a id="7-lab-d--failure-paths-fresh-runs-and-evidence"></a>
### Rejection, timeout and cleanup manipulations

Run each with a new `g2_case LABEL qtooth-ble probe`. Negative trials should
exit nonzero, release the owned advertisement/GATT/link and keep services active.

| Manipulation | Expected result |
|---|---|
| No phone | Readiness timeout, 30 s from advertising |
| Subscribe but do not write | Exchange timeout, 120 s after readiness |
| Write `00 00 00 00`, then `51 54 42` | Wrong token and wrong length rejected; no echo |
| Write the correct token before subscribing | Request denied |
| Disconnect before readiness / after readiness | Link/subscription loss and cleanup |
| Subscribe, observe readiness, then disable indications | Subscription-loss failure |
| Press Ctrl-C / send SIGTERM to the sole active probe from PI B | Cancellation and cleanup |
| Invoke a second probe in PI B while PI A owns one | Second fails at lock; first can still complete |

After each failure, explicitly start a new trial and attempt the good exchange.
Retain failed manipulations and retries separately. Inspector refusal to transmit
offset/prepare/reliable/Command traffic does not prove Pi-side rejection; native
parser tests cover those paths. The archived transport results are in section 9.

## 7. Lab D — build the Android receiver and exchange encrypted messages

<a id="6-g23--minimal-android-proof-app"></a>
### Native verification and the screen

TypeScript presents status, a short ciphertext excerpt and verified plaintext.
Kotlin owns permission/lifecycle integration and serialized foreground GATT
callbacks. The independent Rust JNI workspace owns reassembly, authentication,
keys and decryption through the same `qtooth-ble-core` as Pi. No raw handshake,
PSK or session key reaches JavaScript. A native watchdog enforces deadlines
independently of UI scheduling; backgrounding/invalidation destroys native state.

The screen clears the prior result on a new attempt and retains a labelled last
verified result after disconnect. Only successful native `Session::decrypt`
populates it. Details contains complete text up to 512 UTF-8 bytes, observed MTU,
record RX fragments, authenticated sequence, complete record size and handshake
time. The preview is at most 24 application-ciphertext bytes.

Use [app setup and pinned dependencies](../../app/README.md#setup-linux-x86_64),
including JDK 21, SDK/build-tools 36, NDK 27.1.12297006 and Rust 1.98.0.
Export your SDK/JDK locations using placeholders from that guide; Gradle's
`local.properties` alone does not configure the native Python task.

**HOST**, with the same private pair used for Pi:

```bash
: "${JAVA_HOME:?Set JAVA_HOME to the JDK 21 installation}"
: "${ANDROID_HOME:?Set ANDROID_HOME to the Android SDK installation}"
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/platform-tools:$PATH"
npm ci --prefix app
npm run --prefix app typecheck
./app/android/gradlew -p app/android :app:assembleDebug > "$g2_evidence/apk-build.log" 2>&1
g2_apk=app/android/app/build/outputs/apk/debug/app-debug.apk
sha256sum "$g2_apk" > "$g2_evidence/apk.sha256"
"$ANDROID_HOME/build-tools/36.0.0/apksigner" verify --verbose "$g2_apk" > "$g2_evidence/apk-signing.txt"
"$ANDROID_HOME/build-tools/36.0.0/zipalign" -c -P 16 4 "$g2_apk"
unzip -l "$g2_apk" > "$g2_evidence/apk-contents.txt"
```

Gradle runs Codegen, NDK OpenSSL/static crypto, ARM64 Rust/JNI compilation,
JS/assets bundling and debug signing. Inspect for `assets/index.android.bundle`
and ARM64 libraries. All native LOAD/ZIP offsets must support 16 KB alignment;
use NDK `llvm-readelf -h -l -d` for packaged ELF inspection. The JNI library
must not depend on Android system libcrypto/libssl. Its crypto capability is
checked again at initialization. This debug variant bundles JS and disables
developer-server support, so it runs with **Metro stopped**.

### Install and launch — select the actual USB owner

If **Linux owns USB**, unlock the phone, authorize USB debugging, and run:

```bash
adb devices
adb install -r "$g2_apk"
adb shell am force-stop org.quantumtooth.proof
adb shell am start -n org.quantumtooth.proof/.MainActivity
```

If **Windows owns USB**, keep ADB in PowerShell. In WSL use
`wslpath -w "$(realpath "$g2_apk")"` and
`wslpath -w "$(realpath "$g2_evidence")"` to obtain Windows-readable locations;
keep actual output out of docs. Then **WINDOWS**:

```powershell
$g2SourceApk = '<APK Windows path printed by wslpath>'
$g2Evidence = '<evidence directory Windows path printed by wslpath>'
$g2Adb = '<Windows Android SDK platform-tools adb.exe>'
$g2InstallDir = Join-Path $env:TEMP ('qtooth-g2-' + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $g2InstallDir -ErrorAction Stop | Out-Null
$g2Apk = Join-Path $g2InstallDir 'app-debug.apk'
Copy-Item -LiteralPath $g2SourceApk -Destination $g2Apk -ErrorAction Stop
if ((Get-FileHash -LiteralPath $g2Apk).Hash -ne (Get-FileHash -LiteralPath $g2SourceApk).Hash) {
    throw 'APK copy checksum mismatch'
}
function g2_adb {
    & $g2Adb @args
    if ($LASTEXITCODE -ne 0) { throw "Windows adb failed: $LASTEXITCODE" }
}
g2_adb devices
g2_adb install -r $g2Apk
g2_adb shell am force-stop org.quantumtooth.proof
g2_adb shell am start -n org.quantumtooth.proof/.MainActivity
```

Select one authorized device if several are attached; keep its actual serial
private. Verify launch without Metro, keep Bluetooth on and the app foreground.
Grant Nearby devices when prompted. No Bluetooth-settings pairing is needed.

### Standalone, retained sends and fresh state

**PI A**, then **PHONE** Scan → select Pi → Connect selected Pi within 30 s:

```sh
qtooth-ble closekeep
g2_case secure-01 qtooth-ble send 'Hello, world!'
```

Expected phone result: exact verified text, sequence **0**, **58-byte** record;
normal cleanup retains the labelled result. Expected Pi result: exit zero and
`delivery=unconfirmed`. The supplied recording's Pi exit was **1** despite
successful phone decryption; preserve and investigate that distinction.

Start a new retained trial on **PI A**, manually Scan/connect again on **PHONE**:

```sh
g2_case secure-02 qtooth-ble sendkeep 'Second public message'
```

Wait for verified text at sequence **0**, then **PI A**:

```sh
g2_case secure-03 qtooth-ble sendkeep 'Bye Bye!'
```

Expect sequence **1** without another handshake; the record lengths are **66**
and **53** bytes. Those retained sends and successful Pi exits appear in the GIF.
To test the rest separately, try a conflicting `send`/`probe` (must fail without
disturbing the owner), run `qtooth-ble closekeep`, then start a new `sendkeep`
and connect explicitly. Expect fresh handshake/sequence 0; close when finished.
The recording does not establish this entire close/restart procedure.

## 8. Lab E — capture and interpret the exchange

Use two verified Pi SSH terminals and the paired foreground app. The diagnostic
image did not include `btmon`; a temporary ARM64 BlueZ 5.86 monitor was built
from retained pinned sources. The original helper/build evidence lives privately
under `local/g2-ble-capture/`; it is not a committed prerequisite. To repeat,
provide a compatible, privately built monitor matching the deployed image;
check its dependencies/version rather than replacing system libraries.

**HOST**, upload the existing helper through the verified G1 SSH connection:

```bash
read -r -p 'Verified Pi address: ' g2_address
read -r -p 'SSH client key filename: ' g2_key
test -f local/g2-ble-capture/btmon-aarch64
ssh -T -i "$g2_key" "root@$g2_address" \
    'umask 077; mkdir -p /tmp/qtooth-g2-capture; cat > /tmp/qtooth-g2-capture/btmon; chmod 700 /tmp/qtooth-g2-capture/btmon' \
    < local/g2-ble-capture/btmon-aarch64
```

Disconnect the app and run `qtooth-ble closekeep`. **PI A**:

```sh
umask 077
/tmp/qtooth-g2-capture/btmon --version
ls /sys/class/bluetooth
mkdir /tmp/qtooth-g2-capture/send-01
/tmp/qtooth-g2-capture/btmon -i hci0 -T \
    -w /tmp/qtooth-g2-capture/send-01/trace.btsnoop \
    > /tmp/qtooth-g2-capture/send-01/btmon-live.txt 2>&1
```

Expect version 5.86; select the actual controller if different. Choose a new
trial directory if one exists. If the monitor returns immediately, inspect its
error before continuing. While it runs, **PI B** executes
`g2_case capture-01 qtooth-ble send 'Hello, world!'`; **PHONE** scans/connects.
Record exact phone text, MTU, RX count and timing, plus the Pi's actual exit.
After the send ends/link closes, stop the monitor with Ctrl-C in **PI A only**.
Decode with:

```sh
/tmp/qtooth-g2-capture/btmon -P -T \
    -r /tmp/qtooth-g2-capture/send-01/trace.btsnoop \
    > /tmp/qtooth-g2-capture/send-01/trace.txt 2>&1
sha256sum /tmp/qtooth-g2-capture/send-01/trace.btsnoop
```

In Wireshark, filter `btatt`. Inspect MTU exchange, service discovery, TX CCCD
write, then kinds **1 → 2 → 3 → 4 → 5** from the fragment header. Useful filters
are `btatt.opcode == 0x12 || btatt.opcode == 0x13` (Write Request/Response) and
`btatt.opcode == 0x1d || btatt.opcode == 0x1e` (indication/confirmation).
Distinguish CCCD writes by handle; discovery may be cached. Count ATT values
after HCI/L2CAP reassembly, not individual ACL packets.

The public [annotated excerpt](../experiments/pqc-over-ble-cap.txt) records MTU
517 and fragment counts **1 / 3 / 3 / 1 / 1**. ClientHello at 51.719098 s to
PiFinish at 52.012336 s spans **293.238 ms**; through its ATT confirmation at
52.065823 s spans **346.725 ms**. These are one trace's host/controller timestamp
intervals. The app's 374/491 ms readings use ClientHello submission → verified
PiFinish in separate recorded sessions. Do not combine them into a benchmark.
The excerpt's frame lengths include capture/protocol overhead; logical object
sizes are given in section 3. HCI capture does not measure RF retransmissions.

### Retrieve, hash and publish only reviewed evidence

**HOST**, retain the trace and case logs before Pi reboot:

```bash
ssh -T -i "$g2_key" "root@$g2_address" \
    'tar -C /tmp/qtooth-g2-capture/send-01 -cf - .' > "$g2_evidence/ble-send-01.tar"
ssh -T -i "$g2_key" "root@$g2_address" \
    'tar -C /tmp/qtooth-g2-validation -cf - .' > "$g2_evidence/pi-logs.tar"
(cd "$g2_evidence" && find . -type f ! -name SHA256SUMS -print0 | sort -z | xargs -0 sha256sum > SHA256SUMS && sha256sum -c SHA256SUMS)
```

Extract the trace privately and compare its SHA-256 with the Pi's value. Capture
screens/recordings using the USB owner's tools; review every visible terminal,
phone screen and metadata before publishing. Keep raw traces private because
they contain addresses, identities and complete transcripts. Remove the Windows
temporary APK afterward with `Remove-Item -LiteralPath $g2InstallDir -Recurse`.
Public assets are the reviewed excerpt, redacted transport screenshot and GIF.

## 9. Recorded physical and software results

### G2.1 transport closeout — 2026-11-24

The corrected Yocto image was **60.84 MiB WIC.bz2 / 364.80 MiB raw**. Its build
completed **3813 tasks**, with **19 native tests** and zero parse errors.
The operator confirmed successful Imager read-back and matching baseline hashes;
raw baseline output was not supplied. Negative-case logs retain cleanup,
nonzero probe exit and active shared services. Times below cover the whole
interactive probe, including setup/cleanup, rather than isolated radio latency.

| Physical case | Observation | Whole-run ms | Evidence |
|---|---|---:|---|
| 01 round trip | Four RX/TX bytes `QTB1`, MTU 517, confirmed, exit 0 | 32823 | Pi log + screenshot |
| 02 no phone | Readiness timeout | 30125 | Supplied log |
| 03 no write | Exchange timeout after readiness | 133760 | Supplied log |
| 04 invalid | Wrong token and three-byte value rejected | 45367 | Supplied log |
| 05 before subscription | Correct-token Request denied | 24716 | Supplied log |
| 06 early disconnect | Link lost before readiness | 18840 | Supplied log |
| 07 ready disconnect | Link lost after readiness | 18892 | Supplied log |
| 08 initial attempt / retry | Incorrect phone manipulation timed out; retry observed subscription loss | 32107 / 16575 | Both retained logs |
| 09 Ctrl-C | Cancellation and cleanup | 21248 | Supplied log |
| 10 SIGTERM | Reported pass | — | Operator report only |
| 11 competing owner / owner round trip | Reported pass | — | Operator report only |

![Redacted nRF Connect proof: matching QTB1 RX/TX and indications enabled](../experiments/evidence_ble_phone.png)

The 133.760 s trial includes the 120 s exchange window **after readiness**.
Separate successful restart logs after every failure were not supplied. Physical
MTU 23 and additional write variants were not established by this closeout.
[Archived G2.1 verification](../../openspec/changes/archive/2026-11-24-g2-1-ble-conn-setup/verification.md)
retains exact attribution and the failed pre-patch attempts.

### G2.2 and G2.3 implementation verification

| Check | Recorded result |
|---|---|
| Shared Pi workspace | **41 tests passed**, including 22 focused G2.2 checks; fmt/Clippy passed |
| KEM capability | Real host and ARM64-emulated agreement; public/ciphertext/secret lengths 1184/1088/32; no secrets printed |
| Yocto G2.2 build | **3813 tasks successful**; frozen ARM64 sources, system OpenSSL 3.5.9 and patched BlueZ attributed |
| Provisioning changes | Missing/invalid handling and creation/change/removal rebuild invalidation checked |
| Android adapter | **2 host tests passed** at simulated MTUs 23/517; fmt/Clippy and TypeScript passed |
| Debug APK | Gradle build passed; **47,713,071 bytes**, ARM64 only, embedded JS, valid debug signing |
| Native packaging | **12 ARM64 libraries**, ELF/ZIP 16 KB alignment; static OpenSSL 3.5.9, no Android system crypto dependency |

These are dated results, not claims that checks were rerun for this documentation
update. Exact checksums and tooling remain in the
[G2.2 verification](../../openspec/changes/archive/2026-11-24-g2-2-authenticated-pqc-sessions/verification.md)
and [G2.3 build record](../../openspec/changes/archive/2026-11-25-g2-3-react-native-debug-app/verification.md).
Those archives describe their original evidence boundary; the physical evidence
below is a later addition.

### Supplied encrypted physical demonstration — reviewed 2026-11-25

![Pi commands beside Android ciphertext and tag-verified plaintext](../experiments/pqc-over-ble-evidence-vid.gif)

| Trial / artifact | Observed result | Limit |
|---|---|---|
| `send 'Hello, world!'` GIF | Phone verifies 58-byte record, sequence 0, one RX fragment, MTU 517; UI handshake **374 ms** | Pi subsequently prints `phase=cleanup error=failed`, `command_exit=1`; services remain active |
| `sendkeep 'Second public message'` GIF | Phone verifies 66-byte record, sequence 0; **491 ms** handshake; Pi exit 0 | One observed retained establishment |
| `sendkeep 'Bye Bye!'` GIF | Phone verifies 53-byte record, sequence 1; same 491 ms handshake display; Pi exit 0 | Confirms retained sequence advancement without a displayed new handshake |
| Annotated capture excerpt | MTU 517, discovery/subscription, four handshake kinds, 58-byte record/confirmation and disconnect | Curated transcript; raw private capture is not published; not assumed to be the GIF's session |

Both retained records show one RX fragment and active shared services. A repeated
case label was rejected locally before the final send; the unique-label retry
succeeded. No new plaintext was produced by that helper rejection.

**Result:** physical post-quantum key establishment and tag-verified Android
message decryption are demonstrated, including retained sequences 0/1. **G2 is
complete**, accepted 2026-11-25 under the user-authorized DESIGN BLE-5
demonstration boundary; **G3 is the next milestone**. The standalone Pi failure
is unresolved in the supplied evidence. Physical MTU-23, tamper/wrong-peer and
expiry/reconnect checks remain unperformed follow-up work and do not block G3.
The excerpt and recording are complementary evidence, not an application receipt
returned to Pi. Earlier archived acceptance records preserve their original scope;
the revised DESIGN governs current milestone status.

## 10. Remaining physical validation and troubleshooting

These are follow-up checks and known limitations, not G2 closure conditions or
prerequisites for G3. G2 is accepted on the demonstrated PQC-over-BLE exchange.

Use new case names and preserve actual outcomes under `$g2_evidence`; do not
mark the following procedures passed merely because the software implements them.

| Physical check | Manipulation / acceptance observation | Current evidence |
|---|---|---|
| Clean standalone completion | Repeat `send`; exact verified plaintext **and** Pi exit 0/active services | Phone decrypts; recorded Pi failure remains to diagnose |
| Exclusive secure ownership | During kept session invoke fresh `send` and `probe`; retained session must remain usable | Host checks; physical result not supplied |
| Close and fresh start | `closekeep`, then fresh `sendkeep`/manual connection; new handshake and sequence 0 | Not established by GIF |
| Maximum/invalid input | Send 512 ASCII bytes (557-byte record, two MTU-517 fragments); reject empty/513-byte input locally; next valid send advances only once | Host coverage; physical result not supplied |
| Permission/Bluetooth/scan failures | Deny Nearby devices, disable Bluetooth, scan without Pi; explicit recovery only | Physical result not supplied |
| Link/foreground/process failure | Disconnect, leave app foreground or force-stop; old state unusable; manual reconnect creates fresh sequence 0 | Physical result not supplied |
| Absolute expiry | Send at time 0 and again near 60 s; session still expires at 120 s from handshake; reconnect fresh | Host controlled-clock checks; physical result not supplied |
| Wrong PSK | Build an isolated private diagnostic APK with one changed PSK bit; same IDs; expect no verified text; restore paired APK and reconnect fresh | Host coverage; physical result not supplied |
| Physical MTU 23 | Use a peer/negotiation fixture that actually reports 23; repeat small/max records and observe counts 5/47 | Native simulation only |
| Physical record tamper/replay | Diagnostic GATT peer/proxy alters tag/ciphertext or repeats accepted same-session record; no new plaintext, state invalidated | Host coverage; production app has no injection mode |
| Timing characterization | Repeat independent fresh connections, report sample count/range and clock endpoints | Two UI samples and one separately scoped annotated trace |

For max-input trials on BusyBox, create public ASCII with
`g2_max=$(printf '%0512d' 0 | tr '0' X)` and send `"$g2_max"`. Rejected local
inputs must consume neither a sequence nor a nonce. For an isolated wrong-peer
build, snapshot app/core into a **new ignored local trial directory**, change
only its copy of `pair.bin`, build/install that diagnostic APK, then restore the
original paired APK. Never mutate the established provisioning to run a test.

The Android app requests MTU 517. A smaller application chunk cap does not prove
a negotiated MTU of 23, and repeated smaller Android MTU requests are not a
reliable fixture. Use an observed negotiation/transport trace. Capture at the
HCI boundary is also distinct from over-the-air RF evidence.

| Symptom / lesson encountered | Explanation and next check |
|---|---|
| Pixel Bluetooth tile cannot exchange custom data | Use nRF Connect CLIENT for probe; QuantumTooth app for secure handshake |
| Connected but readiness times out | Enable TX indications before the 30 s readiness deadline |
| Requests reported unsupported | Check actual BlueZ write-type patch and daemon hash, not version alone |
| APK cannot find SDK / needs Metro | Export SDK/JDK for the native task; build the bundled debug variant |
| Gradle resolver failure with template Gradle 9 | Recorded setup pins compatible Gradle 8.13/JDK 21; no dependency-source patch needed |
| Collector cannot find D-Bus package | Runtime manifest uses `dbus-1`; recipe and package names differ |
| Provisioned build has nondeterministic metadata | Optional input existence must be a BitBake parse dependency; contents must affect task signatures |
| Native D-Bus regression cannot open socket | Tests use a private local bus; the host must permit local IPC |
| Phone verifies but Pi exits 1 | Receiver verification and sender completion are separate; retain both observations and inspect worker/transport/cleanup errors before claiming a pass |

Before committing documentation, run `git diff --check` and
`openspec validate --specs --strict`; inspect text, image metadata and every
visible recording screen for local paths, identifiers, keys and credentials.

Explain-back: why does only the phone prove decryption? Which process retains
keys after a CLI exits? Why must encryption precede fragmentation? Which
robustness properties remain unmeasured after the accepted demonstration?
