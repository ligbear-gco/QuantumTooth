## Context

See [proposal.md](proposal.md) for motivation and [platform-base-image](specs/platform-base-image/spec.md) for acceptance. No build metadata exists yet. The user selected Docker on Ubuntu 26.04/Windows 10 WSL2 and QEMU before Pi availability. This design is needed for the new build dependencies and the host/container/guest boundary.

DESIGN BASE-1..3 and G1 govern this increment. HARDWARE HW-1..4 remain physical constraints for later bring-up; QEMU is an additional development target, not a replacement for `raspberrypi4-64`.

## Goals / Non-Goals

**Goals:** Keep shared image policy independent of the emulated machine; make initial source selection, build outputs, and boot evidence traceable; give the learner a short build/inspect loop.

**Non-Goals:** No RT profile or latency claim, Pi BSP implementation, diagnostic image, application/driver code, kernel Rust enablement, or hardware validation. No byte-for-byte reproducibility claim at this stage.

## Decisions

### 1. Pin the container and the build metadata separately

Use the official kas build container with Docker. Commit its digest and a matching `kas-container` release/checksum in versioned build-tool metadata; thin `tools/build` and `tools/run-qemu` entry points use that selection. Check Docker client/server access and Linux engine type before a long build. Preserve the invoking user's writable generated-data mounts; detect the existing daemon mode during preflight rather than reinstalling Docker.

`kas/base.yml` selects OE-Core Wrynose and the compatible BitBake release; `kas/base.lock.yml` pins exact commits. Use the repositories separately, without the historical combined Poky checkout or a second setup orchestrator. The pinned recipes supply checksums and kernel/toolchain source revisions; reject moving source revisions in the selected build closure. Resolve actual digests/commits during implementation and record them before accepting a build; planning does not invent tested pins.

Alternative: native Ubuntu tools are simpler to inspect but drift with host packages. A fixed container reduces that drift, while source locks address a different source of variation. [kas containers](https://kas.readthedocs.io/en/latest/userguide/kas-container.html), [kas lockfiles](https://kas.readthedocs.io/en/latest/userguide/project-configuration.html#working-with-lockfiles), [Wrynose repository model](https://docs.yoctoproject.org/6.0/ref-manual/faq.html#how-does-poky-differ-from-openembedded)

### 2. Separate distribution policy, package selection, and machine configuration

`meta-qtooth-platform/conf/distro/qtooth.conf` owns shared glibc/systemd and headless policy. Its layer configuration declares Wrynose compatibility. A small platform package group feeds `recipes-core/images/qtooth-image-base.bb`; resolve actual binary package names through the pinned recipes/pkgdata. Include the runtime required by the spec, including system libcrypto, without adding OpenSSL command-line tools merely for verification.

`kas/machines/qemuarm64.yml` includes the shared configuration and `kas/profiles/nonrt.yml`, selects upstream `qemuarm64`, and builds `qtooth-image-base`. Start with OE-Core's `linux-yocto` 6.18 recipe; it already supports this virtual board. A platform-owned append/config fragment disables PREEMPT_RT and enables embedded kernel configuration access (`IKCONFIG`/`IKCONFIG_PROC`) so the booted guest can expose its configuration. Preserve and record the remaining upstream preemption choices.

Explicitly opt out of unnecessary graphics/desktop features using Wrynose's current feature-default mechanism; inspect resolved features and the final manifest. Do not add `meta-openembedded`, `meta-yocto`, or the Pi BSP merely to establish this QEMU build. Future Pi work adds a matching BSP and selects its own kernel while reusing the distro/image policy; it must still meet G1.

Alternative: copy a stock image or create a custom QEMU machine. Reusing the upstream machine avoids maintaining virtual hardware definitions; owning a small distro/image makes project policy explicit. [Kernel recipe](https://github.com/openembedded/openembedded-core/blob/wrynose/meta/recipes-kernel/linux/linux-yocto_6.18.bb), [Wrynose feature changes](https://docs.yoctoproject.org/6.0/migration-guides/migration-6.0.html)

### 3. Boot through an isolated development console

Run Yocto's built QEMU through `runqemu` inside the same container environment, selecting the generated boot configuration from this build explicitly rather than an arbitrary latest artifact. Produce an ext4 rootfs and use a disposable snapshot for inspection. Use `nographic` and `slirp`; override the default SLIRP options to remove guest-service port forwarding. Software emulation avoids a KVM/nested-virtualization dependency, including on an x86 workstation.

Scope `serial-autologin-root` plus `empty-root-password` to the QEMU machine configuration. These are a local development convenience, not common distro defaults. Install no SSH server, expose no TCP serial console, and give the learner the normal guest shutdown and emergency emulator exit commands. Bluetooth service installation is checked separately from adapter availability; an absent controller does not block the shell or system D-Bus checks.

Alternative: SSH/TAP adds networking and credential setup before the first boot; ARM KVM would impose host architecture and virtualization requirements. [QEMU workflow](https://docs.yoctoproject.org/6.0/dev-manual/qemu.html), [serial image features](https://docs.yoctoproject.org/6.0/ref-manual/features.html#image-features)

### 4. Keep generated data and verification attributable

Use `upstream/` for kas checkouts/bookkeeping, `build/qemuarm64-nonrt/` for this build, and `cache/downloads/` and `cache/sstate/` for reusable downloads/task results. Configure these through the wrapper's supported environment/mount handling; do not hand-edit generated `local.conf` or `bblayers.conf`. Retain `.gitkeep` only where a directory remains empty.

The data path is: project configuration → kas → BitBake dependency tasks → cross-built kernel/packages → assembled rootfs → QEMU → kernel mounts rootfs → systemd starts services/getty → BusyBox shell. Containers share the host kernel while building; the QEMU guest boots the target kernel. No Rust application, kernel Rust API, unsafe block, or FFI boundary is introduced by this increment. Authored helper code must use standard Doxygen tags.

Put raw logs, hashes, manifest, resolved configuration, and final `.config` under the ignored build root. Keep the concise learner procedure and verification report under `docs/bringup/`; report commands, observations and unperformed checks rather than assuming success. Verify BusyBox, PID 1, D-Bus, installed packages, `uname -r`, and guest `/proc/config.gz` against the selected boot artifacts; record final artifact sizes and unexpected service failures. Optional explain-back: why does an ARM64 RT/non-RT QEMU boot not establish Pi scheduling latency?

## Risks / Trade-offs

- WSL2 host behavior and filesystem/mount differences → use Linux filesystem storage, preflight Docker access and mount ownership, and report the actual environment. Containers do not remove host-kernel constraints; [Yocto does not validate WSL2 hosts](https://docs.yoctoproject.org/6.0/dev-manual/start.html#setting-up-to-use-windows-subsystem-for-linux-wsl-2).
- QEMU service state differs without a Bluetooth controller → require installed BlueZ and active D-Bus; document a condition-skipped Bluetooth service, but investigate an actual unexpected failure.
- Source/container pins can be unavailable or incompatible → fail visibly, choose verified compatible inputs during implementation, and commit any pin update deliberately before rerunning acceptance.
- Builds are large and ARM emulation is slower on x86 → persist caches, record actual disk/RAM availability, and avoid wall-clock acceptance deadlines.
- QEMU-only console access could leak into later targets → constrain it to the QEMU override and inspect the common policy for absence of the exception.

## Migration Plan

Add metadata and helpers in the agreed skeleton, build the non-RT image, boot it, and write the local verification report. There is no existing deployed image to migrate. Rollback is reverting this increment's metadata/helpers and stopping its containers/guest; generated caches can remain. No SD flashing or physical connection occurs, and G1 stays pending.
