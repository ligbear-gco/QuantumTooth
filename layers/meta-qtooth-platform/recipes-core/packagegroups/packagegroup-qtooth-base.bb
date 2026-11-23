# @file
# @brief Required shared runtime; feature applications belong in their own layers.
SUMMARY = "QuantumTooth base runtime"
LICENSE = "MIT"
PACKAGE_ARCH = "${MACHINE_ARCH}"
inherit packagegroup
RDEPENDS:${PN} = "busybox systemd dbus bluez5 glibc libcrypto"
# Required modular drivers and board-revision firmware are explicit image inputs.
RDEPENDS:${PN}:append:raspberrypi4-64 = " \
    kernel-module-hci-uart \
    kernel-module-cmac \
    kernel-module-i2c-bcm2835 \
    kernel-module-i2c-dev \
    kernel-module-spi-bcm2835 \
    qtooth-platform-buses \
    bluez-firmware-rpidistro-bcm4345c0-hcd \
    bluez-firmware-rpidistro-bcm4345c5-hcd \
"
