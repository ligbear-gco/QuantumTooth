## Purpose

Expose fresh and retained authenticated BLE sessions through three shell
commands, with exclusive session ownership, bounded lifetime and honest
best-effort completion reporting (DESIGN BLE-1..4).

## ADDED Requirements

### Requirement: Validated encrypted command inputs

The CLI SHALL expose `send "text"`, `sendkeep "text"` and `closekeep` while
preserving the existing `probe` interface. Data commands SHALL accept 1..512
UTF-8 bytes, including `Hello, world!`; invalid inputs MUST fail before creating
a session or disturbing an existing one. `closekeep` SHALL take no plaintext.

#### Scenario: Validate input before side effects

- **WHEN** an empty, oversized or invalid command is submitted
- **THEN** it fails nonzero before advertising/establishment and leaves any valid kept session unchanged

### Requirement: Fresh send without retained-session interference

`send` SHALL establish fresh authenticated state, transmit one encrypted record
without application ACK/retry, and disconnect/discard its session before returning.
It MUST reject while a kept session exists without replacing or using that session.

#### Scenario: Standalone send

- **WHEN** `qtooth-ble send "Hello, world!"` runs with no kept session and a valid peer
- **THEN** it establishes fresh keys, attempts the record once and cleans up
- **AND** any successful result reports transport completion with delivery unconfirmed

#### Scenario: Send conflicts with kept state

- **WHEN** `send` runs while a kept session exists
- **THEN** it fails visibly without transmitting data or resetting that session's keys, counters or expiry

### Requirement: Retained send across CLI invocations

`sendkeep` SHALL establish a session when absent and reuse the live authenticated
session otherwise. A single worker SHALL retain the connection, keys and counters
after a successful command returns; subsequent CLI processes MUST use advancing
sequences/nonces and MUST NOT extend the absolute lifetime. Failed or expired
sessions MUST NOT be reused or automatically reconnected within an operation.

#### Scenario: Two retained sends

- **WHEN** separate `sendkeep` invocations succeed before expiry
- **THEN** they use one establishment and distinct record sequences under the kept session
- **AND** exiting the first successful CLI does not destroy that retained session

#### Scenario: New invocation after termination

- **WHEN** a kept session has expired/disconnected/failed and a later `sendkeep` runs
- **THEN** the later invocation requires fresh establishment and cannot reuse old secrets or sequence state

### Requirement: Idempotent best-effort close

`closekeep` SHALL attempt a single authenticated close on a reachable kept
session, without waiting for an application reply, then disconnect and discard
all local session state. An absent session SHALL be a successful no-op without
starting discovery. Remote unreachability alone MUST NOT prevent successful
local closure; local cleanup errors MUST fail visibly.

#### Scenario: Close retained session or close twice

- **WHEN** `closekeep` closes a session, or runs when none exists
- **THEN** no usable local session remains and another `closekeep` is harmless
- **AND** the command does not claim confirmed remote processing of the close

### Requirement: Exclusive restricted runtime ownership

Only one local worker/probe SHALL own the application's BLE resources at a time.
Worker IPC SHALL restrict access to the diagnostic owner, validate peer identity
and bound input/queued work; secret keys MUST NOT be persisted or returned to
CLI clients. Racing startups MUST converge on one owner. A competing operation
MUST fail busy rather than interleave, replace state or create a second session.

#### Scenario: Concurrent or unauthorized client

- **WHEN** two clients race startup, a command competes with an active operation/probe, or an unauthorized IPC peer connects
- **THEN** at most one permitted operation owns session state and all conflicting/unauthorized work is rejected without state corruption

### Requirement: Finite session and operation lifecycle

Establishment SHALL finish within 30 s from advertising. A session SHALL expire
120 s after handshake completion regardless of activity. Logical object transfers
SHALL finish within 10 s, individual ATT operations within 5 s and courtesy close
within 2 s, always clipped to the enclosing phase/session deadline. Local startup
and IPC framing/writes SHALL be bounded to 5 s each. All limits use monotonic
time. Invalid traffic MUST NOT refresh them. At expiry, protocol work MUST stop
and secrets MUST be discarded; subsequent resource cleanup SHALL be bounded
to 2 s, preserving shared BlueZ/D-Bus and unrelated links.

#### Scenario: Busy kept session expires

- **WHEN** 120 s elapses after establishment, including during an active send
- **THEN** ongoing protocol work stops and the session is discarded without extending lifetime for a send or courtesy close
- **AND** the worker releases its owned resources within the cleanup bound

#### Scenario: Failure or cancellation cleans up

- **WHEN** establishment times out, a link/subscription is lost, authentication fails, or an operation is cancelled before its result is committed
- **THEN** the command fails visibly and all owned protocol/session resources are discarded
- **AND** a later invocation can start fresh without restarting shared platform services

### Requirement: Best-effort results distinguish local facts

Data commands SHALL return success only after every fragment's selected BLE
operation completes. Success MUST explicitly leave delivery unconfirmed and
MUST NOT imply peer decryption/display. Known transport/protocol/worker failures
SHALL return nonzero with a safe phase-specific reason, without retrying data.

#### Scenario: Successful transport or interrupted transfer

- **WHEN** a data command completes transport, or its transfer fails partway
- **THEN** it respectively returns transport success with `delivery=unconfirmed`, or a nonzero phase-specific failure
- **AND** neither outcome asserts whether the peer received the application message

### Requirement: Attributable packaging and retained-session experiment

The opt-in BLE package SHALL include all three commands and use system
OpenSSL >=3.5 `libcrypto`, with locked sources, no vendored duplicate crypto,
no boot-enabled worker and no compiler added to the image (BASE-1..2).
Evidence SHALL attribute shared sources, executable and system crypto/package
linkage while retaining the G2.1 BlueZ compatibility checks. The project SHALL
provide exact learner-run commands for a native process-level experiment with
two retained sends, conflicting fresh send, close and no-op close, plus focused
ownership/deadline/cleanup checks. Optional physical checks MUST be labelled
separately; no native result closes physical BLE-5 or overall G2.

#### Scenario: Inspect the build and session experiment

- **WHEN** the BLE image/package is cross-built and the learner runs the documented native experiment
- **THEN** system crypto linkage/source attribution and the three command behaviors are observable
- **AND** the guide explains which process retains the connection/counters, offers an optional explain-back question, and states which target/phone checks remain unperformed
