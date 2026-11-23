## Context

See proposal.md for motivation. Pinned OE-Core provides OpenSSH and systemd-networkd. Diagnostic access uses an operator-selected Ethernet interface and address discovery on a direct cable. Base remains locked and has no interactive access path.

This design is required because SSH account policy, packaging and network/service startup interact. It conforms to DESIGN BASE-1..3/G1 and HARDWARE HW-1..4; protected authorities/figures remain unchanged.

## Goals / Non-Goals

**Goals:** provide a reproducible first physical diagnostic login and a truthful path through remaining G1 checks.

**Non-Goals:** change the Windows adapter, add an internet-sharing server, use the Bluetooth UART for debug, implement BLE/FPGA applications, or accept physical measurements before they exist.

## Decisions

1. A diagnostic-only configuration package installs an operator-provided public authorization input from ignored `local/ssh/authorized_keys`, a networkd file ordered before `80-wired.network`, and an OpenSSH configuration drop-in. The tracked fallback contains comments only; diagnostic builds reject missing, private or malformed authorization. No operator identity is distributed in source or documentation. Dependencies explicitly include sshd and networkd, while base requires no authorization input.
2. Use DHCPv4 plus `LinkLocalAddressing=ipv4`. DHCP-only cannot guarantee an address on the direct cable. A fixed Pi address alone would retain dependence on the unresolved Windows configuration. Let networkd perform address-conflict detection; Wireshark observes DHCP/ARP, Windows provides the first SSH client. No expected address is claimed before capture.
3. Permit root public-key authentication for privileged diagnostics; disable password/interactive authentication. The diagnostic image sets root's shadow password field to the non-authenticating `*` marker so OpenSSH's locked-account check does not reject key authentication. No empty-password image feature is enabled. Install authorized_keys with root ownership and restrictive permissions; the client private key stays on Windows. Host private keys are generated on first boot by the pinned sshd service dependency.
4. Keep the existing pinned BSP, kernel profiles and deployment DT/storage layout. Build both diagnostic profiles with the same access package and record changed manifests/hashes. Preserve previous QEMU cohorts before rebuilding their output directories. No new Rust/FFI or kernel implementation is needed; networkd/sshd are userspace services over the kernel's Ethernet/socket interfaces.
5. Execute hardware checks incrementally: first login/attribution; controller initialization and DT/binding; then matched measurements. First-boot evidence and raw captures go under ignored build roots, with concise reviewed observations in a bring-up report. Existing guest helpers are emulation-specific and cannot be treated as ready-made hardware collectors.
6. Base-image observation is a mandatory later gate: establish an access/collection method consistent with BASE-2 before collecting its RAM/boot observations. Do not install diagnostic SSH/stress packages into the base image or substitute diagnostic values. Record that gate as unresolved until a concrete compatible method is available; first diagnostic bring-up is independent of it.
7. Physical boot exposed a missing runtime dependency: `CONFIG_CRYPTO_CMAC=m` but no CMAC module in the rootfs, causing Bluetooth SMP to fail allocating `cmac(aes)`. Add `kernel-module-cmac` to the Pi shared runtime package group, preserving both kernel profiles and base access policy. Retain the first-boot image/evidence before rebuilding. A matching RT module can be temporarily loaded on the board to confirm the fix; treat that session as patched and require a corrected deployment image for clean-boot evidence.
8. The physical audit confirmed disabled I2C1/SPI0 DT nodes despite installed drivers. Compile a project-owned boot overlay separately using dtc-native; select it through firmware config and make WIC depend on its deployment. Enable I2C1 at 100 kHz and SPI0 with only HW-2 CE0/GPIO8; disable stock spidev children so the later FPGA driver can own CE0. Install a modules-load file for i2c-dev in all Pi variants. Preserve the base DTB, Bluetooth UART, kernel configuration and executable. Offline overlay composition checks establish intended DT changes; a physical reboot/binding check is still required. Any temporary on-card debug deployment must be backed up, hashed and distinguished from the newly built candidate.

## Risks / Trade-offs

Measurement collection decision: prepare offline instrumented copies of all four
audited WICs, keeping original exports and recipes intact. Add the same shell
recorder, existing physical inspector, systemd service/timer and activation link;
add source provenance to boot FAT. No package or account-policy changes, no base
SSH/test suite, and no kernel/DT/UART changes. This is temporary BASE-3 bench
instrumentation, not a new production base feature. Attribute derivative sizes
separately from original candidate sizes.

The timer starts at 120 seconds after kernel startup, ordered after normal
multi-user startup. Take three RAM snapshots 30 seconds apart, recording the
observer's RSS, clocks and temperature. Report kernel origin to systemd finish
as the boot interval, excluding EEPROM/firmware. The observer and its units have
nonzero overhead; snapshots are instrumented base observations, not untouched
base memory measurements. They do not quantify overhead by subtracting RSS.
Use identical instrumentation in the diagnostic comparison. Run the existing
read-only inspector after snapshots; on diagnostic only, run three alternating
idle/load pairs with fixed cyclictest/stress-ng parameters. Keep trial outputs in
tmpfs; copy results and a completion marker to FAT before automatic shutdown.
Record failures and fixture/smoke protocols explicitly; never accept emulator or
shortened smoke values as physical BASE-3 results.

- Link-local selection needs a live cable and compatible Windows addressing → capture ARP announcements, inspect Windows's active address, and test TCP/22 before SSH; discovery alone is not connectivity.
- Headless boot failures can be silent → preserve firmware/LED observations and return to boot diagnostics if no network traffic; do not infer an SSH fault.
- First-use SSH host identity is initially unverified → record the first fingerprint and later confirm it on the target; never silently discard host-key mismatch warnings.
- Diagnostic root login changes measurement conditions → report service/package differences and measure matching variants with defined settle/load conditions.
- Four physical image variants and all twelve latency trials are now verified. Retain measurement limits and distinguish later BLE application/electrical gates.

## Acceptance record

Physical verification completed with four attributed RAM/boot runs and twelve
successful 60-second diagnostic scheduling trials (720000 samples, no histogram
overflow). The operator confirmed all four Imager read-back verifications.
The operator explicitly directs acceptance on the assumption that supply current
is measured and within the seller-certified board supply range. No numeric current
reading or instrument trace is supplied; this is an authorized acceptance
assumption, not an independently recorded measurement. No SoT was rewritten.
The reusable, privacy-neutral closeout procedure and results are consolidated
as the final lab of `docs/courses/G1_Architecture_and_Practical_Guide.md`.

## Migration Plan

Freeze prior evidence, build and audit the new diagnostic WIC, export a decompressed image plus SHA-256 for Windows, then use Imager custom-image write/verify without Raspberry Pi OS customization. Boot and inspect through the real firmware path. Roll back by rewriting an identified earlier deployment image; preserve all failed attempts and distinct artifact hashes.
