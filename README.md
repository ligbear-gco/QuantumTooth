# QuantumTooth

Minimal Yocto Linux RT platform for two independent demonstrations: post-quantum
communication over BLE and FPGA-assisted edge-ai error detection/recovery. **G1 platform
bring-up is complete**.

## Build and deploy an RT base image

From the repository root on Linux/WSL with Linux Docker, Git, Python 3.11+ and
bzip2, use a reviewed checkout. The wrapper uses the pinned Kas/container inputs.

```bash
# @brief Build RT base, verify recorded artifacts and export a partitioned SD image.
set -euo pipefail
./tools/build --machine raspberrypi4-64 --profile rt --image base
selection=build/raspberrypi4-64-rt-base
(cd "$selection" && sed 's|  /build/|  |' evidence/boot.sha256 | sha256sum -c -)
release="build/releases/rt-base-$(date -u +%Y%m%dT%H%M%SZ)"
mkdir -p "$release"
bzip2 -dc "$selection/tmp/deploy/images/raspberrypi4-64/qtooth-image-base-raspberrypi4-64.rootfs.wic.bz2" > "$release/image.img"
cp "$selection/evidence/"{kernel.config,kernel.release,image.manifest,kas-resolved.yml,build-tool.env,project-revision.txt,project-status.txt} "$release/"
(cd "$release" && sha256sum image.img kernel.config kernel.release image.manifest \
    kas-resolved.yml build-tool.env project-revision.txt project-status.txt > SHA256SUMS)
```

1. Retain the release directory and any source patch privately. Transfer the image
   and trusted manifest to the writing host; verify SHA-256 again after transfer.
2. In Raspberry Pi Imager select **Use custom → `image.img` → the identified spare
   microSD**. Apply no OS customization. Complete write/read-back verification and
   safely eject; insert into the unpowered Pi 4, then connect its supply.
3. Verify the board, running kernel/config SHA, boot-file hashes, packages and
   services using [the final physical lab](docs/courses/G1_Architecture_and_Practical_Guide.md#final-physical-lab).
   `zcat /proc/config.gz | sha256sum` must match the retained `kernel.config`,
   and RT must report `CONFIG_PREEMPT_RT=y`. **Base has no SSH**: the lab supplies
   offline collection; choose diagnostic for a live shell, with your own ignored
   local public-key input. Never flash QEMU derivatives.

## G1 — physical platform results

Brought up a **Raspberry Pi 4 Model B, 4 GB** with pinned ARM64 Linux 6.18.33,
RT/non-RT profiles, minimal and diagnostic images, verified SD deployment,
key-only diagnostic SSH, Bluetooth firmware/LE readiness and I2C/SPI/GPIO/USB
controller checks. Fixed CMAC packaging and enabled I2C1/SPI0 with a project DT
overlay while preserving Bluetooth's UART and reserving SPI CE0 for the FPGA.

**Base userspace:** BusyBox 1.37.0, systemd/networkd 259.5, D-Bus 1.16.2,
BlueZ 5.86, glibc 2.43, OpenSSL libcrypto 3.5.9. **Pi support:** HCI UART, CMAC,
I2C bcm2835/i2c-dev and SPI bcm2835 modules; BCM4345C0/C5 firmware and
`qtooth-platform-buses`. Diagnostic adds OpenSSH/access policy, rt-tests,
stress-ng, procps-ps and util-linux-chrt plus dependencies. These are image
packages; the full installed closure is recorded in each build's `image.manifest`.

| Image | WIC.bz2 MiB | Used RAM MiB | Boot s | Idle max µs | Loaded max µs |
|---|---:|---:|---:|---:|---:|
| RT base | 54.87 | 112.42 | 6.168 | — | — |
| Non-RT base | 54.78 | 108.26 | 5.407 | — | — |
| RT diagnostic | 59.90 | 112.53 | 6.372 | 56 | 337 |
| Non-RT diagnostic | 59.82 | 107.84 | 5.509 | 68 | 323 |

RAM is the median of three settled samples on instrumented copies; boot is one
fresh-image kernel-to-systemd observation per variant, excluding firmware.
Diagnostic maxima cover three 60-second trials per condition/profile; base runs
no workload tests. Raw WIC sizes are 338.80 MiB (base), 359.60 MiB (diagnostic).

**Result:** four physical variants verified; twelve successful scheduling trials,
720000 samples and no histogram overflow. RT showed mixed latency results, with
no consistent improvement across all statistics or proven deadline bound.
G1 is accepted under the operator-authorized assumption that supply current was
measured within the seller-certified board range; no numeric reading/trace is
supplied. BLE/PQC application and FPGA/electrical/recovery validation remain G2..5.

[Complete course and final lab](docs/courses/G1_Architecture_and_Practical_Guide.md) ·
[G1 acceptance record](openspec/changes/archive/2026-11-23-close-pi4-physical-g1/verification.md) ·
[Design](openspec/DESIGN.md) · [Hardware](openspec/HARDWARE.md)
