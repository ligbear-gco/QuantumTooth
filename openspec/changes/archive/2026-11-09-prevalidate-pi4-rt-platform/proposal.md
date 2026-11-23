## Why

The archived bootstrap proves only a non-RT `qemuarm64` platform. While the Pi is awaiting delivery, prevalidate the Pi BSP, PREEMPT_RT integration and measurement procedure under QEMU so physical G1 bring-up starts from attributable build artifacts and tested tools.

## What Changes

- Increment 1: pin a Wrynose-compatible `meta-raspberrypi`, build a `raspberrypi4-64` non-RT reference and boot its kernel, Pi device tree and root filesystem on a pinned QEMU `raspi4b` model. Establish BSP/emulator feasibility before dependent work; a generic ARM64 boot cannot substitute for this acceptance.
- Increment 2: build and boot the corresponding Pi RT profile, documenting integrated RT support and any additional version-matched patch set; verify the final configuration and its running-guest identity.
- Increment 3: add a separate diagnostic image with `rt-tests` and bounded stress workloads; compare RT/non-RT image size, idle RAM, boot time and scheduling latency as emulation observations.
- Extend build/launch/evidence tooling with explicit machine, kernel-profile and image selection while preserving the existing no-argument QEMU/non-RT workflow. Isolate emulation-only console/device-tree adaptations from hardware deployment artifacts.
- Deliver a learner-run experiment, verification report and hardware handoff checklist. Keep physical boot, peripherals/radio, HW-4 facts and Pi measurements pending; G1 remains open.

## Capabilities

### New Capabilities

- `platform-rt-prevalidation`: Build and identify Pi non-RT/RT artifacts, exercise them on `raspi4b`, and reproduce a diagnostic comparison with explicit emulator limits and deferred G1 acceptance.

### Modified Capabilities

None. The existing `platform-base-image` contract remains the original non-RT `qemuarm64` reference.

## Impact

Affects `kas/`, `layers/meta-qtooth-platform/`, `tools/` and `docs/bringup/`. Adds a locked Pi BSP and, only if required, a pinned RT patch dependency. Generated sources, outputs and caches remain under `upstream/`, `build/` and `cache/`. No BLE/Android applications, FPGA logic, feature drivers or public application APIs are introduced.

Conforms to DESIGN SCOPE, BASE-1..3 and G1 and HARDWARE HW-1..4. QEMU is a development target; its model and virtual console do not change the selected physical board or Bluetooth UART allocation. Neither SoT nor protected diagrams are revised. This change can finish QEMU prevalidation without claiming G1 completion; physical acceptance is follow-up work after delivery.

### Learning

Prerequisites: the bootstrap runbook's shell/Git, Docker and Linux-filesystem setup; access to pinned sources and sufficient build resources. Hardware is not required for this change.

Outcomes:

1. Trace machine/BSP metadata through kernel, device tree and rootfs into an identified running guest.
2. Distinguish integrated PREEMPT_RT, optional RT patches, final kernel configuration and observable interrupt/scheduling behavior.
3. Run a controlled diagnostic comparison and explain which observations must be repeated on the Pi.

The experiment makes this path observable: kas selects the BSP/profile, BitBake builds the artifacts, QEMU supplies modeled devices, and guest checks/diagnostics produce evidence tied to those artifacts. Emulation timings are influenced by the host and are not physical Pi acceptance measurements.
