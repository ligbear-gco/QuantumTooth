## Context

See [proposal.md](proposal.md) for motivation and [the delta spec](specs/platform-rt-prevalidation/spec.md) for acceptance. DESIGN BASE-1..3/G1 and HARDWARE HW-1..4 govern this work.

The existing kas wrapper, build helper, launcher and collector assume `qemuarm64`, non-RT and one evidence directory. The pinned OE-Core includes QEMU 10.2.0, `rt-tests` and `stress-ng`. Its `linux-yocto-rt` recipe limits compatible machines to QEMU targets; changing its provider selection alone does not establish Pi BSP support.

QEMU 10.2.0's source documentation identifies `raspi4b` as four Cortex-A72 cores/2 GiB, with PCIe root port and GENET Ethernet missing. This differs from HW-1's physical 4 GB board. Current development documentation describes more devices and is not evidence for the pinned binary. The Wrynose Pi machine metadata supplies a BCM2711 Pi 4 device tree and Broadcom firmware recommendations; the exact selected commit still needs implementation-time verification.

## Goals / Non-Goals

**Goals:** preserve an identifiable hardware-deployment candidate and its explicitly derived QEMU experiment; exercise matched Pi non-RT/RT base and diagnostic images; retain a reproducible path to physical G1 acceptance.

**Non-Goals:** execute physical deployment, close G1, infer radio/electrical behavior or physical timing, implement feature applications/drivers, or qualify future FPGA kernel APIs under G3. This platform change adds no Rust application or FFI boundary; kernel Rust/toolchain qualification remains with its owning feature gate.

## Decisions

### 1. Keep the Pi BSP and use a bounded compatibility gate

Use `MACHINE=raspberrypi4-64`, a commit-locked Wrynose `meta-raspberrypi` and its `linux-raspberrypi` provider for both profiles. Select a kernel meeting DESIGN BASE and pin its recipe-resolved sources. Keep the existing OE-Core/BitBake/container pins unless an evidenced compatibility issue requires a deliberate, reported lock update.

Start with the existing QEMU 10.2.0 build. Inspect `--version`, `-machine help`, its source docs and the resolved Pi metadata; verify DTB/SD/serial/timer support and available RT configuration before bulk builds. Build and boot the non-RT Pi base image first. If this combination cannot pass with bounded DT/launch adaptations, retain the failed logs and stop dependent work. Do not silently replace the kernel/model or advance QEMU. An explicit pinned compatibility fix must be assessed against the same acceptance contract before proceeding.

Alternatives: `qemuarm64`/`virt` does not exercise Pi device addresses/drivers; merely widening `linux-yocto-rt`'s machine filter does not supply the BSP; a moving emulator release loses attribution.

### 2. Make PREEMPT_RT a verified profile of the same Pi kernel family

Prefer one Pi kernel source commit for both profiles, differing in intentional RT configuration and its required dependencies. Inspect that source's PREEMPT_RT support first. Use integrated support when sufficient; if additional RT maintenance patches are needed, pin their exact source, checksum, order and matching base revision and record the source difference. Never stack an already integrated patch set blindly or call a fragment a verified RT kernel.

The collector checks the final `.config`, audit logs and release for each profile, requires `IKCONFIG`/`IKCONFIG_PROC`, and compares the full saved configuration hash with `/proc/config.gz` after boot. Inspect module release/vermagic where modules are installed and record the preemption/debug delta. PREEMPT_RT moves much interrupt work into scheduler-controlled threads and makes many locks preemptible; this is the mechanism to inspect, not a claim that all IRQs are threaded or that every scheduling delay disappears.

Keep BSP boot/storage, Bluetooth UART/firmware, I2C/SPI/GPIO and USB support in the deployment configuration. Missing modeled devices can explain guest limitations but cannot justify removing required hardware support.

### 3. Select configurations explicitly without breaking the bootstrap

Extend `tools/build` and `tools/run-qemu` with `--machine`, `--profile` and `--image`; retain `--check` for build parsing. Defaults stay `qemuarm64`, `nonrt`, `base`. Support the existing default and the four Pi combinations `nonrt|rt` × `base|diagnostic`; reject other combinations until intentionally supported. Let `tools/kas` select the corresponding build directory rather than overwrite the bootstrap's directory.

Keep shared metadata in `kas/base.yml`, add a Pi machine selection, an RT profile and a diagnostic selection, with machine-scoped provider settings. Store Pi outputs/evidence under `build/raspberrypi4-64-<profile>-<image>/`; retain `build/qemuarm64-nonrt/`. Share downloads/sstate while preserving distinct resolved configuration, kernel, manifests and boot-selection records. Helpers remain small; any new code receives standard Doxygen documentation.

### 4. Direct-boot Pi artifacts with isolated QEMU derivatives

Produce original Pi `Image`, `bcm2711-rpi-4-b.dtb`, ext4 rootfs, matching modules and microSD `.wic` artifacts for each selection. Use `qemu-system-aarch64 -M raspi4b` with software emulation, fixed model resources, explicit kernel/DTB and disposable SD storage derived from that rootfs. Direct loading bypasses Pi boot ROM/EEPROM, firmware boot selection and physical microSD behavior, even though their deployment files are built/inspected.

Prepare the emulation SD image with unprivileged native image tools under the selection's build directory. A separately hashed copy can include a local serial-getty/autologin exception and required SD sizing; the original rootfs/`.wic` stays unchanged. Apply any needed DT adaptations to a separate DTB, with a transformation log and source/derived hashes. Keep the deployment kernel unchanged. Record QEMU's own DT fixups and check guest model/compatible properties; do not expect an identical binary DT after emulator fixups.

Disable guest networking for this Pi experiment; use serial transcripts to collect diagnostics. No KVM, TAP, forwarded ports or privileged container is required. Snapshot each prepared disk so guest writes disappear between sessions. Verify source hashes before/after, exercise the marker-discard check, and shut down the guest before terminating QEMU if its modeled poweroff does not exit automatically. Set documented host-side boot/run timeouts after the first feasibility run; preserve timeout logs.

The physical Bluetooth UART remains reserved. Emulated serial changes must not leak into the deployment DT or console configuration. Keeping an insecure login in all Pi images would violate the existing platform contract; the explicit derivative avoids that leak.

### 5. Add diagnostics as an image variant and compare like with like

Add `qtooth-image-diagnostic` extending the base runtime with `rt-tests`, `stress-ng` and only the tools needed for collection. Leave SSH out for this serial experiment. Inspect manifest differences and base exclusions. Build/boot both base images before the diagnostic pair; compare base sizes/idle RAM/boot intervals separately from diagnostic sizes/measurements.

Use one recorded protocol for both diagnostic profiles: same QEMU binary, four modeled CPUs, 2 GiB model memory, accelerator/threading/clock settings, scheduler policy/priority, timer interval, load and run duration. Initial protocol: three fresh-boot trials per profile at idle and under load; 60-second `cyclictest` runs using FIFO priority 80 and a 1 ms interval after pinned-tool option checks. Load uses two CPU workers and one bounded 128 MiB memory worker, started before measurement and stopped afterward. Record histogram overflow/sample counts, raw outputs and min/mean/max; do not hide failed trials. Reduce experiment duration only with a recorded resource rationale before collecting the compared set, applying the same protocol to both profiles.

Record `/proc/meminfo` after a fixed documented settle interval with field/formula definitions. Measure the guest kernel-start-to-systemd-ready interval separately from host launcher wall time; `systemd-analyze` availability and journal timestamps determine the exact documented collection command. Label compressed/uncompressed rootfs, `.wic` and kernel sizes separately. Record host OS/WSL/container/CPU resources and load, guest clock source and test parameters. An emulated RT result can be worse or noisier; success is an attributable experiment, not an improvement threshold.

### 6. Complete prevalidation and hand off physical acceptance separately

Write `docs/bringup/pi4-rt-qemu.md` and `pi4-rt-prevalidation.md`, linked from the README, with expected versus measured observations, limitations and learning outcomes. Include an optional explain-back: why does `CONFIG_PREEMPT_RT=y` plus a low QEMU latency maximum not prove Pi deadline behavior?

Use the three increments as learn/build/inspect/explain cycles. The learner traces BSP → kernel/DT/rootfs → modeled devices → guest services and then runs the matched diagnostic procedure. Keep raw data under the matching ignored evidence directories, with commands, artifact hashes and concise committed results.

The handoff lists physical microSD boot/firmware path, actual Pi revision/resources/clocks/temperature, required interfaces and Bluetooth initialization, applicable HW-4 facts, and physical BASE-3 comparisons. Missing FPGA/bench HW-4 facts remain with their later gates. This change finishes only after its QEMU criteria pass; a hardware checklist is a later acceptance obligation, not fictitious completed tasks or a reason to call G1 passed.

## Risks / Trade-offs

- Pi kernel/device-tree mismatch with QEMU → fail the non-RT compatibility gate early; retain exact diagnostics and assess a pinned fix before dependent work.
- QEMU 2 GiB resources and missing devices differ from HW-1 → disclose the model matrix; inspect deployed support and repeat relevant tests physically.
- Derivative DT/rootfs hides hardware defects → retain original/derived hashes and transformations; prohibit hardware claims from adapted guest behavior.
- Host scheduling/TCG and guest clocks distort latency/boot measurements → fix settings, repeat matched trials and report clock bases without physical bounds.
- Separate base/diagnostic/RT outputs increase build cost → share caches and stage builds after feasibility, keeping attributable evidence per selection.

## Migration Plan

Add the profiles/tool selectors without changing legacy defaults, then pass increments 1–3 in order. Re-run the existing bootstrap build/boot checks after helper changes. Roll back versioned metadata/helpers if needed; ignored per-selection outputs can remain. No disk flashing or hardware connection occurs in this change.

## References

- [QEMU 10.2.0 Pi model documentation](https://raw.githubusercontent.com/qemu/qemu/v10.2.0/docs/system/arm/raspi.rst); the checked-out matching source is the implementation reference.
- [Wrynose Pi machine metadata](https://github.com/agherzan/meta-raspberrypi/blob/wrynose/conf/machine/raspberrypi4-64.conf); replace branch-only provenance with the selected commit in implementation evidence.
- [PREEMPT_RT mechanism](https://docs.kernel.org/core-api/real-time/theory.html); verify configuration against the selected source tree.
