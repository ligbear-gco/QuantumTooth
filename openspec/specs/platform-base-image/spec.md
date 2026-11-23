# platform-base-image Specification

## Purpose

Provide a repeatable non-RT ARM64 platform build and QEMU boot experiment before the RT and physical-board stages of DESIGN G1, preserving BASE-1..3 and HARDWARE HW-1..4.

## Requirements

### Requirement: Fixed build inputs

The project SHALL provide a documented Docker/kas build entry point for `qtooth-image-base` using Wrynose, exact upstream repository commits, a container digest, and a matching kas wrapper release. It MUST record the resolved kernel and toolchain versions. Repeating the build from the same project revision MUST NOT silently advance upstream sources. This contract fixes inputs; it does not claim bit-identical output without a separate comparison.

#### Scenario: Build from a fresh checkout

- **WHEN** a developer follows the documented procedure with a reachable Linux Docker daemon and access to the pinned sources
- **THEN** the build resolves the recorded inputs and produces a QEMU ARM64 kernel, root filesystem, boot configuration, and installed-package manifest
- **AND** those resolved inputs can be identified from the build evidence

#### Scenario: A pinned input is unavailable

- **WHEN** a required digest or source revision cannot be fetched
- **THEN** the command fails visibly without substituting a moving tag, branch head, or different release

### Requirement: Explicit non-RT reference image

The QEMU build SHALL use a Linux kernel satisfying the DESIGN BASE version floor with PREEMPT_RT disabled. The final kernel configuration MUST be inspectable and attributable to the kernel booted by the documented command. Other preemption settings MUST be recorded without being presented as PREEMPT_RT.

#### Scenario: Confirm the selected kernel

- **WHEN** the developer builds and boots the non-RT profile
- **THEN** the final kernel configuration reports `# CONFIG_PREEMPT_RT is not set`
- **AND** the recorded boot artifacts and running kernel release correspond to that build

### Requirement: Shared lightweight runtime

The image SHALL contain the runtime defined in DESIGN BASE, with BusyBox providing the shell/utilities, systemd as PID 1, working system D-Bus, BlueZ/bluetoothd installed, glibc, and system libcrypto from OpenSSL >=3.5. It MUST exclude the feature applications, desktop, target compiler, training stack, SSH server, and diagnostic/stress suites reserved by BASE-2.

#### Scenario: Inspect image contents and services

- **WHEN** the developer examines the installed-package manifest and running QEMU guest
- **THEN** the required runtime packages and versions are identifiable, the BusyBox shell works, PID 1 is systemd, and the system D-Bus is active
- **AND** the excluded packages are absent
- **AND** an unavailable Bluetooth adapter is reported as an emulation limitation rather than successful radio validation

### Requirement: Accessible pre-hardware boot

The project SHALL document a headless `qemuarm64` boot procedure using software emulation and user-mode networking without requiring KVM, a TAP interface, or a privileged container. It SHALL provide root autologin on the local emulated serial console only for this QEMU development configuration, with no SSH server or forwarded guest-service ports. The common distro/image policy MUST NOT enable this login exception for future Pi builds.

#### Scenario: Open and close a guest session

- **WHEN** the developer launches the documented QEMU command inside the build environment
- **THEN** the terminal shows kernel boot output and reaches an interactive root shell without physical hardware
- **AND** the documented shutdown procedure stops the guest without deleting build output or caches

### Requirement: Persistent generated-data boundaries

Fetched upstream repositories, build outputs, and reusable download/task caches SHALL reside under the existing ignored `upstream/`, `build/`, and `cache/` roots respectively, persist across container runs, and be writable by the invoking development user. The documented workflow MUST NOT require hand edits to generated build configuration or manual changes to upstream repositories.

#### Scenario: Repeat a build

- **WHEN** the developer reruns the entry point after a successful build
- **THEN** it uses the same pinned inputs and can reuse persisted downloads/task results
- **AND** generated build data does not appear as untracked project content

### Requirement: Reproducible learner-run verification

The project SHALL provide a build/boot/inspect experiment with expected observations, an evidence report, and a short explanation of host/container/guest boundaries. The report MUST identify build inputs, artifact hashes and package versions, non-RT configuration, guest checks, unexpected service failures, and the final image/rootfs artifact sizes. It MUST label results as QEMU-only and leave DESIGN G1 and BASE-3 physical/RT measurements pending.

#### Scenario: Run the experiment

- **WHEN** a learner follows the documented build and QEMU boot procedure and runs `uname -m`, `cat /proc/1/comm`, `busybox`, and `systemctl is-active dbus` in the guest
- **THEN** the expected observations are `aarch64`, `systemd`, a BusyBox banner, and `active`
- **AND** the learner can inspect the corresponding final kernel configuration and installed-package manifest to confirm the remaining requirements
- **AND** the report distinguishes observed results, unperformed checks, and hardware-dependent checks
