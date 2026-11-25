# G2.3 implementation verification — 2026-11-25

**G2.3 build/package acceptance is met. Overall G2 remains open.**
The app is implemented in top-level `app/`; no Pixel installation, app launch,
physical decryption or screen recording was performed. The operator owns those
steps in the [runbook](../../../../app/README.md#operator-owned-pixel-experiment--pending-physical-execution).

## Observed build/package evidence

Executed from the repository root after setup:

```bash
npm ci --prefix app
npm run --prefix app typecheck
./app/android/gradlew -p app/android :app:assembleDebug
sha256sum app/android/app/build/outputs/apk/debug/app-debug.apk
```

Clean npm installation and TypeScript checking pass. Gradle completes Codegen,
CMake/Kotlin/Java compilation, the integrated native task, JS/assets bundling,
debug signing and APK assembly. The first full successful build executed all
79 tasks; subsequent source changes were rebuilt and the final acceptance run
succeeded. Generated `NativeQToothSpec` and linked `QToothSpec_ModuleProvider`
match the typed module. No separate manual native step is needed.

| Artifact fact | Observed value |
|---|---|
| APK | `app/android/app/build/outputs/apk/debug/app-debug.apk` |
| APK bytes | 47713071 |
| APK SHA-256 | `ee14721a27a17035816dff0bb8b0cbd5c2792b076d22d6ffc0d39b7946b8301c` |
| ABI | `arm64-v8a` only |
| Embedded `assets/index.android.bundle` bytes | 1125852 |
| Signing/debuggability | `apksigner` verification passes; Android Debug signer; manifest debuggable=true |
| Native packaging | 12 ARM64 libraries; `zipalign -c -P 16 4` passes |

Inspected every packaged ELF using NDK `llvm-readelf -h -l -d`. All LOAD
segments are at least 16 KB aligned. Uncompressed library ZIP offsets are
16 KB aligned. All NEEDED entries resolve to another packaged library or an
Android system library; none depends on system libcrypto/libssl.
The adapter's only dynamic dependencies are `libdl.so` and `libc.so`.
The linked, unstripped adapter defines `EVP_PKEY_encapsulate`,
`EVP_PKEY_decapsulate` and `EVP_aes_256_gcm`; the packaged crypto is static
OpenSSL 3.5.9. Capability is checked again on native initialization.

| Packaged library | Bytes | Minimum LOAD alignment |
|---|---:|---:|
| `libappmodules.so` | 875584 | `0x4000` |
| `libc++_shared.so` | 1292904 | `0x4000` |
| `libfbjni.so` | 177000 | `0x4000` |
| `libhermestooling.so` | 555704 | `0x4000` |
| `libhermesvm.so` | 3699496 | `0x4000` |
| `libimagepipeline.so` | 8760 | `0x4000` |
| `libjsi.so` | 923712 | `0x4000` |
| `libnative-filters.so` | 23712 | `0x4000` |
| `libnative-imagetranscoder.so` | 585376 | `0x4000` |
| `libreact_codegen_safeareacontext.so` | 563392 | `0x4000` |
| `libreactnative.so` | 19959280 | `0x4000` |
| `libqtooth_android.so` | 6081936 | `0x4000` |

## Native ownership, protocol and provisioning checks

- Independent `app/native/Cargo.lock` resolves only the adapter and shared core
  as local packages. Full locked metadata contains no BlueR, D-Bus or Tokio.
  The unchanged Pi workspace still resolves with locked metadata; its recipe,
  Cargo inputs, kas pins and protected baseline all remain unchanged.
- Two focused adapter tests pass using actual OpenSSL and the production
  peer-role adapter: fragmented establishment and two verified records at MTUs
  23/517, authenticated sequences 0/1, record-specific lengths/counts, close,
  absolute expiry and rejection of a tampered tag. This is host evidence, not
  Android runtime/radio evidence. Formatting and Clippy (warnings denied) pass.
  Shared core source did not change, so the prior Pi/core suite was not rerun.
- The user explicitly authorized reuse of the existing G2.2 disposable pair.
  It was copied into ignored `local/ble/pair.bin`; 64-byte size and equality
  were checked without publishing bytes. The later Pi experiment must use that
  same input; no provisioned Pi deployment is asserted here.
- Missing input and a disposable 63-byte input fail before native compilation.
  Two different disposable 64-byte contents trigger Cargo recompilation and
  different ARM64 library hashes. The selected pair is never overwritten; the
  library is rebuilt with that input afterward. Gradle tracks its contents and
  forces the expected path; Cargo tracks path/environment/content changes.
- JNI copies checked arrays (max 512 bytes), contains panics, uses opaque
  monotonically increasing generations and drops active state on failure/close.
  Poisoned-registry cleanup still drops owned secrets; stale destroy calls
  cannot discard a newer owner. Secrets never enter JS or diagnostic events.
- Reviewed compiled Kotlin paths: Nearby devices gate, max eight scan peers,
  10 s filtered scan, 30 s setup, CCCD completion before ClientHello, one ordinary
  write-with-response at a time, copied/ordered indication callbacks, max 128
  queued output fragments and 256 queued work items. Overload fails closed.
  Each outgoing object gets its own absolute 10 s deadline; ATT gets 5 s.
  Rust enforces incoming-object 10 s and immutable session 120 s deadlines;
  a native-worker 100 ms watchdog runs independently of JS. Callback paths
  recheck deadlines before success. Local teardown invalidates/drops ownership
  and calls disconnect/close directly without awaiting a remote response;
  it adds no wait beyond the 2 s cleanup allowance. This is reviewed code,
  not a measured physical cleanup-latency guarantee.
- Foreground exit, expiry, link failure and native module invalidation clear
  active state; no automatic reconnect, retry or application ACK exists.
  Both Kotlin and JS reject stale/currently-backgrounded success events.
  Only `Session::decrypt` produces the native verified plaintext event.

The screen review used a 411 × 914 dp portrait reference with system-inset
space reserved, default font sizing, bounded status/ciphertext and a four-line
result. The primary result and controls require no scrolling; long verified
messages have a complete Details view. This is source/layout review, not an
observed Pixel screenshot. Last verified text remains explicitly labelled after
normal closure and clears at a new attempt. Diagnostics use actual record
metadata, a <=24-byte ciphertext excerpt and an eight-entry phase history;
unmeasured values stay unavailable. The app contains no preset success string.

## Tools, sources and corrections

Selected/tested: Node 22.23.4, npm 10.9.9, Android Studio JBR 21.0.8,
SDK/build-tools 36/36.0.0, NDK 27.1.12297006, CMake 3.22.1, Gradle 8.13,
AGP 8.12.0, Kotlin 2.1.20, Rust 1.98.0, React Native 0.84.1, React 19.2.3,
TypeScript 5.8.3 and OpenSSL 3.5.9. npm/Cargo lockfiles fix dependency checksums;
Gradle/OpenSSL source checksums and setup are in the
[runbook](../../../../app/README.md#setup-linux-x86_64).

Sources: repository G2.2 core/[wire registry](../2026-11-24-g2-2-authenticated-pqc-sessions/registry.md);
[RN upstream template](https://github.com/react-native-community/template/tree/0.84-stable/template)
for wrapper/host conventions; [RN Codegen](https://reactnative.dev/docs/0.84/the-new-architecture/using-codegen);
[AGP compatibility](https://developer.android.com/build/releases/agp-8-12-0-release-notes);
[Android alignment](https://developer.android.com/guide/practices/page-sizes);
[OpenSSL Android build](https://github.com/openssl/openssl/blob/openssl-3.5.9/NOTES-ANDROID.md).
The archive checksum matches the existing pinned OE recipe, without changing it.

Initial sandbox DNS restrictions required network-enabled dependency downloads.
The RN template's Gradle 9.0 failed against bundled Foojay 0.5 (`IBM_SEMERU`);
AGP-supported Gradle 8.13 fixes that without editing dependency sources.
TypeScript's inherited Jest type requirement was overridden with the actual
React/native types. JNI poison cleanup and overlapping outgoing-object deadline
handling were corrected during review and rebuilt. Upstream deprecation/SDK
XML warnings remain; the final clean npm audit reports eight low-severity
advisories. No broad dependency upgrades were folded into this change.

## Handoff, privacy and limits

Learning: Kotlin transports copied values; Rust owns authentication/state;
the UI displays an authenticated result. Discarding a native owner destroys
its owned key/counter state, without claiming to erase every compiler/library
copy. The default React Native debug variant omits the JS bundle: empty
`debuggableVariants` enables bundling, and `useDevSupport=false` selects
embedded-only loading.

The [operator experiment](../../../../app/README.md#operator-owned-pixel-experiment--pending-physical-execution)
covers Metro-stopped launch, standalone send, retained sequences 0/1, close,
labelled-result retention and a fresh manual session. Optional explain-back:
ATT confirmation can coexist with `delivery=unconfirmed` because no application
ACK reports the phone's successful decryption to the Pi.

Pending: operator Pixel install/launch/decryption and video/GIF, physical
small/large-MTU fragmentation/timing, handshake cost, wrong-peer/tamper rejection,
reconnect cleanup and remaining BLE-5/G2 evidence. A chunk cap or host MTU
simulation is not a measured small negotiated radio MTU. No hard real-time,
independent cryptographic audit or physical acceptance is claimed.

Private APK, native outputs, signing/caches and raw logs remain ignored; raw
inspection/signing output is under `local/g2-3-react-native-debug-app/`.
Published source/report review excludes pair bytes, identities, addresses,
credentials, workstation paths and real fingerprints. All 89 preflight Pi/tool/
protected file hashes match; pre-existing staged edits were preserved. Root
SoTs, protected figures and archives were not edited. Local documentation links,
`git diff --check` and strict OpenSpec validation are checked at completion.

## Later physical evidence

This record preserves the original build/package boundary. The subsequently
supplied capture and GIF now demonstrate phone decryption and retained sequences
0/1; the standalone Pi failure and remaining physical gates are documented in
the [consolidated G2 course](../../../../docs/courses/G2_Architecture_and_Practical_Guide.md#9-recorded-physical-and-software-results).
