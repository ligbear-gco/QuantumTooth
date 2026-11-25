## 1. Learn and establish build prerequisites

- [x] 1.1 Re-read both root SoTs, the app spec/design and G2.2 registry; explain the UI → native BLE → Rust boundary and record a preflight showing conformance and preserved Pi/protected assets.
- [x] 1.2 Inspect installed Node/npm, JDK, SDK/NDK and Rust Android support, verify upstream compatibility and pin the selected versions/checksums; verify the documented setup resolves every prerequisite without workstation-specific tracked paths or changing Pi pins.

## 2. Build the isolated Android native core

- [x] 2.1 Create `app/native/` as an independent Cargo workspace/cdylib with a locked path dependency on `components/ble/core/`; inspect Cargo metadata to confirm the Android build excludes BlueR/Linux IPC and the existing Pi workspace/recipe remains resolvable.
- [x] 2.2 Add reproducible NDK OpenSSL >=3.5 PIC/static build integration and Rust cross-compilation; verify the ARM64 library links the required KEM/AEAD functions, has no Android system-libcrypto dependency and meets native alignment requirements.
- [x] 2.3 Consume existing ignored `local/ble/pair.bin` natively, enforce size/presence and rebuild tracking, and ignore generated/provisioned/signing outputs; verify missing/invalid input fails and disposable private input changes invalidate affected outputs without printing bytes or replacing the established pair.
- [x] 2.4 Implement the JNI owner, framing/peer handshake/record flow, deadlines and sanitized result/error boundary with Doxygen ownership contracts; verify Android compilation, reviewed teardown/stale-handle paths and that only successful core decryption produces a plaintext result. Run existing focused core checks if shared inputs change.

## 3. Build the foreground BLE/native module

- [x] 3.1 Scaffold the React Native/TypeScript project in `app/`, its Gradle project in `app/android/`, lock dependencies and define the typed native module; verify `npm ci --prefix app`, TypeScript checking and native Codegen succeed.
- [x] 3.2 Implement Nearby devices permissions, bounded service-filtered scan/selection/connect, MTU observation, RX/TX discovery, CCCD subscription and serialized writes/indications; verify generated Android code compiles and the callback flow preserves subscription-before-ClientHello and per-operation completion/bounds.
- [x] 3.3 Integrate native failure/close/expiry, foreground exit and React Native invalidation cleanup, explicit retry controls and safe user errors; verify the compiled module exposes these paths with bounded queues, generation checks and no automatic reconnect or application ACK.

## 4. Build and inspect the visual proof screen

- [x] 4.1 Implement the single portrait dark/cyan/green screen with controls, phase status, ciphertext preview and prominent verified text; verify TypeScript checks pass and layout review against Pixel dimensions/insets keeps the primary result visible without scrolling.
- [x] 4.2 Connect native verified-result/counter events to the screen, preserve the labelled last result after normal closure and clear it for a new attempt; inspect the event paths to verify no hardcoded plaintext/simulated success, stale-session success or unverified record reaches the result, and diagnostics have defined scopes/bounds.

## 5. Build and inspect the standalone debug APK

- [x] 5.1 Wire native build tasks, Codegen, JavaScript/assets bundling and embedded-bundle loading into `:app:assembleDebug`, retaining Android debug signing/debuggability with Metro support disabled; verify task dependencies package the current bundle and native outputs without a separate manual native build.
- [x] 5.2 Run `npm ci --prefix app`, the project's TypeScript check and `./app/android/gradlew -p app/android :app:assembleDebug`; inspect `app/android/app/build/outputs/apk/debug/app-debug.apk` for ARM64 libraries, JS/assets, debug signing, dependency/alignment correctness, size and SHA-256. Save raw evidence privately and report only observed build facts; no exhaustive new test suite.

## 6. Explain and hand off the functional experiment

- [x] 6.1 Document exact setup/build/output paths and operator-owned ADB installation, Metro-stopped launch, standalone send, two retained sends, close and fresh manual-session steps; inspect the runbook for expected plaintext/sequence/closure observations, safe errors and an optional ATT-confirmation-versus-decryption question.
- [x] 6.2 Reconcile live app design/course and README evidence links with top-level `app/` and actual build status, reusing the existing simple UML; verify local links, consistent pending physical labels and no root SoT/protected-figure/archive edits.
- [x] 6.3 Write a sanitized verification summary with source/tool attribution and the APK checksum/path, explain the native ownership and embedded-bundle lessons, and list operator Pixel/recording and remaining BLE-5 work as pending follow-up; verify privacy review, `git diff --check` and strict OpenSpec validation. Mark G2.3 implementation complete only after build/package acceptance, while leaving overall G2 open.

The operator personally installs/runs/records the APK. Delivering its functional
runbook is required here; performing those device steps is subsequent work and
does not block the agreed G2.3 build acceptance boundary.
