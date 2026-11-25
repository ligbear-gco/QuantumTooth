# @file
# @brief Build the repository-owned diagnostic with locked crates and system D-Bus/OpenSSL.
SUMMARY = "QuantumTooth authenticated PQC sessions and BLE probe"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://LICENSE;md5=d3493c71e6347f93433c4cf1728bef8b"

FILESEXTRAPATHS:prepend := "${THISDIR}/../../../../components/ble:${THISDIR}/../../../..:"
SRC_URI = "file://Cargo.toml file://Cargo.lock file://build.rs file://core file://src file://LICENSE"
S = "${UNPACKDIR}"

inherit cargo cargo-update-recipe-crates pkgconfig
require qtooth-ble-crates.inc

DEPENDS += "dbus openssl"
RDEPENDS:${PN} += "bluez5 dbus"
COMPATIBLE_MACHINE = "^raspberrypi4-64$"

# @brief Optional operator input is private; BitBake hashes file contents into task signatures.
FILESEXTRAPATHS:prepend := "${THISDIR}/../../../../local/ble:"
QTOOTH_PAIR_INPUT := "${THISDIR}/../../../../local/ble/pair.bin"
QTOOTH_PAIR_PRESENT = "0"

# @brief Track optional-file existence in the parse cache, and contents via SRC_URI checksums.
python __anonymous() {
    path = os.path.normpath(d.getVar('QTOOTH_PAIR_INPUT'))
    bb.parse.mark_dependency(d, path)
    if os.path.isfile(path):
        d.setVar('QTOOTH_PAIR_PRESENT', '1')
        d.appendVar('SRC_URI', ' file://pair.bin')
}
export QTOOTH_PAIR_FILE = "${@d.getVar('UNPACKDIR') + '/pair.bin' if d.getVar('QTOOTH_PAIR_PRESENT') == '1' else ''}"

# @brief Empty env values mean no provisioning, not a path to a built-in/test key.
do_compile:prepend() {
    if [ -z "$QTOOTH_PAIR_FILE" ]; then
        unset QTOOTH_PAIR_FILE
    fi
}
