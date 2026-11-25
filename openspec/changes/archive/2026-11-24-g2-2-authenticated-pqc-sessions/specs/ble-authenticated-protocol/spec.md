## Purpose

Establish pair-authenticated post-quantum session keys and protect best-effort
application records over the existing Pi BLE transport, with a shared core
usable by the later Android adapter (DESIGN BLE-2..4).

## ADDED Requirements

### Requirement: Distinct bounded authenticated transport

The authenticated service SHALL use documented UUIDs distinct from `probe`,
ordinary writes with response toward Pi and indications toward the peer
(BLE-1). It SHALL support ATT MTU 23 and larger MTUs using versioned framing
within `min(ATT_MTU - 3, 512)`, with at most 2048 bytes per reassembly object
and one incomplete object per direction (BLE-3). Object identity, kind, total
length, offset and protocol state MUST be validated before accepting bytes.
Overlapping, duplicate, noncontiguous, excessive or wrong-state fragments MUST
fail the session. Incomplete objects MUST time out within 10 s without refresh
by incoming fragments, or sooner at the enclosing deadline.

#### Scenario: Small and large MTU establishment

- **WHEN** paired peers exchange the version-1 handshake at MTUs 23 and 517
- **THEN** each transfer fits its value budget and reconstructs the same canonical logical objects
- **AND** the authenticated service does not accept the probe token as a handshake

#### Scenario: Malformed or incomplete object

- **WHEN** a fragment violates bounds/order/state or an incomplete object reaches its deadline
- **THEN** it is rejected, pending state is discarded and the session cannot deliver plaintext
- **AND** later traffic does not revive that session or extend its deadline

### Requirement: Pair-authenticated ephemeral establishment

Each session SHALL use a fresh Pi ML-KEM-768 key pair, fresh nonces from both
peers, the pair's unique provisioned 256-bit PSK for HMAC-SHA-256 transcript
authentication, and mutual KEM-derived key confirmation (BLE-2). The canonical
transcript SHALL bind version, suite, roles, configured identities, nonces,
public key and KEM ciphertext. Unsupported encodings/suites, authentication or
confirmation failure MUST abort before application data is accepted. HKDF-SHA-256
SHALL derive session-specific, role-separated confirmation and directional
traffic/nonce material (BLE-4).

#### Scenario: Matching pair completes establishment

- **WHEN** both configured peers authenticate the complete exchange and verify the other's key confirmation
- **THEN** they can exchange protected records under the same session identity with distinct directional key material
- **AND** reconnecting requires new ephemeral state rather than resuming old counters or keys

#### Scenario: Wrong peer or modified exchange

- **WHEN** the PSK/identity is wrong, transcript fields are altered or key confirmation is invalid
- **THEN** the handshake fails without accepting application plaintext or retaining a usable session

### Requirement: Authenticated records and replay rejection

Application data SHALL use AES-256-GCM, authenticating version, record type,
session identity, direction, sequence and plaintext length as associated data.
The complete encrypted record SHALL be fragmented only after encryption.
Receivers MUST verify the tag before releasing plaintext (BLE-4). Per-direction
sequences/nonces MUST never repeat under the same key. Wrong session/direction,
replayed/out-of-sequence records, counter exhaustion or invalid tags MUST abort
the session without releasing plaintext from that record.

#### Scenario: Demo plaintext decrypts

- **WHEN** the paired native peer receives a valid encrypted `Hello, world!` record
- **THEN** it releases exactly that plaintext only after authentication succeeds
- **AND** a later kept-session send uses a new sequence and nonce

#### Scenario: Tamper or replay

- **WHEN** ciphertext, authenticated metadata or tag is changed, or a previously accepted record is replayed
- **THEN** the receiver rejects the record and destroys session state without releasing its plaintext

### Requirement: Best-effort application and close records

Each data command SHALL send at most one logical application record, without an
application ACK, receipt, automatic retry or reconnect. Handshake key confirmation
and the selected BLE transport responses/confirmations SHALL remain enabled.
An authenticated close record SHALL request teardown without requiring a reply;
local teardown MUST proceed even if the close cannot reach the peer.

#### Scenario: Transport completes without an application reply

- **WHEN** all fragments of a data record complete their normal transport operation and no application reply is sent
- **THEN** the sender completes without waiting for a receipt or retransmitting the record
- **AND** it does not claim that the receiver decrypted or displayed the message

#### Scenario: Close cannot reach the peer

- **WHEN** the close record fails transport or reaches its finite limit
- **THEN** local connection/key teardown proceeds without retry or an expected close ACK

### Requirement: Private pair provisioning and secret lifetime

Real pair inputs and provisioned artifacts SHALL remain in ignored local/build
storage. No real PSK, private key or shared secret SHALL enter tracked sources,
reports, diagrams, logs or CLI/worker arguments. Builds without pair inputs
MUST reject encrypted data commands before advertising, without substituting a
default/test PSK. Disconnect, expiry, cancellation and protocol/authentication
failure SHALL discard session secrets, reassembly and counters (BLE-2..4).

#### Scenario: Missing provisioning or terminated session

- **WHEN** pair inputs are missing or an established session terminates
- **THEN** no unauthenticated fallback or old-session continuation is available
- **AND** outputs disclose only safe phase/count/timing information, not secret material

### Requirement: Focused reproducible native experiment

The project SHALL provide a learner-run native peer experiment using real crypto
and simulated MTUs 23/517, with exact documented commands and expected public
`Hello, world!` output. Focused checks SHALL cover correct establishment/decryption,
wrong PSK/transcript, tamper/replay, framing rejection/expiry and no-ACK behavior.
Canonical encoding fixtures SHALL supplement shared-library round trips.
Reports MUST label simulated/native evidence separately from physical results;
G2.3/overall G2 and unperformed BLE-5 checks SHALL remain open.

#### Scenario: Learner runs the protocol experiment

- **WHEN** the learner runs the documented experiment and focused rejection checks
- **THEN** matching peers decrypt the public demo and invalid inputs release no plaintext
- **AND** the guide explains fragmentation, authentication and encryption, with an optional question about transport confirmation versus decryption
