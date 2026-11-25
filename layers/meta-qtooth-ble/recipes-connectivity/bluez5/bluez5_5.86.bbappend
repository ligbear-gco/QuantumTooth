# @file
# @brief Preserve strict ATT operation validation only in BLE feature builds.
FILESEXTRAPATHS:prepend := "${THISDIR}/files:"
SRC_URI:append = " file://0001-gatt-preserve-characteristic-write-type.patch"
PR:append = ".qtoothble1"
