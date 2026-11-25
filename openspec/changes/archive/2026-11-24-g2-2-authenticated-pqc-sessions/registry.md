# Version-1 wire registry

Frozen before adapter integration; integers unsigned big endian. ASCII labels
have no NUL terminator. `D = QTOOTH/G2.2/v1` (14 bytes).

| Item | Exact encoding |
|---|---|
| Service / RX / TX | `b6632000-7e7b-4fb5-9dc4-93f4152a0e7a` / `b6632001-7e7b-4fb5-9dc4-93f4152a0e7a` / `b6632002-7e7b-4fb5-9dc4-93f4152a0e7a` |
| Fragment header | version:u8=1, kind:u8, object-id:u16, total:u16, offset:u16 |
| Kinds | 1 ClientHello, 2 PiOffer, 3 PeerFinish, 4 PiFinish, 5 EncryptedRecord |
| Object IDs | Start 0 independently per direction; no wrap |
| `E(k,b)` | k:u8, len(b):u16, b |
| ClientHello | version:u8=1, suite:u16=1, role:u8=1, peer-id:16, Pi-id:16, peer-nonce:32 (68 bytes) |
| OfferCore | role:u8=2, Pi-nonce:32, raw ML-KEM-768 public-key:1184 (1217 bytes) |
| PiOffer | OfferCore, HMAC(PSK, D || `pi-offer` || E(1,ClientHello) || E(2,OfferCore)) (1249 bytes) |
| ResponseCore | raw ML-KEM-768 ciphertext:1088 |
| T | D || E(1,ClientHello) || E(2,OfferCore) || E(3,ResponseCore) (2396 bytes) |
| PeerFinish | ResponseCore, HMAC(PSK, `peer-auth` || T), peer-confirmation (1152 bytes) |
| HKDF | SHA-256; Extract salt=SHA-256(T), IKM=32-byte KEM secret; Expand info below |
| peer-confirmation | HMAC(peer-confirm-key, `peer-finished` || SHA-256(T)) |
| PiFinish | HMAC(pi-confirm-key, `pi-finished` || SHA-256(T) || peer-confirmation) (32 bytes) |
| Record AAD | version:u8=1, type:u8, direction:u8, session-id:16, sequence:u64, plaintext-length:u16 (29 bytes) |
| Record body | AAD, ciphertext:plaintext-length, AES-256-GCM tag:16 |
| Record types/directions | data=1, close=2; Pi-to-peer=1, peer-to-Pi=2 |
| Nonce | direction's 4-byte prefix || sequence:u64; sequences start 0, reserve once; exhaustion aborts |

Exact HKDF info strings (each expansion uses the extracted PRK independently):

| Purpose | ASCII info | Bytes |
|---|---|---|
| Pi-to-peer key | `QTOOTH/G2.2/v1/pi-to-peer/key` | 32 |
| Peer-to-Pi key | `QTOOTH/G2.2/v1/peer-to-pi/key` | 32 |
| Pi-to-peer nonce prefix | `QTOOTH/G2.2/v1/pi-to-peer/nonce` | 4 |
| Peer-to-Pi nonce prefix | `QTOOTH/G2.2/v1/peer-to-pi/nonce` | 4 |
| Peer confirmation key | `QTOOTH/G2.2/v1/peer-confirm` | 32 |
| Pi confirmation key | `QTOOTH/G2.2/v1/pi-confirm` | 32 |
| Session identity | `QTOOTH/G2.2/v1/session-id` | 16 |

No unknown/trailing fields. Fragment payload is nonempty; value budget is
`min(MTU-3,512)`; payload budget subtracts the 8-byte header. One contiguous
incomplete object per direction, fixed storage capped at 2048 bytes. Data is
1..512 UTF-8 bytes; close has zero plaintext. Largest object=1249; largest
record=557, both below 2048. Object state and type-specific lengths are checked
before copying; failure invalidates the session.

| Object | Bytes | Fragments MTU 23 / 517 |
|---|---:|---:|
| ClientHello | 68 | 6 / 1 |
| PiOffer | 1249 | 105 / 3 |
| PeerFinish | 1152 | 96 / 3 |
| PiFinish | 32 | 3 / 1 |
| `Hello, world!` data | 58 | 5 / 1 |
| Maximum data | 557 | 47 / 2 |
| Close | 45 | 4 / 1 |

Even the short demo requires fragmentation for its handshake and its protected
record at MTU 23. ATT values are not radio packets. Data/close have no
application reply or retry; PiFinish transport completion precedes Pi data.

Monotonic origins: establishment=advertising start (30 s); lifetime=handshake
completion (120 s absolute); object=first fragment/send start (10 s, never
refreshed); ATT=operation start (5 s); startup/IPC framing/writes=start (5 s);
courtesy close=start (2 s); cleanup=protocol stop (2 s). Use the earliest
applicable enclosing deadline, including lifetime during close and active sends.

[fixtures.json](fixtures.json) contains public deterministic encoding fixtures,
independently generated with Python: identities 00..0f / 10..1f, peer nonce
20..3f, Pi nonce 40..5f, synthetic public key a5 repeated 1184 times and
ciphertext 5a repeated 1088 times. These bytes test encoding only, never KEM
validity or production provisioning. An initial fixture length assertion counted
the domain as 13 rather than 14 bytes; the independent count corrected it to
2396 before implementation.
