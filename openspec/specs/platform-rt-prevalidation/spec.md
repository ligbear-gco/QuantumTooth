# platform-rt-prevalidation Specification

## Purpose

Prevalidate the Pi BSP, PREEMPT_RT integration and platform measurement procedure under QEMU while physical hardware is unavailable, preparing evidence for DESIGN G1 without closing its hardware acceptance gate.

## Requirements

### Requirement: Locked Pi build inputs and preserved reference

The project SHALL provide documented `raspberrypi4-64` non-RT and RT build selections using a Wrynose-compatible Pi BSP and Linux satisfying DESIGN BASE. Repository commits, build container, kernel/toolchain sources, emulator and any external RT patches MUST be fixed and identifiable. Unavailable pins MUST fail visibly without substitution. The existing no-argument non-RT `qemuarm64` build/boot contract MUST remain usable. Generated sources, outputs and caches SHALL remain under the existing ignored roots, with separate outputs/evidence for each machine/profile/image selection.

#### Scenario: Resolve the Pi profiles

- **WHEN** a developer resolves and builds either Pi profile using the documented entry point
- **THEN** the machine, kernel provider/revision, RT integration provenance, toolchain, BSP and emulator inputs are recorded
- **AND** the result includes a kernel, matching modules, Pi device tree, rootfs, microSD deployment image and package manifest without replacing the original QEMU reference evidence

#### Scenario: An input or selection is invalid

- **WHEN** a required pin cannot be fetched or a requested machine/profile/image combination is unsupported
- **THEN** the command exits nonzero with an actionable error and does not select another source or configuration

### Requirement: Verifiable non-RT and RT kernel profiles

Both Pi profiles SHALL preserve the lightweight runtime and required BSP driver/firmware configuration from DESIGN BASE-1..2 and HARDWARE HW-1..3. Their final configurations MUST be inspectable: non-RT disables PREEMPT_RT and RT enables `CONFIG_PREEMPT_RT=y`. Evidence MUST distinguish integrated RT support from additional patches, record other preemption/debug settings and relevant configuration audit results, and attribute each boot to the selected kernel, modules and configuration. Requested fragments or a kernel name alone MUST NOT count as RT verification.

#### Scenario: Inspect both running kernels

- **WHEN** the developer inspects the saved final configurations, running releases and `/proc/config.gz` in both Pi guests
- **THEN** guest configuration hashes and releases match their respective build evidence, PREEMPT_RT is disabled only in the non-RT profile, and matching module artifacts are identified
- **AND** required runtime packages, Pi driver settings and firmware availability are verified from build evidence without claiming physical peripheral operation

### Requirement: Pi artifact boot on the Pi emulator model

Prevalidation SHALL boot the Pi-built kernel, matching Pi device tree and root filesystem on the pinned QEMU `raspi4b` model through an interactive headless serial session. The launcher MUST record exact artifacts/hashes, emulator version, arguments, model resources, and all derived device-tree/rootfs changes. Hardware deployment artifacts MUST remain intact. Direct kernel loading and emulated storage MUST be identified as bypassing physical boot stages. A boot using a generic ARM64 machine/kernel MUST NOT satisfy Pi prevalidation.

#### Scenario: Reach the Pi guest userspace

- **WHEN** a developer launches either Pi profile with its recorded boot selection
- **THEN** the serial guest reports `aarch64`, a Pi-compatible device tree, systemd as PID 1, BusyBox and active system D-Bus, with kernel/configuration identity checks passing
- **AND** a bounded boot check, service-failure review and clean shutdown are recorded, explaining unsupported firmware poweroff behavior if the emulator needs a controlled exit after guest shutdown

#### Scenario: The BSP and emulator cannot boot together

- **WHEN** the selected Pi artifacts fail the documented compatibility or boot checks
- **THEN** the report identifies the failed gate and retained diagnostics, stops dependent acceptance work, and leaves Pi prevalidation incomplete rather than silently substituting `qemuarm64`

### Requirement: Isolated emulation access and adaptations

The Pi emulation procedure SHALL work without KVM, TAP, a privileged container or physical hardware. It MUST NOT require networking or expose guest-service ports. Any serial autologin/login exception and device-tree adaptation MUST be confined to an explicitly identified emulation derivative; neither common policy nor the original Pi deployment image SHALL inherit them. Guest writes MUST be disposable and source artifact hashes MUST remain unchanged after the session. Physical UART/pin assignments SHALL remain governed by HW-2..3.

#### Scenario: Repeat a disposable session

- **WHEN** the developer creates a marker in one emulated session, shuts down and opens a fresh session
- **THEN** the marker is absent, original deployment hashes still match, and the report identifies any emulation-only derivative and console exception
- **AND** the Pi deployment configuration contains no inherited empty-password/autologin exception

### Requirement: Separate diagnostic image

The project SHALL provide a diagnostic image variant for both Pi profiles containing `rt-tests`/`cyclictest`, a bounded stress tool and the inspection utilities required by the experiment, in accordance with BASE-2. The base image MUST retain the shared runtime without diagnostic/stress suites, feature applications, desktop, target compiler or training stack. Diagnostic differences MUST be attributable through manifests; no SSH server is required by this change.

#### Scenario: Compare base and diagnostic contents

- **WHEN** the developer examines both image manifests and launches the diagnostic image for each profile
- **THEN** required diagnostics execute in both guests, the documented package difference is present only in the diagnostic variant, and required shared runtime remains available

### Requirement: Controlled emulation comparison

The experiment SHALL record RT/non-RT kernel and rootfs/deployment artifact sizes, idle RAM, a defined guest boot interval and scheduling wake-up latency under idle and bounded load conditions. Comparisons MUST use matching image variants, emulator resources/settings and test parameters; kernel source/configuration differences and host conditions MUST be disclosed. At least three runs per profile/condition SHALL retain raw outputs, durations, sample counts, workload parameters and latency summaries. The report MUST label all timings/RAM as emulation observations and MUST NOT require RT to outperform non-RT, infer physical Pi performance or claim a worst-case deadline bound.

#### Scenario: Run the matched diagnostic comparison

- **WHEN** the developer follows the documented idle/load protocol in both diagnostic guests
- **THEN** repeated results and collection commands are retained with artifact identity, guest/host clock basis, host resources/load, and identical configured guest resources and workload parameters
- **AND** sizes are labeled by artifact format, idle-memory and boot-interval definitions are stated, failures/incomplete trials are disclosed, and no physical acceptance threshold is inferred

### Requirement: Reproducible learning evidence and hardware handoff

The project SHALL document a learner-run build/boot/inspect/compare experiment with prerequisites, commands, expected observations and an evidence report distinguishing measured results, assumptions, failures and unperformed checks. The report MUST leave G1 open and enumerate the remaining physical microSD boot, peripheral/Bluetooth checks, applicable HW-4 facts and BASE-3 RT/non-RT measurements. It MUST describe the limits of the selected emulator, required physical retests and any difference between deployment artifacts and what QEMU exercised. Both SoTs and protected diagrams SHALL remain unchanged.

#### Scenario: Inspect the experiment and remaining G1 evidence

- **WHEN** a learner follows the runbook and runs `uname -m`, `cat /proc/1/comm`, `busybox`, `systemctl is-active dbus`, and `zcat /proc/config.gz` in each guest
- **THEN** expected results are `aarch64`, `systemd`, a BusyBox banner, active D-Bus and the correct profile-specific PREEMPT_RT setting, with commands to compare complete configuration hashes
- **AND** the diagnostic experiment explains the measurement boundaries and offers an optional explain-back question
- **AND** the final report can accept this change's QEMU prevalidation while explicitly recording physical G1 acceptance as pending hardware delivery
