# @file
# @brief Minimal shared image without development console policy.
SUMMARY = "QuantumTooth headless platform image"
LICENSE = "MIT"
inherit core-image
IMAGE_INSTALL = "packagegroup-core-boot packagegroup-qtooth-base"
IMAGE_FEATURES = ""
# Native emulator closure is needed for attributable, unprivileged Pi launch.
DEPENDS:append:raspberrypi4-64 = " qemu-system-native dtc-native e2fsprogs-native kmod-native"
# WIC must wait for the independent boot overlay's deploy task.
do_image_wic[depends] += "${@bb.utils.contains('MACHINE', 'raspberrypi4-64', 'qtooth-platform-buses:do_deploy', '', d)}"
