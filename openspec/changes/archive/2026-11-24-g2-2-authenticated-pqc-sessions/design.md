## Context

See [proposal.md](proposal.md) for scope and learning outcomes. The existing
standalone `components/ble/` crate implements only `probe`, with a process lock,
BlueR callbacks and bounded cleanup. Its recipe has no OpenSSL dependency, and
`tools/collect-ble-evidence` explicitly rejects crypto linkage. G2.1 was accepted
at MTU 517; physical MTU 23 and authenticated phone operation remain unverified.

This design crosses crypto, local IPC and Yocto boundaries, so a design artifact
is necessary. Authorities are DESIGN SCOPE/BASE-1..3/BLE-1..5 and HARDWARE
HW-1/HW-3; both remain unchanged.

## Goals / Non-Goals

**Goals:** keep protocol state reusable by the future Android native core;
make one process own every live session; exercise real crypto through a native
peer without needing Android; preserve explicit evidence limits.

**Non-Goals:** application receipts/retries, reliable delivery, multiple paired
phones, automatic reconnect, boot-enabled services, Android/JNI implementation,
production credential storage or overall G2 acceptance.

## Decisions

### 1. Shared core and platform adapters

Keep `components/ble/Cargo.toml` as the build entry point and workspace root;
add a nested path library member for framing, canonical encodings, crypto and
protocol state, covered by workspace tests and the existing lockfile.
It must not depend on BlueR, Linux IPC or Android. The executable owns the Pi
GATT adapter, worker and CLI. A native test peer uses the same library's peer
role through a simulated fragment transport. Freeze byte-level fixtures to
check encodings independently of a same-library round trip.

```text
CLI -- restricted Unix socket --> worker -- BlueR/D-Bus --> BlueZ --> BLE
                                   |
                              shared Rust core
                                   |
                         native peer / future JNI
```

A Yocto layer supplies build/package metadata; Linux/BlueZ own Bluetooth, not
the shared crypto core. A transport trait allows focused worker tests without
adding a production simulator command or claiming radio evidence.

Alternative: duplicate protocol logic in an Android implementation. Rejected
because it would create two encodings and state machines to maintain.

### 2. Distinct authenticated GATT service and bounded framing

Advertise `QTooth Secure` with service/RX/TX UUIDs
`b6632000-7e7b-4fb5-9dc4-93f4152a0e7a`,
`b6632001-7e7b-4fb5-9dc4-93f4152a0e7a` and
`b6632002-7e7b-4fb5-9dc4-93f4152a0e7a`. Keep the probe UUIDs intact.
Reuse its strict Request/offset-zero/subscription/selected-peer validation.
Stop owned advertising when selecting the peer. BLE address selection is not
authentication; only the configured pair PSK/identities establish that.

Each write/indication contains one 8-byte header followed by nonempty bytes:
`version:u8 | kind:u8 | object_id:u16 | total:u16 | offset:u16`, big endian.
Version is 1; kinds 1..5 are ClientHello, PiOffer, PeerFinish, PiFinish and
EncryptedRecord. Object IDs start at zero independently per direction and
increase without wrap. The value budget is `min(ATT_MTU - 3, 512)`; default to
MTU 23 until valid peer MTU information is available. Never use ATT long writes.

Keep at most one reassembly object per direction, capped at 2048 bytes each,
with tighter type-specific lengths. Require contiguous offsets, unchanged
headers, the next object ID and the expected protocol state. Reject gaps,
overlaps, duplicates, interleaving and overflow before copying/allocating.
Use a capacity-one callback handoff; no unbounded channel or crypto inside a
D-Bus write callback. A pending indication is owned and cancellable.

MTU 23 leaves 12 object bytes per fragment. Even `Hello, world!` needs this
framing because its key-establishment objects are much larger than its data.
Alternative: use only a large negotiated MTU. Rejected by DESIGN BLE-3.

### 3. Pair-authenticated establishment and explicit encoding

Use one suite: ML-KEM-768/HMAC-SHA-256/HKDF-SHA-256/AES-256-GCM. Pair inputs
contain a unique random 32-byte PSK and two fixed 16-byte public peer identities;
identities are labels, not Bluetooth addresses or secret authenticators.
Generate fresh 32-byte nonces on both sides and an ephemeral Pi ML-KEM key pair
for each new session, using OpenSSL randomness.

All logical handshake bodies use fixed field order, big-endian integers, exact
length checks and no trailing bytes. Define `E(kind, body)` as kind:u8,
body-length:u16 and body. The domain string is ASCII `QTOOTH/G2.2/v1`.
ClientHello contains version:u8, suite:u16 (=1), role:u8 (=1 for peer),
peer identity, Pi identity and peer nonce. OfferCore contains role:u8 (=2 for
Pi), Pi nonce and the 1184-byte raw public key. ResponseCore is the 1088-byte
raw KEM ciphertext. These fixed bodies fit the 2048-byte object cap.

1. Peer sends ClientHello. Pi checks the version, suite and expected identities.
2. Pi sends OfferCore plus PSK HMAC over the domain, `pi-offer` label,
   E(ClientHello) and E(OfferCore). Peer verifies it before encapsulation.
3. Let T be domain || E(ClientHello) || E(OfferCore) || E(ResponseCore).
   Peer sends ResponseCore, HMAC(PSK, `peer-auth` || T) and its derived-key
   confirmation. Pi verifies the PSK HMAC before decapsulation, then verifies
   confirmation. Every HMAC/confirmation is 32 bytes, compared in constant time.
4. Pi sends its derived-key confirmation. Peer verifies it before accepting
   application records. Pi does not start data until that finish's indication
   completes. This finish confirms keys, not delivery of later application data.

HKDF-Extract uses SHA-256(T) as salt and the KEM secret as input. HKDF-Expand
uses distinct domain-prefixed role/direction labels for two 32-byte traffic
keys, two 4-byte nonce prefixes, two 32-byte confirmation keys and a 16-byte
session ID. Peer confirmation is HMAC(peer-confirm-key, `peer-finished` ||
SHA-256(T)); Pi confirmation is HMAC(pi-confirm-key, `pi-finished` || SHA-256(T)
|| peer-confirmation). Commit the exact label registry and canonical byte
fixtures before integrating the adapters; no implicit serialization defaults.

Alternative: bare KEM establishment. Rejected because a KEM alone does not
authenticate the intended peer. This remains an experimental application
protocol, not TLS or a claim of independent cryptographic review.

### 4. Best-effort encrypted records, with no application ACK

Record header: version:u8, type:u8 (1=data, 2=close), direction:u8
(1=Pi-to-peer, 2=peer-to-Pi), session-id:16 bytes, sequence:u64,
plaintext-length:u16; all integers big endian. Authenticate this entire header
as AEAD associated data. Append ciphertext and a 16-byte tag. Encrypt the
complete record before fragmenting; release no plaintext before tag validation.
Data is UTF-8, 1..512 bytes; close has zero plaintext bytes. Maximum data record
size is 557 bytes, independent of ATT limits.

Per-direction sequences start at zero. Nonce = that direction's 4-byte derived
prefix || sequence:u64. Reserve each sequence once; never re-encrypt/retry with
it. Require the next expected sequence on receive; wrong session/direction,
duplicate, gap, wrap or authentication failure destroys the session. Even
best-effort delivery requires nonce uniqueness and replay rejection (BLE-4).

A data command sends one logical record and awaits only normal transport
completion for its fragments. The result states `delivery=unconfirmed` even
when transport completes. Known transport/authentication failures return nonzero;
there is no application receipt, ACK deadline or automatic resend/reconnect.
Neither stack-level BLE behavior nor handshake confirmation is removed.

`closekeep` attempts one authenticated close record if a live session is
reachable, waits only for bounded transport completion, then always discards
secrets/disconnects. It never waits for a close reply. An unreachable peer is
reported as a local close with remote closure unconfirmed, not a data failure.

Alternative: authenticated data receipts/retries. Removed by the user's scope
decision; live teaching drafts will be reconciled during implementation.

### 5. On-demand worker, ownership and commands

Launch an internal worker from the CLI when `send`/`sendkeep` has no owner.
Use a root-owned runtime directory (0700), Unix socket (0600), peer-credential
checks, a bounded/versioned request frame and exclusive file locking. Root-only
access matches the current diagnostic shell; no new user/group policy is needed.
Do not send keys through IPC or put plaintext/PSKs in worker launch arguments.

The worker owns BlueR, the session, counters and cleanup. Share exclusive
ownership with `probe` so neither can register a competing application. One
active local operation is admitted; racing startups converge on one owner and
competing operations fail busy. Only the lock owner may remove its stale socket
after validating the runtime path; never delete/bypass another owner's lock.

| Command | Worker behavior |
|---|---|
| `send "text"` | Reject if kept session exists; otherwise establish fresh keys, send once, clean up and exit. |
| `sendkeep "text"` | Establish if absent, otherwise reuse live session with next sequence; send once, return result, keep worker alive. |
| `closekeep` | No worker/session is a successful no-op; otherwise attempt close once, clean up and exit. |

Validate input before acquiring Bluetooth resources. A startup/handshake failure
does not leave a retained worker. IPC loss before an operation's result is
committed cancels that operation/session; a successful `sendkeep` result is the
handoff after which CLI exit must leave the worker alive. SIGINT/SIGTERM,
subscription loss, link loss, malformed input or crypto failure clean up without
reusing secrets; after a failure a later invocation must establish fresh keys.

Alternative: save keys/counters to disk between CLI calls. Rejected because
live connection ownership and atomic nonce state belong in one process.

### 6. Finite timing and cleanup

Use monotonic time and these planning defaults, not measured latency claims:

| Operation | Limit |
|---|---|
| Worker startup/local IPC request or result write | 5 s each; active CLI result wait includes the bounded protocol operation |
| Establishment, advertising through mutual confirmation | 30 s total |
| Session lifetime from handshake completion | 120 s absolute; sends never refresh it |
| Complete logical object send/reassembly | 10 s, never refreshed by fragments |
| Individual ATT transfer | 5 s |
| Courtesy close | 2 s |
| Resource cleanup after stopping protocol work | 2 s |

Nested work uses the earliest applicable deadline. Expiry cancels transfers and
discards secrets immediately; courtesy close cannot extend lifetime. Drop owned
GATT/advertising/task handles and disconnect only the selected owned link;
preserve shared BlueZ/D-Bus services. An idle kept worker exits at expiry.
No hard real-time guarantee is implied.

### 7. Crypto bindings, provisioning and packaging

Use Rust OpenSSL bindings and a narrow reviewed FFI wrapper only for missing
KEM APIs. OpenSSL documents ML-KEM support from 3.5 and EVP output-length/error
contracts: [ML-KEM](https://docs.openssl.org/3.5/man7/EVP_KEM-ML-KEM/),
[encapsulation](https://docs.openssl.org/3.5/man3/EVP_PKEY_encapsulate/).
The checked-out OE tree contains `openssl_3.5.9.bb`; recipe presence is not
proof that the selected sysroot/provider works. Check native and target
capabilities before dependent integration; do not silently change algorithms,
upstream pins or vendor a second OpenSSL.

Use RAII wrappers for EVP allocations, validate every return and actual output
length, document pointer ownership/lifetimes, and contain unsafe operations in
one module with Doxygen contracts. Keep secret buffers out of Debug/Clone/logs,
cleanse owned secret buffers on teardown and promptly free EVP private keys;
do not claim complete elimination of compiler/library copies.

Supply pair inputs from ignored local files to local builds, not command-line
key literals or tracked configs. A build without pair inputs remains usable as
`probe` but rejects encrypted commands before advertising; no default/test PSK
is accepted by the production path. Tests inject per-run disposable pair inputs.
Ensure input content changes invalidate the Yocto task/cache without printing
it. Embedded pair keys are extractable from binaries: provisioned binaries,
images, task logs and sstate require private storage, not publication.

Extend the recipe to include the nested library and locked sources, depend on
OpenSSL and link system `libcrypto.so.3`. Update the collector to attribute the
library/provider and shared sources, preserving BlueZ patch/package checks and
default image selection. No compiler, vendored crypto or boot service is added.

### 8. Focused acceptance and learner experiment

Provide a native test-peer harness over the same fragment interface and a
process-level worker test seam. A documented Cargo test filter runs real crypto
at simulated MTUs 23 and 517, prints only the public demo plaintext/safe phase
results, and exercises two `sendkeep` client processes followed by `send`
rejection and `closekeep`. Use controlled clocks for deadline checks.

Minimum checks: successful authenticated decryption; wrong PSK/transcript and
tampered/replayed record rejection; malformed/oversize/incomplete fragments;
session reuse, sequence advancement, absolute expiry, explicit close/no-op,
ownership and error cleanup; no-ACK behavior. Verify existing probe regressions,
the Pi build and system linkage. This is focused engineering evidence, not an
audit or exhaustive protocol validation. Optional explain-back: why can a kept
worker outlive its CLI while a successful send still leave delivery unconfirmed?

## Risks / Trade-offs

- Shared code can share encoding mistakes → fixed canonical fixtures and
  negative inputs in addition to round trips; physical interoperability in G2.3.
- Small-MTU handshake may exceed chosen limits → measure actual phases; report
  failures, adjust bounded defaults explicitly, never claim mock timing as radio cost.
- Embedded PSK/build caches expose demo credentials → keep provisioning and
  provisioned artifacts private; never publish secrets or unredacted evidence.
- Best-effort transport cannot prove decryption → honest CLI result plus native
  receiver evidence now; Pixel observation and remaining BLE-5 evidence later.
- Native/target providers or bindings differ → gate integration on both capability
  probes and narrow FFI review; report blockers without inventing a pass.

## Migration Plan

Build the existing opt-in BLE diagnostic selection with the extended package;
retain attributable source/image/library evidence under ignored build roots.
Run the native experiment first. Optional physical target smoke checks use the
existing verified-SD procedure; no Android decryption result is expected yet.
Update the live course and conceptual command/PQC diagrams to remove receipts
without touching protected root SoT figures or archived G2.1 history.
Rollback uses the retained accepted G2.1 image; default platform builds remain
independent. Publish only reviewed sanitized verification, keeping G2.3/G2 open.
