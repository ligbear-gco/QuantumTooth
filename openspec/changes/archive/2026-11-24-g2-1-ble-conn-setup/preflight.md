# G2.1 implementation preflight — 2026-11-24

Tasks 1.1–1.2: baseline and API inspection complete; compilation and physical
acceptance remain pending. The operator subsequently selected assistant implementation and the existing
`components/ble/` directory for the crate.

## Baseline and acceptance boundary

Read both root SoTs, this change's artifacts and
[G1 acceptance](../2026-11-23-close-pi4-physical-g1/verification.md).
Protected SoT/diagram SHA-256 hashes and raw host checks are retained under
ignored `local/g2-1-ble-conn-setup/`. No protected assets were edited.

Accepted G1 observations: Pi 4 Model B Rev 1.5, 4 GB; kernel 6.18.33-v8;
BlueZ 5.86, D-Bus 1.16.2; controller firmware and LE capabilities initialized.
These are historical evidence, not a fresh target inspection or phone transfer.
G1's supply-current assumption remains as recorded in its acceptance report.

- G2.1: generic foreground Pixel inspector and fixed-token GATT round trip.
- G2.2: bounded framing, authenticated KEM and AEAD protocol.
- G2.3: custom phone adapter, physical decryption and BLE-5 evidence; then G2.

BLE-2/4/5 remain pending. Prerequisites: G1 build environment, diagnostic Pi
shell access and the HW-1 Pixel with a foreground BLE inspector. Learning goals:
trace Rust → D-Bus → BlueZ → controller → phone; distinguish ATT write acceptance
from indication confirmation; attribute the eventual binary to its image.

## Selected stack and build mechanics

The OE checkout matches `kas/base.lock.yml` at
`ae5a301091f5b0574ea7444aab40b309b6d9f3f2`. Inspected:

- `upstream/openembedded-core/meta/recipes-devtools/rust/{rust,cargo}_1.94.1.bb`:
  target build uses Rust/Cargo 1.94.1, independently of the host toolchain.
- `meta/classes-recipe/cargo.bbclass`: `--frozen` forbids dependency resolution
  changes/network during compilation. `cargo_common.bbclass` redirects Cargo
  to BitBake-fetched sources and configures cross-compilation/pkg-config.
- `meta/classes-recipe/cargo-update-recipe-crates.bbclass`: generates
  `crate://` inputs and named SHA-256 checksums from Cargo.lock.

[BlueR 0.17.4 metadata](https://docs.rs/crate/bluer/0.17.4/source/Cargo.toml)
declares Rust 1.75 minimum; select its `bluetoothd` feature for local GATT and
the D-Bus bindings. No API-level version conflict with OE 1.94.1 was found.
Transitive dependency/toolchain compatibility still needs a locked build.

[Write requests](https://docs.rs/bluer/0.17.4/bluer/gatt/local/struct.CharacteristicWriteRequest.html)
expose exchanged MTU, offset, operation type and prepare-authorization status.
[Indication settings](https://docs.rs/bluer/0.17.4/bluer/gatt/local/struct.CharacteristicNotify.html)
must be `indicate=true, notify=false` for confirmations.
[Notifier source](https://docs.rs/bluer/0.17.4/src/bluer/gatt/local.rs.html#520-581)
checks confirmation mode and waits for confirmation in `notify().await`, returning
an error if its confirmation channel closes. Therefore a successful queue handoff
alone cannot establish exchange success. Return from the write callback before
awaiting the indication in a separate owned task.

Rust owns probe validation/lifetimes; the system D-Bus carries method calls;
BlueZ owns GATT/controller operations. The recipe needs `dbus` and `pkgconfig`
for the native binding build, with system libdbus linkage and no vendored feature.
Crypto/OpenSSL additions are outside G2.1.

## Host prerequisite and next increment

Initial preflight observed Rust/Cargo 1.98.0 and pkg-config available;
`pkg-config --modversion dbus-1` fails because development metadata is absent.
The existing ARM64 target sysroot is not a replacement for native host libraries.
For Debian/Ubuntu, install `build-essential pkg-config libdbus-1-dev` before
native Cargo checks. Other distributions need their corresponding development
packages. No host package was installed during preflight.

On implementation resumption, `pkg-config --modversion dbus-1` reports 1.16.2;
the host prerequisite is satisfied. The assistant did not install host packages.

Begin task 2.1 with a standalone crate, a clap `probe` subcommand and locked
clap/Tokio/BlueR dependencies. Verify help without opening D-Bus, then
`cargo check --locked --manifest-path components/ble/Cargo.toml` after satisfying
the host prerequisite. At initial preflight no Rust application, lockfile,
recipe, image or radio result existed. Subsequent implementation results belong
in `verification.md`; API inspection alone is not compilation/connectivity evidence.

Physical validation subsequently exposed a missed compatibility assumption:
the pinned BlueZ server's `WriteValue` callback omits operation `type`, while
BlueR 0.17.4 defaults its absence to `Command`. The existing validator therefore
rejects ordinary Requests. Request-MTU/indication API availability remains valid,
but strict write-operation validation needs the feature-local BlueZ correction
described in `design.md`; ambiguous Command values remain rejected. See the
implementation and build/physical status in `verification.md`.
