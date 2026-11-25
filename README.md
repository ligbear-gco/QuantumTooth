# QuantumTooth

Minimal Yocto Linux RT platform for two independent demonstrations:
* **Post-quantum cryptography (PQC) over Bluetooth Low Energy (BLE)**
* **FPGA-assisted edge-ai enhanced error detection/recovery**
> G1 and G2 are complete: platform bring-up and demonstrated post-quantum BLE communication.
> Next: G3 — FPGA feasibility, board qualification and Rust kernel API checks.

## Skills demonstrated

> The project is built using AI SDD (Specification Driven Development) with Openspec.

| Embedded Linux | Systems syntax & programming | Post-quantum cryptography (PQC) | Connectivity | Android integration |
|---|---|---|---|---|
| Yocto (Wrynose) · Kas · BitBake · ARM64 · PREEMPT_RT | Rust · OpenSSL · DTS · BB | ML-KEM-768 · AES-256-GCM · HMAC | BLE GATT · BlueZ/D-Bus | React Native · TypeScript · Kotlin · JNI/NDK |
| Board bring-up and RT measurements | Shared crypto core, bounded sessions and kernel init fixup | Authenticated post-quantum key establishment | Physical tests and capture analysis | Standalone APK and verified phone decryption |

## G1 — physical platform results

Brought up a **Raspberry Pi 4 Model B, 4 GB** with pinned ARM64 Linux 6.18.33.
Contains:
* RT/non-RT profiles, minimal and diagnostic images, verified SD deployment
* Bluetooth firmware/LE readiness
* I2C/SPI/GPIO/USB controller checks. 
* Fixed CMAC packaging and enabled I2C1/SPI0 with a project device tree (DT) overlay while preserving Bluetooth's UART and reserving SPI CE0 for the FPGA.

**Base userspace:** BusyBox 1.37.0, systemd/networkd 259.5, D-Bus 1.16.2, BlueZ 5.86, glibc 2.43, OpenSSL libcrypto 3.5.9.
**Pi support:** HCI UART, CMAC, I2C bcm2835/i2c-dev and SPI bcm2835 modules; BCM4345C0/C5 firmware and `qtooth-platform-buses`. Diagnostic adds OpenSSH/access policy, rt-tests, stress-ng, procps-ps plus dependencies.

| Image | WIC.bz2 MiB | Used RAM MiB | Boot s | Idle max µs | Loaded max µs |
|---|---:|---:|---:|---:|---:|
| RT base | 54.87 | 112.42 | 6.168 | — | — |
| Non-RT base | 54.78 | 108.26 | 5.407 | — | — |
| RT diagnostic | 59.90 | 112.53 | 6.372 | 56 | 337 |
| Non-RT diagnostic | 59.82 | 107.84 | 5.509 | 68 | 323 |

> RAM measurment is done with three settled samples on instrumented copies.
> Boot is one image kernel-to-systemd observation, excluding firmware.
> Diagnostic maxima cover three 60-second trials per condition/profile base runs.
> Raw WIC sizes are 338.80 MiB (base), 359.60 MiB (diagnostic).

**Result:** 4 image variants verified; 12 successful scheduling trials,
720000 samples and no overflow. RT showed mixed latency results, with
no consistent improvement across all statistics or proven deadline bound, yet 
it cannot be tested without a proper RT software, the goal was merely to gain
knowledge on how to integrate it into a kernel.

[Complete course and final lab](docs/courses/G1_Architecture_and_Practical_Guide.md) ·
[G1 acceptance record](openspec/changes/archive/2026-11-23-close-pi4-physical-g1/verification.md) ·
[Design](openspec/DESIGN.md) · [Hardware](openspec/HARDWARE.md)

## G2 — Post-quantum cryptography (PQC) over BLE

Built a Rust BLE sender on Raspberry Pi and a standalone React Native Android
receiver sharing the same Rust crypto core. 
Ensures:
* Pair-authenticated **ML-KEM-768** establishes keys
* **AES-256-GCM** protects messages.
* The phone displays ciphertext beside plaintext released only after authentication-tag verification.

The post-quantum implementation covers key encapsulation, Pre-Shared Key (PSK)-authenticated
transcripts, directional session-key derivation and replay protection. Large
ML-KEM handshake objects are fragmented across BLE GATT and reassembled in a
bounded Rust receiver shared by embedded Linux and Android.

| Result | Recorded evidence |
|---|---|
| Post-quantum key establishment | **ML-KEM-768** through OpenSSL; shared Rust implementation on Pi and Android |
| Physical link | Raspberry Pi 4 → Pixel 7a; ATT MTU **517** |
| Fragmented handshake | ClientHello / PiOffer / PeerFinish / PiFinish: **1 / 3 / 3 / 1** ATT values |
| Verified messages | **58 / 66 / 53-byte** records; one RX fragment each |
| Retained session | Two verified messages, sequences **0 → 1**, Pi exits **0** |
| Handshake time | App readings **374 ms / 491 ms**; two observed sessions |

<table>
  <tr>
    <td><strong>Physical demo — Pi commands and verified phone text</strong><br><a href="docs/experiments/pqc-over-ble-evidence-vid.gif"><img src="docs/experiments/pqc-over-ble-evidence-vid.gif" width="760" alt="Pi sends encrypted BLE records; Android verifies Hello, world!, Second public message and Bye Bye!, with retained sequences 0 and 1"></a></td>
  </tr>
  <tr>
    <td>
      <strong>Annotated BLE capture public summary</strong></br>
      Peer addresses are replaced by PI/PHONE; host/controller label the HCI boundary.
      Times are relative capture seconds. Length is the supplied capture-frame length,
      not logical object size or ATT value size. Multi-packet objects are summarized;
      raw private evidence is not included. This is not an over-the-air RF trace.

      1. BLE setup and authenticated ML-KEM-768 key establishment
      No.	Time	Source	Destination	Protocol	Length	Info
      19	50.569159	controller	host	HCI_EVT	21	LE Connection Complete
      28	50.598643	PI	PHONE	ATT	11	Exchange MTU Request: 517
      34	50.714235	PHONE	PI	ATT	11	Exchange MTU Response: 517
      125	51.538440	PI	PHONE	ATT	52	RX/TX characteristic discovery
      135	51.658168	PHONE	PI	ATT	13	Enable TX indications
      136	51.658644	PI	PHONE	ATT	9	Subscription acknowledged
      137	51.719098	PHONE	PI	ATT	87	ClientHello — start authenticated handshake
      141	51.737801	PI	PHONE	ATT	21	PiOffer 1/3 — ML-KEM public key + authentication
      147	51.765259	PI	PHONE	ATT	21	PiOffer 2/3
      152	51.795037	PI	PHONE	ATT	9	PiOffer 3/3 — 1249-byte object complete
      158	51.891177	PHONE	PI	ATT	21	PeerFinish 1/3 — KEM ciphertext + authentication/confirmation
      162	51.951170	PHONE	PI	ATT	21	PeerFinish 2/3
      164	52.007716	PHONE	PI	ATT	163	PeerFinish 3/3 — 1152-byte object complete
      167	52.012336	PI	PHONE	ATT	51	PiFinish — final key confirmation
      168	52.065823	PHONE	PI	ATT	9	PiFinish ATT confirmation

      2. AES-256-GCM encrypted message transfer
      170	52.067793	PI	PHONE	ATT	77	58-byte record: metadata 29 + ciphertext 13 + authentication tag 16
      171	52.125669	PHONE	PI	ATT	9	ATT confirmation — transport acknowledged

      3. BLE connection closure
      178	54.183502	host	controller	HCI_CMD	6	Disconnect
      180	54.226662	controller	host	HCI_EVT	6	Disconnect Complete
  <a href="docs/diagrams/g2-pqc-over-ble.svg">PQC-ed fragmented handshake and messaging over BLE sequence</a>
  </td>
  </tr>
</table>

**Result: G2 complete.** Post-quantum-cyphered BLE communication, verified phone decryption
and session reuse demonstrated.

[Complete G2 course, labs and results](docs/courses/G2_Architecture_and_Practical_Guide.md) ·
[Android build setup](app/README.md)
