## Purpose

Establish attributable bidirectional BLE transport between the G1 Pi and a
generic foreground Pixel client before implementing the authenticated protocol
required for DESIGN G2. Acceptance here proves transport only, not BLE-2/4/5.

## ADDED Requirements

### Requirement: Explicit feature packaging

The project SHALL provide an opt-in Pi RT diagnostic build containing the
`qtooth-ble` executable, owned by `meta-qtooth-ble`, with locked Cargo sources and
identifiable native/runtime dependencies. Default platform selections MUST
remain feature-free, and the BLE selection MUST use a separate build/evidence
directory. Neither target compiler nor FPGA packages SHALL be introduced
(DESIGN BASE-1..2).

#### Scenario: Select and inspect BLE diagnostic image

- **WHEN** the developer selects the BLE RT diagnostic build
- **THEN** its resolved metadata and image manifest identify the BLE layer/package and dependencies
- **AND** the executable's ARM64 identity is inspectable, while default base/diagnostic selections and prior G1 output directories remain unchanged

### Requirement: Discoverable transport-only probe

`qtooth-ble probe` SHALL advertise an explicitly named, unencrypted test service
as the Pi peripheral/GATT server. It SHALL expose stable, documented custom
UUIDs for the service, a write-with-response RX characteristic and an
indicate-only TX characteristic. It MUST NOT present the probe as encrypted
`send` or as a completed G2 handshake (DESIGN BLE-1).

#### Scenario: Generic phone discovers the service

- **WHEN** the command runs on the G1 Pi and the foreground generic Pixel BLE inspector scans and connects
- **THEN** the inspector discovers the documented service/RX/TX UUIDs and can enable TX indications through its subscription control
- **AND** the command identifies itself as an unauthenticated transport test

#### Scenario: Adapter or service is unavailable

- **WHEN** the required controller, system D-Bus, BlueZ or registration capability is unavailable
- **THEN** the command exits nonzero with an actionable phase-specific diagnostic and releases any registrations already acquired

### Requirement: Bounded bidirectional test transfer

After TX subscription, RX SHALL accept the public four-byte token
`51 54 42 31` through an ordinary write with response at offset zero and indicate
the identical token through TX. The probe SHALL require no more than the MTU-23
value budget and SHALL report the observed ATT MTU from the request rather than
claiming that 512 bytes is the negotiated MTU (DESIGN BLE-3).

#### Scenario: Successful round trip

- **WHEN** the subscribed phone writes `51 54 42 31` with response
- **THEN** the write succeeds and the phone receives exactly those four bytes as an indication
- **AND** the Pi records the observed MTU, byte counts and whether indication confirmation was observed, without calling that an authenticated application receipt

#### Scenario: Unsupported test write

- **WHEN** a write has an incorrect token/length, nonzero offset, unsupported operation or no active TX subscription
- **THEN** it is rejected without an echo, unbounded buffering or a successful round-trip result

### Requirement: Bounded probe lifecycle

The probe SHALL allow at most 30 s from advertising until connection/subscription
readiness and at most 120 s from readiness for the test exchange, using monotonic
elapsed time. Successful confirmed exchange SHALL end the probe promptly;
timeouts and premature disconnects MUST NOT report success. These are probe
phase limits, not implemented cryptographic-handshake or kept-session behavior.

#### Scenario: No ready phone

- **WHEN** no connected, subscribed phone is ready within 30 s
- **THEN** the probe stops advertising, releases its GATT registration and exits nonzero with the failed phase

#### Scenario: Ready phone makes no successful exchange

- **WHEN** 120 s expires after readiness without a confirmed valid round trip
- **THEN** the probe disconnects its test peer where reachable, releases state and exits nonzero without extending the deadline for invalid traffic

### Requirement: Cleanup and repeatability

Success, cancellation, link loss and ordinary failures SHALL release the probe's
advertisement, GATT registration, subscription/task state and owned test link,
without stopping shared BlueZ/D-Bus services or disturbing unrelated links.
Concurrent local probes SHALL fail visibly rather than create ambiguous tests.

#### Scenario: Interrupt and restart

- **WHEN** the developer interrupts an active probe or the phone disconnects, then starts a fresh probe
- **THEN** no old application registration or pending transfer is reused and a new scan/connect/subscribe/round trip can succeed
- **AND** shared platform services remain running

### Requirement: Reproducible physical learner experiment

The project SHALL provide a concise lab with build/deployment attribution,
commands, expected results and generic-phone actions. Acceptance MUST include
the HW-1 Pi/Pixel's discovery, write/indication round trip, timeout, disconnect
and restart evidence. It SHALL record actual MTU and client/version; unavailable
MTU control MUST be disclosed rather than claimed as a physical MTU-23 test.
Raw evidence SHALL stay in ignored local/build roots; published summaries MUST
remove workstation paths, peer addresses, credentials and secrets. G2 SHALL
remain open until BLE-5 is completed by later milestones.

#### Scenario: Learner performs and explains the experiment

- **WHEN** the learner builds the selected image, verifies it on the physical Pi, runs `qtooth-ble probe`, enables TX indications and writes the documented token
- **THEN** the phone displays the matching indication and the Pi reports the expected transport events and termination
- **AND** the learner repeats no-phone timeout, disconnect/interrupt and restart checks, retaining measured results separately from unperformed checks
- **AND** the lab explains Rust/D-Bus/BlueZ boundaries and asks optionally why ATT confirmation does not prove decryption; it adds no quiz-dependent runtime behavior
