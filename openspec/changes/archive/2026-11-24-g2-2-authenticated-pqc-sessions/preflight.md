# G2.2 preflight — 2026-11-24

Read both root SoTs, both delta specs, proposal/design/tasks and the accepted
[G2.1 evidence](../2026-11-24-g2-1-ble-conn-setup/verification.md).
No conflict with DESIGN SCOPE/BASE-1..3/BLE-1..5 or HARDWARE HW-1/HW-3 found.
No hardware or protected architecture changes are authorized or needed.

Scope: one shared Rust protocol core, distinct authenticated GATT service,
on-demand worker and `send`, `sendkeep`, `closekeep`; preserve `probe`.
One best-effort data record per command, no application receipt/retry/reconnect.
BLE responses/indication confirmations and mutual handshake key confirmation remain.

A transport completion concerns ATT fragments. Key confirmation demonstrates
matching derived keys bound to this handshake. Only receiver tag verification
and released plaintext demonstrate decryption; a successful sender cannot claim it.

Prerequisites: accepted G1/G2.1, native development libraries, pinned Cargo/kas
build environment, OpenSSL >=3.5 and private pair inputs for provisioned builds.
Learning outcomes: trace fragments through authentication/decryption; explain
worker-owned keys/counters; distinguish the three observations above.

G2.1's physical public-token round trip was accepted at MTU 517. Physical MTU 23,
Pixel authenticated decryption, physical fragmentation/timing, handshake cost,
tamper/wrong-peer/reconnect behavior and remaining BLE-5 acceptance stay open
for G2.3; native simulation will not close G2.

## Protected SHA-256 baseline

| Repository path | SHA-256 |
|---|---|
| `openspec/DESIGN.md` | `e442f41dd1227d9333fe626e19a3d47f998b672050ce6e7a6ad8ddf7bc9b2617` |
| `openspec/HARDWARE.md` | `e3bb1eb059b27416ab6a0fccd6d81ba3fda7722f04982571b58d72325130ee3a` |
| `docs/diagrams/platform.puml` | `07eb667b88842e84c0a0bfc103d0a0efb35a4363d22f7e38b4e73d576b5c571e` |
| `docs/diagrams/platform.svg` | `061feb87208a922bd23bea1c0e71c3a9cbf4d92c66b8078575560c3996836a9d` |
| `docs/diagrams/ble.puml` | `88774dfbef5c4cd75cf2a0bbf2175dcd1794d43aefda80c34c1b0d905d7c9693` |
| `docs/diagrams/ble.svg` | `6ee77829aeedebf4f1c284343e86516b3eb36d79eae80cedfc534bd2ffe5df55` |
| `docs/diagrams/hwe-runtime.puml` | `cd610016ff7437c788bb06bc53ba5497fc8587411238e2971043c160f9133b72` |
| `docs/diagrams/hwe-runtime.svg` | `b35f43037fb6ecbbe934af05eed416a400295649c9adfcd2d05789ac5054785f` |
| `docs/diagrams/hwe-training.puml` | `5a7e3068d6ee08f6fb3e1101177921ffac02861b24bea98b46c397c02d07f8d5` |
| `docs/diagrams/hwe-training.svg` | `6cf00d132ac28be476babbafb80100d9fb0d96aa4ffa0bbb989142d760a1f58b` |
| `docs/diagrams/hardware.puml` | `825cb8ec8565a2fc335f5f916c5d13eeb0623fecc703afb4e741419efb39d774` |
| `docs/diagrams/hardware.svg` | `db72190e865e5dea4a88335e84647b12928c74fc575adc909bcbaf78e97f3c7b` |

Private raw capability evidence: ignored `local/g2-2-authenticated-pqc-sessions/`.

## Crypto feasibility (task 1.2)

Native OpenSSL/default provider 3.5.5: real keygen, raw public export/import,
encapsulation and decapsulation pass. Selected OE provider/recipe `openssl`
3.5.9: the same probe compiles with GCC 15.3.0, links ELF64 AArch64 against
system `libcrypto.so.3`, and passes under OE QEMU user emulation. Both report
public=1184, ciphertext=1088, secret=32 bytes and agreement=true. No secrets printed.
This establishes provider/ABI feasibility, not physical Pi execution or timing.

Inspected `openssl` 0.10.82 source: raw provider-key imports, HMAC/HKDF/AEAD
are wrapped, but encapsulation/decapsulation and name-based key generation
lack safe wrappers. A narrow KEM FFI module is needed; no vendored feature.
See [OpenSSL KEM contracts](https://docs.openssl.org/3.5/man3/EVP_PKEY_encapsulate/).
Two failed target invocations used a pruned recipe sysroot and an incomplete
compiler PATH; the retained application sysroot/full toolchain fixed these.
Registry DNS required sandbox network access. No pins or host packages changed.
Private scripts, raw logs and ELF/symbol inspection stay in ignored local/build storage.
