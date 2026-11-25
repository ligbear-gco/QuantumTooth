## Context

See [proposal.md](proposal.md) for motivation and learning outcomes. The existing
Rust core provides peer-role handshake, bounded framing and verified record
decryption. The Pi recipe stages only its current Cargo workspace files, so
adding Android dependencies there could break Yocto's frozen source projection.
The user selected TypeScript/React Native and top-level `app/`; Android Studio
is installed, while exact SDK/NDK/JDK/Node availability remains a preflight fact.

This design spans React Native, Android GATT, JNI and native crypto. It conforms
to DESIGN SCOPE/BASE-1..3/BLE-1..5/G2 and HARDWARE HW-1/HW-3 without further SoT
or protected-figure edits. The existing app design draft supplies the screen
sketch; this change's spec defines acceptance.

## Goals / Non-Goals

**Goals:** keep protocol decisions and secrets native; make a single Gradle build
produce an installable self-contained debug APK; expose genuine verification in
a readable recording; keep Android dependencies out of the Pi build.

**Non-Goals:** production credential storage, cross-platform/iOS support, Expo
hosting, JS crypto, a new wire protocol, an exhaustive test suite, automated ADB
installation/recording or accepting overall G2 from a build alone.

## Decisions

### 1. One top-level app, isolated native workspace

```text
app/
  package.json / package-lock.json   React Native + TypeScript
  src/                              one screen and typed native interface
  android/                          Gradle wrapper, Kotlin module and packaging
  native/                           Rust cdylib, own Cargo workspace/lockfile
components/ble/core/                 existing shared protocol/crypto
local/ble/pair.bin                   existing private pair input (ignored)
```

Use a bare React Native project and a typed Turbo Native Module generated from
a TypeScript interface. Kotlin is confined to Android BLE/lifecycle/permission
integration and JNI calls; TypeScript owns UI and presentation state. The native
crate path-depends on the existing core and builds only `aarch64-linux-android`.
Its independent workspace/lockfile avoids pulling BlueR/Linux IPC into Android
or making the Yocto recipe resolve unstaged Android workspace members.

Alternative: put UI in Kotlin or add a general BLE library plus a separate crypto
bridge. The former contradicts the chosen UI stack; the latter adds another
native dependency while protocol callbacks and JNI still need custom ownership.

### 2. Native event flow and ownership

```text
React Native UI -- typed actions --> Kotlin GATT/native module
      ^                                  | serialized copied bytes + time
      | verified result / safe counters  v
      +---------------------------- Rust JNI adapter --> shared core / libcrypto
```

The module exposes scan/connect/disconnect actions and bounded typed status,
failure and verified-message events. Use one serialized native owner for protocol
events and GATT operations, with generation-checked opaque handles rather than
JavaScript-held pointers. Copy callback bytes immediately. Keep expensive crypto
off UI/JS threads; do not route raw fragments or handshake objects through JS.
Bound queued work and aggregate fragment counters; overload fails closed.

JNI converts checked byte arrays and safe result types, documents ownership and
thread assumptions, contains Rust panics at the boundary, and rejects stale
handles/callbacks. Drop secret-bearing state on teardown; do not claim removal
of every compiler/library copy. Add repository-standard Doxygen tags to written code.

Alternative: orchestrate protocol steps and timers in TypeScript. Rejected
because UI pauses and runtime reloads must not control cryptographic state.

### 3. Foreground GATT client and unchanged protocol

Request Nearby devices permissions, scan by the existing secure service UUID,
offer explicit selection/connect, discover RX/TX and subscribe to TX indications.
Await the CCCD write before ClientHello; send ordinary writes with response and
await each completion. Use existing version-1 kinds/encodings and peer-role
handshake, then reassemble records and release only verified UTF-8 plaintext.
Process copied indication events in order before a later disconnect event so a
completed standalone send can retain its verified display result.

Use observed MTU, defaulting to 23 until known, with the existing value budget
and 2048-byte object bound. Android's requested MTU is not a measurement; neither
a request for 23 nor small chunks proves physical MTU 23. Preserve native
10 s object / 5 s ATT / 120 s absolute session / 2 s cleanup limits. Add a bounded
10 s scan and 30 s local connect/setup/establishment limit; the Pi's earlier
advertising-based deadline remains authoritative and is never refreshed.

Close, link loss, timeout, foreground exit or native module reload/invalidation
destroys active state. A new attempt always requires user action and fresh
handshake state. Alternative: retain background sessions or auto-reconnect.
Excluded to keep ownership and the foreground demonstration simple.

### 4. One screen with a trustworthy success condition

Use the consolidated [screen design](../../../../docs/courses/G2_Architecture_and_Practical_Guide.md#6-g23--minimal-android-proof-app):
near-black background, cyan technical status, green verified-result card, large
plaintext and monospaced counters/hex. Include text/icons, system insets and
scan/connect/disconnect controls; keep the main proof visible without scrolling.
Details disclose a bounded phase log. No decorative animation or simulated mode.

Separate link/session status from the last verified message. Only successful
Rust decryption emits the plaintext result, its authenticated sequence, a short
application-ciphertext preview, complete record byte count and RX fragment count.
Handshake time is ClientHello submission to verified PiFinish, excluding scan
and connection setup. Preserve the explicitly labelled last result after normal
close; clear it on a new attempt. Errors do not display rejected plaintext.

Alternative: show a success badge on indication receipt. Rejected because ATT
confirmation does not establish authentication or decryption.

### 5. Private, self-contained Gradle debug build

Pin the compatible React Native/React/TypeScript/Node/npm, JDK, SDK, NDK, Gradle,
Android plugin, Rust and OpenSSL versions during preflight; verify upstream
compatibility before locking them. Android build prerequisites include the Rust
target and native build tools. Do not silently change Pi/Yocto pins.

Gradle builds pinned/checksummed NDK OpenSSL >=3.5 as PIC static libcrypto, builds
the Rust cdylib with locked dependencies, runs React Native Codegen, bundles JS
and assets, and packages ARM64 native libraries. Check ML-KEM capability during
native initialization; failure becomes a safe app error. Check ELF/APK alignment
for modern Android page sizes and inspect packaged dependencies.

React Native's default debug variant relies on Metro. Explicitly enable bundle
generation for debug and load the embedded bundle with developer-server support
disabled, retaining Android debug signing/debuggability. This avoids needing a
running workstation server during the operator's recording. Alternative: default
Metro-dependent debug or a release pipeline; neither matches this handoff.

Consume the existing 64-byte `local/ble/pair.bin` in native build inputs only.
Missing/invalid input fails before packaging; changes invalidate Gradle/Cargo
outputs without printing bytes. Do not generate/replace a pair automatically.
Ignore node_modules, Gradle/native outputs/caches, SDK local properties, signing
material and provisioning; keep private raw evidence under
`local/g2-3-react-native-debug-app/`. Embedded credentials are extractable, so the
provisioned APK is private. Publish reviewed source and sanitized screen evidence.

### 6. Functional acceptance and operator handoff

From the repository root after toolchain setup:

```bash
npm ci --prefix app
./app/android/gradlew -p app/android :app:assembleDebug
sha256sum app/android/app/build/outputs/apk/debug/app-debug.apk
```

Inspect the bundle, ARM64 JNI/runtime libraries, native dependency/alignment
information and debug signing; record versions, source attribution, APK size/hash
and relative output path. These build/package observations complete G2.3's
implementation boundary, without claiming runtime or physical proof. Use
TypeScript checking and existing focused native checks as appropriate; add no
exhaustive instrumentation/fuzzing/benchmark suite or simulator product mode.

Provide an exact operator-run experiment: personally install with ADB, launch
with Metro stopped, run Pi `sendkeep "Hello, world!"`, then scan/connect before
the Pi timeout. Expect verified exact text. A second `sendkeep` with different
public text advances sequence without another handshake; `closekeep` disconnects
while preserving the labelled last result. A later manual connection uses fresh
keys. Document `send` as a separate fresh-session smoke step after closing kept
state. Include expected errors for denied access/disabled Bluetooth and an optional
question: why does Pi still report `delivery=unconfirmed` while the phone shows text?

Device steps and video/GIF production remain with the operator, not unchecked
implementation blockers. Record them as pending until supplied. Reuse the README
UML as an overview and link actual results; the build does not close overall G2
or unperformed BLE-5 checks.

## Risks / Trade-offs

- Toolchain/JNI/provider incompatibility → establish the ARM64 native build first;
  report blockers instead of changing algorithms or weakening authentication.
- React Native adds runtime/APK weight → accept it for the chosen minimal UI;
  keep one screen, one native module and one target ABI.
- Small-MTU radio timing may exceed existing deadlines → disclose measured
  failures; do not relabel chunk limits as negotiated MTU or extend deadlines silently.
- Shared-core changes could affect Pi packaging → prefer the isolated adapter;
  run existing focused checks only when shared inputs change.
- Build success cannot establish physical decryption → separate implementation
  acceptance, operator smoke observations and overall G2/BLE-5 completion.

## Migration Plan

Add the app alongside existing components; preserve the Pi image and probe/data
interfaces. The operator installs the private debug APK through ADB. Rollback
removes/replaces that APK and retains the accepted Pi image. Update live guides
and sanitized verification with actual build results; preserve archive history,
source-of-truth files and protected figures.
