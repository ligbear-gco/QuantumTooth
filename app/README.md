# QuantumTooth Android proof app

One foreground portrait React Native screen, a Kotlin GATT TurboModule and an
independent Rust JNI workspace. Kotlin copies/serializes BLE callbacks; Rust
reassembles and authenticates the existing version-1 protocol. Only successful
`Session::decrypt` emits plaintext. JavaScript never receives pairing material,
handshake objects or raw BLE fragments. An ATT confirmation proves transport
completion; it does not prove tag verification.

Prerequisites: the G2.2 Pi software/protocol and the **same** private pair for both
peers. Learn the callback → native verification → UI path, native lifetime
ownership, and why debug needs an embedded bundle. The supplied physical
demonstration shows verified phone plaintext and retained sequences 0/1.
**G2 is complete under DESIGN BLE-5's demonstration boundary; G3 is next.**
The standalone Pi error and additional physical checks remain documented follow-up work.

The [G2 course](../docs/courses/G2_Architecture_and_Practical_Guide.md) consolidates
architecture, host checks, paired deployment, physical results and capture labs.

## Setup (Linux x86_64)

Run all commands from the repository root. Install Node **22.23.4**, npm
**10.9.9**, Python **3.12+**, Perl 5, GNU make, C/C++ tools, pkg-config,
OpenSSL development headers **>=3.5** (for optional host adapter checks), and
Rustup. Use your distribution/package-manager or vendor installation procedure.
The APK build uses its own OpenSSL and does not require host OpenSSL headers.
For Debian/Ubuntu, the native tools are `build-essential perl python3 curl
unzip pkg-config libssl-dev`. The optional host checks require verifying that
the installed libssl development package is at least 3.5.

Install Android Studio with command-line tools, and select a **JDK 21** runtime
(tested: Android Studio JBR **21.0.8**). Set these to your installations; keep
their real paths in the environment or ignored local files:

```bash
export JAVA_HOME='<jdk-21-installation>'
export ANDROID_HOME='<android-sdk-installation>'
export PATH="$JAVA_HOME/bin:$ANDROID_HOME/cmdline-tools/latest/bin:$ANDROID_HOME/platform-tools:$PATH"
sdkmanager 'platform-tools' 'platforms;android-36' 'build-tools;36.0.0' 'ndk;27.1.12297006' 'cmake;3.22.1'
sdkmanager --licenses
rustup toolchain install 1.98.0 --profile minimal
rustup target add aarch64-linux-android --toolchain 1.98.0
```

Review/accept Android SDK licenses yourself. The project pins React Native
**0.84.1**, React **19.2.3**, TypeScript **5.8.3**, CLI **20.1.0**, Gradle
**8.13**, AGP **8.12.0**, Kotlin **2.1.20**, compile/target SDK **36**, min SDK
**24**, build-tools **36.0.0**, NDK **27.1.12297006**, Rust **1.98.0** and
OpenSSL **3.5.9**. SDK 36-targeted apps can run on the selected Android 17
device; the current physical evidence is recorded in the G2 course.
Gradle 8.13/JDK 21 is a compatible combination; the RN template's Gradle 9.0 fails with its packaged
Foojay 0.5 resolver. No dependency-source patch is required.

Integrity pins live in `package-lock.json` (npm integrity),
`native/Cargo.lock` (crate checksums), `android/gradle/wrapper/gradle-wrapper.properties`
(Gradle distribution SHA-256), and `scripts/build-native.py` (OpenSSL archive
SHA-256). The wrapper JAR comes from the RN 0.84 upstream template; its SHA-256:
`76805e32c009c0cf0dd5d206bddc9fb22ea42e84db904b764f3047de095493f3`.

The existing pair must be at ignored `local/ble/pair.bin`, exactly 64 bytes:
PSK (32), peer identity (16), Pi identity (16). Reuse the established G2.2 pair.
Do not generate a different pair for the phone.
Use the same input when building/provisioning the Pi, following the
[G2 course](../docs/courses/G2_Architecture_and_Practical_Guide.md#private-provisioning-and-image-build).
An older unprovisioned Pi image cannot authenticate. Do not display the input
or publish a provisioned APK: its embedded credentials are extractable.

## Build and inspect

```bash
npm ci --prefix app
npm run --prefix app typecheck
./app/android/gradlew -p app/android :app:assembleDebug
sha256sum app/android/app/build/outputs/apk/debug/app-debug.apk
```

Gradle runs Codegen, the pinned NDK OpenSSL PIC/static build, locked ARM64 Rust
compilation, JS/assets bundling and packaging. Missing/invalid pair input fails
before native compilation; changed contents invalidate Gradle/Cargo outputs.
No separate manual native build is required. The bundled JS loads with
`useDevSupport=false`; Android debug signing/debuggability remain enabled.
`debuggableVariants=[]` makes the React Native plugin package JS for debug.

The source archive downloads from OpenSSL's GitHub release (or reuses the
checksum-verified existing Yocto download cache). Native source/install/cache
and raw logs belong under ignored `local/g2-3-react-native-debug-app/`.
Cargo output stays in ignored `app/native/target/`; Gradle outputs stay in
ignored `app/android/**/build/`. The wrapper defaults its cache into ignored
local storage. Generated APKs and signing keys are ignored. Linux development
tools and the Rust Android target must already be installed as above.

Optional focused adapter check (real host OpenSSL >=3.5; no physical BLE):

```bash
cargo test --locked --manifest-path app/native/Cargo.toml
```

For package inspection, use the installed SDK tools. Keep raw output private:

```bash
unzip -l app/android/app/build/outputs/apk/debug/app-debug.apk
"$ANDROID_HOME/build-tools/36.0.0/apksigner" verify --verbose app/android/app/build/outputs/apk/debug/app-debug.apk
"$ANDROID_HOME/build-tools/36.0.0/zipalign" -c -P 16 4 app/android/app/build/outputs/apk/debug/app-debug.apk
```

Expect `assets/index.android.bundle`, ARM64 runtime/JNI libraries, valid debug
signing and alignment. Inspect every packaged ELF's LOAD alignment (>=`0x4000`)
with NDK `llvm-readelf -l`, and the adapter's dynamic dependencies with `-d`:
no Android system `libcrypto.so`/`libssl.so`. The Rust linker and NDK 27 CMake
configuration explicitly enable 16 KB pages. Package facts and checksum belong
in the [verification record](../openspec/changes/archive/2026-11-25-g2-3-react-native-debug-app/verification.md).

<a id="operator-owned-pixel-experiment--pending-physical-execution"></a>
## Physical experiment and results

Follow [G2 Lab D](../docs/courses/G2_Architecture_and_Practical_Guide.md#7-lab-d--build-the-android-receiver-and-exchange-encrypted-messages)
for Linux/Windows ADB installation, standalone and retained sends, then
[Lab E](../docs/courses/G2_Architecture_and_Practical_Guide.md#8-lab-e--capture-and-interpret-the-exchange)
for private capture and evidence handling. Keep Metro stopped and use the same
private pair on Pi and phone.

The [recorded results](../docs/courses/G2_Architecture_and_Practical_Guide.md#9-recorded-physical-and-software-results)
show verified phone decryption at MTU 517 and retained sequences 0/1. The first
standalone trial decrypts but ends with Pi exit 1; additional physical checks are
listed as follow-up work in the course. These do not block G3.
Only native tag verification produces the UI result;
Pi `delivery=unconfirmed` remains correct because there is no application ACK.

## Primary sources

- [React Native environment](https://reactnative.dev/docs/0.84/set-up-your-environment)
  and [Codegen](https://reactnative.dev/docs/0.84/the-new-architecture/using-codegen)
- [React Native 0.84 template](https://github.com/react-native-community/template/tree/0.84-stable/template)
- [AGP 8.12 compatibility](https://developer.android.com/build/releases/agp-8-12-0-release-notes)
  and [Gradle JVM compatibility](https://docs.gradle.org/current/userguide/compatibility.html)
- [Android GATT transfer](https://developer.android.com/develop/connectivity/bluetooth/ble/transfer-ble-data)
  and [16 KB page sizes](https://developer.android.com/guide/practices/page-sizes)
- [OpenSSL ML-KEM](https://docs.openssl.org/3.5/man7/EVP_KEM-ML-KEM/),
  [Android build notes](https://github.com/openssl/openssl/blob/openssl-3.5.9/NOTES-ANDROID.md),
  and repository-pinned OE OpenSSL source checksum.
