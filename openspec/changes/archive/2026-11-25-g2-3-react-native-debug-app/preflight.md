# G2.3 implementation preflight — 2026-11-25

Read both root SoTs, this change's proposal/design/spec/tasks, the archived
[G2.2 registry](../2026-11-24-g2-2-authenticated-pqc-sessions/registry.md),
its preflight/verification, the shared core interfaces and the Pi Cargo recipe.
No conflict found with DESIGN SCOPE/BASE-1..3/BLE-1..5/G2 or HARDWARE HW-1/HW-3.
Existing staged edits predate this implementation; they are preserved.
The baseline below records the current, already revised root design.

## Boundary and learning outcomes

React Native/TypeScript owns presentation and typed actions. Kotlin owns Android
permissions, foreground lifecycle and serialized GATT callbacks. Rust owns
bounded fragment reassembly, peer-role authentication, session deadlines and
AEAD verification. JNI is the checked language boundary between Kotlin and Rust.
Only successful core decryption may produce a verified UI result; ATT indication
confirmation establishes transport completion, not decryption.

Learn to trace that callback/data path, distinguish transport confirmation from
verified plaintext, and explain why standalone debug loading requires an embedded
JavaScript bundle. Prerequisites are the G2.2 protocol/core, compatible Android
and Rust build tools, and the same private pair used by the Pi. The Android
workspace must remain independent so Yocto's frozen Cargo source projection and
Pi system-libcrypto linkage remain unchanged. No Pi, FPGA, hardware, archive or
protected-figure edits are needed.

## Initial prerequisite observations

- Node **22.23.4**, npm **10.9.9**, Rust **1.98.0** are installed.
- Default JDK is **25.0.5.1**; JDK 21 and Android Studio's **21.0.8** runtime are
  also present. A compatible build JDK has not yet been selected/validated.
- Android platforms **35**, **36**, **37.0**; build-tools **36.0.0**, **36.1.0**,
  **37.0.0**; NDK **27.1.12297006**, **30.0.16248370** are present.
- Rust has only its host target installed; Android target setup is pending.
- Required `local/ble/pair.bin` was initially absent. The ignored local tree
  contained one existing 64-byte G2.2 disposable pair. The user explicitly
  authorized reusing it; it was copied into `local/ble/pair.bin` with restricted
  permissions and equality verified without printing bytes. No new pair was
  generated. The later Pi experiment must provision this same input.

These initial inventory observations do not establish Android build acceptance.
Selected pins and subsequent evidence belong in the app runbook and verification
report. No Pixel run or physical BLE-5 result is claimed; overall G2 stays open. Raw baseline evidence is ignored under
`local/g2-3-react-native-debug-app/`.

Upstream setup reference: [React Native 0.84 environment](https://reactnative.dev/docs/0.84/set-up-your-environment).
Native packaging reference: [Android page-size requirements](https://developer.android.com/guide/practices/page-sizes).
These references inform later compatibility checks; installed tools alone do not
prove a compatible build.

## Protected SHA-256 baseline

| Repository path | SHA-256 |
|---|---|
| `openspec/DESIGN.md` | `cad29ee4579d24e7ec198937b56747a30b6fcd6c4674f0e288ac7fa8a13c3315` |
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

Additionally snapshotted **77** tracked Pi workspace, kas, layer and tool
files. SHA-256 of their sorted repository-relative path/NUL/file-hash/newline
manifest: `13eb51b1970abb29943bd4e4c8e7edb6bbb393d8a898d1a935089a05f0ff3ea7`. No implementation command changed those inputs.

## Hands-on check after restoring prerequisites

Check only the pair's size, without displaying contents:

```bash
test -f local/ble/pair.bin && test "$(wc -c < local/ble/pair.bin)" -eq 64
```

Expected: exit status zero for the existing 64-byte input; this checks format
length, not peer identity or successful authentication. The functional Pixel
experiment will be documented with the implemented app. Optional explain-back:
why can ATT delivery succeed while plaintext verification still fails?
