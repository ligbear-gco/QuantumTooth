# QuantumTooth design

Architecture SoT, baseline 2026-11-08; phone UI stack and G2 acceptance boundary revised 2026-11-25 by explicit user decision. Frozen for future feature work; revisions require explicit user authorization. [HARDWARE.md](HARDWARE.md) owns physical facts and wiring. Specs derive behavior from these documents; unresolved gates are not permission to invent requirements.

## SCOPE — boundaries and language

One headless Yocto Linux RT image hosts two independent shell-driven demonstrations: **qtooth-ble** and **qtooth-hwe2dr** (HardWare Enhanced Error Detection and Recovery). Neither depends on or forwards data through the other. Shell entry points are userspace applications; kernel drivers handle configuration and event delivery. The FPGA owns qtooth-hwe2dr device reset; its IRQ only notifies Linux.

Rust owns CLIs, protocol/state machines, Android native core, training/export/reference tools and target firmware. Train through tch/LibTorch on the workstation. Use Rust in the kernel wherever the pinned tree exposes the required APIs; permit small C glue for missing SPI/GPIO/IRQ bindings. TypeScript with React Native owns the phone UI; minimal Kotlin adapts Android BLE and bridges to Rust through JNI. Verilog/SystemVerilog describes FPGA hardware. No soft CPU or software VAE runtime on the FPGA.

## BASE — lightweight Yocto RT platform

| Stack | Decision |
|---|---|
| Build | Yocto/OE-Core 6.0 Wrynose LTS, matching meta-raspberrypi; kas pins commits, recipes and Cargo.lock dependencies |
| Target | raspberrypi4-64, glibc; BSP-compatible Linux >=6.12 with CONFIG_PREEMPT_RT; exact kernel/toolchain revisions pinned after bring-up |
| Runtime | BusyBox shell/utilities, systemd, D-Bus, BlueZ/bluetoothd, OpenSSL >=3.5 libcrypto; no desktop, compiler or training stack |
| Drivers | Bluetooth LE/HCI UART + Broadcom support/firmware, I2C controller, SPI, GPIO IRQ, USB and microSD; device-tree overlays describe project devices |
| Rust build | OE cargo classes and locked crate sources; kernel Rust uses Kbuild and matching rustc/LLVM/bindgen, not userspace Cargo linkage |

**BASE-1:** meta-qtooth-platform owns image/kernel configuration and overlays. Independent meta-qtooth-ble and meta-qtooth-hwe2dr layers own feature recipes/packages. The combined image installs both; each feature can also be selected alone. A Yocto layer packages an application; it is not a kernel execution layer.

**BASE-2:** Trim unused services/drivers without removing BSP boot or Bluetooth initialization requirements. Keep debugging/SSH, rt-tests and stress tools in a diagnostic image variant. Build, train and synthesize on the workstation.

**BASE-3:** Record image size, idle RAM, boot time and scheduling latency under load; compare RT/non-RT configurations. Bluetooth, userspace crypto and radio delivery have no hard deadline guarantee. Keep kernel event handling bounded and separate from BLE work; FPGA reset timing is independent of Linux scheduling.

![Yocto platform and independent applications](../docs/diagrams/platform.svg)

## BLE — qtooth-ble

Entry point:

    qtooth-ble send "Hello from QuantumTooth"

| Component | Stack |
|---|---|
| Pi application | Rust, Cargo, clap, Tokio, BlueR/bluer GATT server over BlueZ D-Bus |
| Shared protocol/crypto | Rust library; OpenSSL EVP ML-KEM-768, HKDF-SHA-256, AES-256-GCM; openssl bindings plus a narrow reviewed FFI wrapper if KEM APIs are absent |
| Phone | TypeScript / React Native UI; minimal Kotlin Android BLE/native module; shared Rust core through JNI; NDK-built OpenSSL >=3.5, independent of Android's system crypto |
| Packaging | meta-qtooth-ble Cargo recipe, system libcrypto linkage on Pi; no vendored duplicate OpenSSL in the image |

**BLE-1:** The command advertises a custom service and waits with a timeout. The foreground phone app scans, connects and subscribes; no background-discovery guarantee. Pi is peripheral/GATT server; phone is central/client. Use writes with response toward Pi and indications toward phone initially.

**BLE-2:** Pi generates an ephemeral ML-KEM key pair; its public key travels to the phone, which encapsulates and returns the KEM ciphertext. Use a securely provisioned, unique 256-bit pairing PSK for transcript authentication with HMAC-SHA-256, plus mutual confirmation of the KEM-derived keys. Bind version, suite, peer roles/identities, fresh nonces and complete exchange into authentication/derivation. Exact framing/transcript encoding belongs in the feature spec; this is an experimental application protocol, not TLS.

**BLE-3:** Fragment both KEM objects at the application layer; do not confuse ATT fragmentation with radio packets. For the selected writes/indications, usable bytes are at most min(ATT_MTU - 3, 512), minus the protocol header. Support MTU 23. Bound message length, offsets, reassembly memory and timeout; reject malformed/overlapping fragments and discard state on disconnect.

**BLE-4:** Derive session-specific directional keys/nonce material with HKDF. Encrypt application data with AEAD; authenticate message metadata, enforce unique nonces and sequence-based replay rejection. Deliver plaintext only after tag verification. Reconnect establishes fresh state; never log PSKs, private keys or shared secrets.

**BLE-5:** G2 acceptance is the physical Pi/Pixel demonstration of authenticated ML-KEM-768 key establishment over BLE, encrypted application transfer and tag-verified phone plaintext, supported by reviewed capture/screen evidence and reported sizes/counts/timings without secret material. Native checks substantiate small/larger-MTU framing, tamper/wrong-peer rejection and session cleanup. Additional physical robustness checks remain documented follow-up work; they do not block G2 closure or progression to G3. Unperformed checks must remain explicitly labelled.

![BLE discovery, fragmented key establishment and encrypted data](../docs/diagrams/ble.svg)

## HWE — qtooth-hwe2dr

Entry points:

    qtooth-hwe2dr monitor --arm
    qtooth-hwe2dr disarm

| Component | Stack |
|---|---|
| Control CLI | Rust/Cargo, clap; versioned ioctl for policy/arm/disarm and poll/read notification events from /dev/qtooth-hwe2dr |
| Kernel driver | Rust first, minimal C glue if needed; SPI configuration/status, GPIO threaded IRQ, event read/acknowledgment and reporting; no reset callback |
| FPGA | Passive capture/features, fixed-point VAE, qualification/reset FSM and timestamped event FIFO in RTL; Yosys, nextpnr-ice40, IceStorm/iceprog; known-good bitstream packaged by Yocto |
| Offline tools | Rust tch with pinned LibTorch for training; Rust feature/reference/quantization/export tools; RTL simulation and place-and-route reports; no LibTorch on Pi |
| Fault target | Rust no_std RP2040 firmware, rp2040-hal/PIO as needed; repeatable normal/degrading/stuck-bus modes |

**HWE-1:** When armed, FPGA passively samples SDA/SCL through synchronized input-only taps, decodes transactions and forms bounded, timestamped feature windows. It never addresses targets, ACKs bytes, generates SCL or drives either bus line. Preserve address/direction and ACK/NACK context; normal read-ending NACKs and supported clock stretching are not automatically faults. Active control is confined to a separate reset output.

**HWE-2:** Run the trained, quantized VAE in FPGA fabric. Use deterministic latent-mean reconstruction scoring and validate it against software; no random latent sampling in deployed inference. A compatible model manifest binds weights, normalization, feature schema/window, precision and thresholds to the bitstream.

**HWE-3:** The FPGA qualifies an action only when anomaly AND all hardware guards hold: explicitly armed, compatible valid model/window, persistence threshold met, configured target allowed, cooldown/settling interval elapsed, retry budget available and no reset in progress. Otherwise continue observing without reset or action IRQ. Linux configures these limits before arming; FPGA enforces them locally and rechecks them at reset assertion. Attribution is limited to the configured test target, not inferred physical certainty.

**HWE-4:** After completing the reset pulse, FPGA enqueues a notification and asserts IRQ while unread events remain. Records carry version, sequence, target, score/reason, decision/assertion/release timestamps and action status. The kernel's threaded handler reads/acknowledges records over SPI and exposes them to the CLI; it never authorizes, repeats or performs the reset. IRQ service, FIFO capacity and SPI acknowledgment must not gate the reset FSM; bound the FIFO and count notification overflow. No sleeping SPI work in hard IRQ context.

**HWE-5:** FPGA is the sole reset owner through the external reset stage. Qualify a STOP/bus-idle boundary before a preventive reset; both lines HIGH during a data bit is insufficient. A configured finite deadline permits a target-approved emergency abort if no boundary arrives; the Pico bench firmware must tolerate this. Pulse width and target settling time are clock-counted and configured from device requirements/bench evidence. Clear old feature windows and mask expected reboot transients before fresh decisions. Power-up/configuration defaults are disarmed/reset-released. Acknowledged disarm cancels a pending action; an asserted pulse completes its minimum width before release. The target boots its own I2C firmware; workload software tolerates an interrupted transfer and resumes on readiness. IRQ handling adds no automatic I2C recovery/reinitialization.

![FPGA-owned guarded reset and notification-only kernel path](../docs/diagrams/hwe-runtime.svg)

**HWE-6:** Train offline on representative normal runs with reconstruction plus KL loss; use held-out complete runs/fault trajectories without overlapping-window leakage or injector-state inputs. Compare simple thresholds/AE/VAE; report false alarms, missed failures and warning lead time. Use matched trials without intervention to establish reference failure behavior. Prediction requires observable precursors and warning lead time exceeding the FPGA boundary wait and required reset/recovery time plus margin; Linux notification time is excluded. Abrupt faults may only be detected after onset.

**HWE-7 — measurement contract:** Scope SDA, SCL, physical RESET_N at the target and a dedicated FPGA DECISION marker. The marker rises when an anomaly passes qualification, before the bounded boundary wait, and falls on reset release/cancellation. FPGA timestamps mark the marker and commanded reset edges in one clock domain; physical reset-stage delay is measured at the target. Record the tick rate. IRQ occurs after the pulse and is a separate notification-latency observation.

| Interval | Meaning |
|---|---|
| DECISION rise to RESET_N fall | Boundary wait, final guard check and electrical reset-assertion delay; excludes prior observation/inference |
| RESET_N fall to rise | Actual reset pulse width at the target |
| RESET_N rise to first successful target transaction | Observed return to service, including boot/initialization and host polling cadence |
| Defined bus precursor to RESET_N fall | Full measured reaction, including windowing, inference, persistence and reset scheduling |

Use recognizable trace events and matched fault trials for precursor/lead-time measurements; the marker alone does not prove prediction. Observe healthy-target continuity and notification loss separately. Do not subtract unsynchronized Linux and FPGA timestamps or label decision-to-reset delay as total recovery time.

![Offline training and deployment](../docs/diagrams/hwe-training.svg)

## GATES — implementation order and evidence

1. **G1:** Pin/build/boot the minimal BSP-compatible RT image; establish baseline measurements.
2. **G2 — complete:** The physical PQC-over-BLE demonstration establishes authenticated keys, transfers encrypted messages and shows verified Pixel plaintext, including retained-session reuse. Accepted 2026-11-25 under the user-authorized BLE-5 demonstration boundary.
3. **G3 — next:** Verify board revision/pinout, synthesize representative inference, and confirm capture/model fit, timing and Rust kernel API availability. No assumed hls4ml iCE40 backend.
4. **G4:** Validate capture with a logic analyzer; implement repeatable degradation and abrupt-fault trials with healthy-device traffic. Confirm controller/target clock-stretch compatibility before using it as a precursor.
5. **G5:** Establish model evidence, fixed-point/RTL agreement, FPGA guard/pulse/disarm behavior and HWE-7 timing measurements. Delay IRQ servicing/SPI acknowledgment and exercise FIFO overflow to verify reset timing remains independent; measure notification latency and healthy-device disruption separately. Numeric deadlines/thresholds follow measurements; observed maxima are not proven worst-case bounds.

G2 evidence and limitations are retained in the [course results](../docs/courses/G2_Architecture_and_Practical_Guide.md#9-recorded-physical-and-software-results), [demonstration GIF](../docs/experiments/pqc-over-ble-evidence-vid.gif) and [annotated capture](../docs/experiments/pqc-over-ble-cap.txt). The recorded standalone Pi error and unperformed physical robustness checks remain follow-up items, without reopening the completed demonstration milestone. G3 is the next milestone; its hardware/fit/timing checks have not yet been established.

## References

[Yocto releases](https://www.yoctoproject.org/development/releases/) · [Cargo recipes](https://docs.yoctoproject.org/dev/ref-manual/classes.html#cargo) · [Pi RT kernel](https://www.raspberrypi.com/documentation/computers/linux_kernel.html) · [BlueR](https://github.com/bluez/bluer) · [OpenSSL ML-KEM](https://docs.openssl.org/3.5/man7/EVP_KEM-ML-KEM/) · [Android GATT](https://developer.android.com/develop/connectivity/bluetooth/ble/transfer-ble-data) · [Kernel IRQs](https://docs.kernel.org/core-api/genericirq.html) · [Kernel Rust](https://docs.kernel.org/rust/quick-start.html) · [tch/LibTorch](https://github.com/LaurentMazare/tch-rs)

Figures are PlantUML-generated SVGs with opaque backgrounds for viewer compatibility; adjacent .puml files are sources. Prose owns requirements.
