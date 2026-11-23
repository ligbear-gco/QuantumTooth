# @file
# @brief Deploy the Pi bus overlay and automatic I2C userspace interface policy.
SUMMARY = "QuantumTooth Pi I2C1 and SPI0 activation"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"
SRC_URI = "file://qtooth-platform-buses.dts file://qtooth-platform-buses.conf"
DEPENDS = "dtc-native"
COMPATIBLE_MACHINE = "^raspberrypi4-64$"
PACKAGE_ARCH = "${MACHINE_ARCH}"
S = "${UNPACKDIR}"
B = "${WORKDIR}/build"

inherit deploy

# Required module packages are owned by the Pi shared-runtime package group.

## @brief Compile an independently deployable, symbol-linked DT overlay.
## @return Zero if the pinned native compiler accepts the overlay.
do_compile() {
    ${STAGING_BINDIR_NATIVE}/dtc -@ -I dts -O dtb \
        -o ${B}/qtooth-platform-buses.dtbo ${UNPACKDIR}/qtooth-platform-buses.dts
}

## @brief Install the module-load policy without adding target debug tools.
## @return Zero on successful installation.
do_install() {
    install -d ${D}${sysconfdir}/modules-load.d
    install -m 0644 ${UNPACKDIR}/qtooth-platform-buses.conf \
        ${D}${sysconfdir}/modules-load.d/qtooth-platform-buses.conf
}

## @brief Publish the overlay for the partitioned firmware boot image.
## @return Zero on successful deployment.
do_deploy() {
    install -d ${DEPLOYDIR}
    install -m 0644 ${B}/qtooth-platform-buses.dtbo ${DEPLOYDIR}/
}
addtask deploy after do_compile before do_build

FILES:${PN} = "${sysconfdir}/modules-load.d/qtooth-platform-buses.conf"
