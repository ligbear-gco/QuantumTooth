## 1. Learn: prerequisites and build boundaries

- [x] 1.1 Re-read both SoTs and explain host/container/guest roles; run `docker version`, inspect Docker engine mode, Linux filesystem placement, disk/RAM availability, and writable mount ownership. Record actual results and resolve prerequisites before building; a Docker CLI version alone is insufficient evidence.
- [x] 1.2 Select an official kas release/container digest and compatible OE-Core Wrynose/BitBake commits; verify the matching wrapper checksum, repository compatibility, and fixed kernel/toolchain source revisions. Record candidate pins and explain why container and source locks solve different problems.

## 2. Build: versioned configuration and entry points

- [x] 2.1 Add the pinned kas wrapper/tool metadata and Doxygen-documented `tools/build` and `tools/run-qemu` helpers. Verify the selected version/digest, clear prerequisite errors, and user-writable persistent mounts without requiring a privileged container.
- [x] 2.2 Add `kas/base.yml`, its lockfile, `kas/machines/qemuarm64.yml`, and `kas/profiles/nonrt.yml`; configure local platform-layer inclusion and the agreed ignored data roots. Verify kas checkout/dump resolves fixed commits, `qemuarm64`, `qtooth`, and `qtooth-image-base`, with no generated-config hand edits.
- [x] 2.3 Add the platform layer, distro policy, package group, and base image recipe. Verify Wrynose layer compatibility, glibc/systemd selection, headless feature resolution, required package availability including OpenSSL >=3.5 libcrypto, and exclusion of feature/diagnostic/SDK/desktop packages through BitBake metadata inspection.
- [x] 2.4 Add the platform-owned `linux-yocto` configuration for the 6.18 non-RT candidate and kernel configuration access. Inspect the final generated `.config` for disabled PREEMPT_RT and enabled `IKCONFIG`/`IKCONFIG_PROC`; record other preemption settings and any configuration audit failures.
- [x] 2.5 Configure QEMU-only serial autologin, ext4/snapshot boot, and user-mode networking without guest-service forwarding. Inspect the resolved image features, boot configuration, and launcher arguments to confirm these settings and absence of KVM/TAP requirements or common-policy login exceptions.

## 3. Inspect: build and guest integration

- [x] 3.1 Run the documented kas configuration/BitBake parse checks before a full build; require the expected provider, machine, distro, and target with no unresolved dependencies or configuration errors. Explain how the resulting dependency tasks assemble the image.
- [x] 3.2 Build with `./tools/build`; retain logs, resolved inputs/toolchain versions, final kernel configuration, package manifest, boot configuration, artifact hashes, and final image/rootfs artifact sizes under `build/qemuarm64-nonrt/`. Require a successful build and identify the exact artifacts selected for boot.
- [x] 3.3 Launch with `./tools/run-qemu`; verify the serial root shell, `uname -m` = `aarch64`, `cat /proc/1/comm` = `systemd`, a BusyBox banner, and `systemctl is-active dbus` = `active`. Compare `uname -r` and `zcat /proc/config.gz` with the selected build artifacts and confirm PREEMPT_RT is disabled; demonstrate clean guest shutdown and snapshot behavior.
- [x] 3.4 Inspect the final package manifest/pkgdata for the required runtime, glibc, installed bluetoothd/service, and system libcrypto version; verify excluded packages are absent. Inspect `systemctl --failed` and Bluetooth unit state, explain absent-controller conditions, and resolve or explicitly report unexpected failures without claiming radio validation.
- [x] 3.5 Rerun the build to verify stable input resolution and cache reuse, check generated paths with `git status`/`git check-ignore`, and exercise an unavailable pin using an isolated temporary configuration. Require a visible failure without fallback and no changes to committed pin selection or unrelated files.

## 4. Explain: reproducible experiment and evidence

- [x] 4.1 Write a concise `docs/bringup/` runbook covering prerequisites, pinned setup, build/boot/inspection commands and expected results, snapshot shutdown/exit behavior, and artifact locations. Verify the learner can follow it using the delivered entry points; offer the optional explain-back question from the design without making it a gate.
- [x] 4.2 Write the verification report with measured observations and artifact identifiers from section 3, resource/environment facts, failures or unperformed checks, and demonstrated learning outcomes. Confirm it labels the image non-RT/QEMU-only and leaves G1, Pi microSD deployment/peripheral checks, and BASE-3 physical RT/non-RT measurements pending; verify both SoTs and diagrams are unchanged and run `git diff --check`.
