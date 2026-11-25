## Purpose

Provide a minimal foreground Android debug app that receives the existing Pi's
authenticated BLE records and visibly demonstrates verified plaintext, with a
reproducible APK build and an honest separation of build and physical evidence.

## ADDED Requirements

### Requirement: Standalone paired ARM64 debug APK

The app project SHALL reside in repository-root `app/`. After documented
toolchain and dependency setup, the project SHALL produce a
debug-signed ARM64 APK through Gradle `:app:assembleDebug` for Pixel 7a / Android
17 (HW-1). The APK MUST contain its JavaScript/assets and required native code,
run without a development server, and use its packaged OpenSSL >=3.5 rather than
Android's system crypto (DESIGN SCOPE/BLE). Build/package inspection SHALL be
the G2.3 implementation acceptance boundary; it MUST NOT imply device execution.

#### Scenario: Reproducible debug build

- **WHEN** the documented build runs with compatible tools and valid private pair input
- **THEN** Gradle succeeds and the documented APK path contains ARM64 native libraries and the embedded application bundle
- **AND** the build record identifies tool/dependency versions and the APK checksum without claiming an unperformed Pixel run

### Requirement: Existing private pair provisioning

The Android build SHALL consume the same ignored pair input as the Pi without
overwriting it, reject missing/invalid input clearly, and rebuild affected native
outputs when its contents change. Pair/session secrets MUST remain outside the
JavaScript bundle, logs and tracked files; provisioned APKs, signing material,
caches and raw evidence MUST remain in ignored private storage (BLE-2/4).

#### Scenario: Missing or changed pair input

- **WHEN** the pair input is absent/invalid or its contents change
- **THEN** the build respectively fails clearly or regenerates affected provisioned outputs
- **AND** it never substitutes a default key, overwrites an established pair or prints secret bytes

### Requirement: Foreground authenticated BLE reception

The app SHALL support user-initiated scan, selection, connection, service
discovery and TX indication subscription to the existing secure Pi service.
It SHALL interoperate with the version-1 protocol, framing and protection in
`ble-authenticated-protocol`, accepting `send` and repeated `sendkeep` records
without changing Pi command semantics. It MUST preserve ordinary writes with
response, indications/ATT confirmations and application-layer MTU bounds (BLE-1..4).
No application ACK, retry or automatic reconnect SHALL be added.

#### Scenario: Receive standalone and retained sends

- **WHEN** the paired foreground app connects/subscribes while a Pi data command is advertising
- **THEN** it completes authenticated establishment and releases each valid received message only after verification
- **AND** repeated retained sends advance the sequence under one live session, while a later fresh connection establishes new state

### Requirement: Bounded native lifecycle and safe failure

The app SHALL enforce protocol deadlines independently of UI scheduling, discard
session/reassembly/counter state on failure, expiry, disconnect or native module
invalidation, and release its connection when leaving the foreground. Authenticated
close SHALL terminate the session without an application reply. Invalid records
MUST release no plaintext (BLE-3/4); failures MUST require an explicit new attempt.

#### Scenario: Failure, close or interrupted foreground session

- **WHEN** authentication/reassembly fails, a deadline expires, close arrives, the link is lost or the app leaves the foreground/reloads
- **THEN** owned active state is discarded and the UI reports the resulting phase without automatic reconnection
- **AND** rejected records never produce a success result or expose unverified plaintext

### Requirement: Clear recoverable operating errors

The app SHALL request the required Nearby devices permissions and display
actionable errors for denial, unavailable/disabled Bluetooth, scan/setup timeout
and native crypto initialization failure. User controls SHALL remain usable for
a later explicit attempt; error handling MUST NOT bypass authentication.

#### Scenario: Permission denied or Bluetooth disabled

- **WHEN** required access is denied or Bluetooth is unavailable/disabled
- **THEN** the screen explains the failed prerequisite and offers an appropriate user action
- **AND** it does not start an authenticated session or display decryption success

### Requirement: Recording-friendly verified result

The app SHALL provide one portrait screen with a dark technical appearance,
readable status text, scan/connect and disconnect controls, a ciphertext preview,
and prominent verified plaintext. The primary result MUST fit without scrolling
on the Pixel with system insets. Success MUST follow real record authentication,
never a preset demo string, handshake status or transport completion. A last
verified result SHALL remain readable after normal disconnection, explicitly
separate from current link status, and SHALL clear when a new attempt begins.

#### Scenario: Public demo decrypts visibly

- **WHEN** the phone verifies a received encrypted `Hello, world!` record
- **THEN** the screen displays exactly that plaintext with a clear verified label and the received ciphertext preview
- **AND** normal session closure preserves the labelled last result while showing the disconnected link

### Requirement: Attributable bounded diagnostics

The screen SHALL show actual observed MTU, displayed-record RX fragment count,
authenticated sequence, record length and measured handshake time with a defined
scope. Diagnostics SHALL be bounded and omit addresses, provisioned identities,
keys and raw transcripts. Requested MTU or a small application chunk cap MUST NOT
be presented as a measured small negotiated MTU (BLE-3/5).

#### Scenario: Inspect a received message

- **WHEN** the operator opens diagnostics for a verified message
- **THEN** counts/lengths refer to that record, sequence is authenticated, and handshake timing excludes scanning/connection setup
- **AND** unmeasured values remain unavailable rather than using illustrative placeholders as results

### Requirement: Reproducible functional handoff and honest G2 evidence

The guide SHALL provide exact setup/build/output instructions and a short
operator-run ADB/Pixel experiment: launch without a development server, receive
two retained public messages, close, then establish fresh state explicitly.
Expected observations and an optional transport-versus-decryption question SHALL
be documented without quiz-dependent app behavior. The README UML SHALL remain
an illustration with links to actual evidence. APK build, native checks and
physical observations MUST be labelled separately; overall G2 SHALL remain open
until Pixel decryption and remaining BLE-5 evidence are recorded.

#### Scenario: Operator performs the functional experiment

- **WHEN** the operator follows the documented installation and Pi send/close steps
- **THEN** expected observations cover exact verified text, advancing retained sequences, closure and a fresh manual session
- **AND** the record states which steps were actually performed, keeps raw artifacts private and claims no unperformed BLE-5 result
