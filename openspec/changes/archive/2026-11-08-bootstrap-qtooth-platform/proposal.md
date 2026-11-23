## Why

The repository has a directory skeleton but no buildable platform image. A pinned, non-RT ARM64 image that boots in QEMU establishes the build and userspace foundation before RT configuration and Pi hardware introduce additional variables.

## What Changes

- Add a Docker-based kas entry point with a pinned container digest, matching wrapper release, and locked Wrynose build inputs.
- Introduce `meta-qtooth-platform`, the `qtooth` distro, and `qtooth-image-base` with the runtime defined in DESIGN BASE.
- Add the `qemuarm64` machine selection and explicit non-RT profile; boot through a development-only serial console without requiring KVM or privileged networking.
- Document build, boot, inspection, and evidence collection on Ubuntu 26.04 under Windows 10 WSL2.
- Defer RT, diagnostic images, Pi builds/SD deployment, BLE applications, and FPGA work. Pi microSD boot remains the agreed later deployment path.

## Capabilities

### New Capabilities

- `platform-base-image`: Build a pinned non-RT base image, boot it in QEMU ARM64, and verify its runtime and configuration.

### Modified Capabilities

None.

## Impact

Affects `kas/`, `layers/meta-qtooth-platform/`, `tools/`, and build/bring-up documentation. Generated checkouts, outputs, and caches stay under their existing ignored roots. No application APIs change.

Conforms to DESIGN SCOPE and BASE-1..3 as a preparatory non-RT reference; G1 remains open until the BSP-compatible RT image boots and measurements are collected on the Pi. Conforms to HARDWARE HW-1..4 without changing physical assignments or claiming hardware evidence. Both SoTs and their diagrams remain unchanged.

### Learning

Prerequisites: basic shell/Git use, a Linux Docker daemon reachable from WSL2, Internet access, and project/build data on the WSL Linux filesystem. Docker 29.9.2 and sufficient resources are user-reported; verify accessibility and available resources during implementation.

Outcomes: explain host/container/guest boundaries; trace kas configuration through BitBake into kernel/rootfs artifacts; verify a booted non-RT system using configuration and runtime evidence. The observable mechanism is cross-compilation followed by QEMU loading the resulting kernel and root filesystem.
