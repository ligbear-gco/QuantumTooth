# @file
# @brief Apply the explicitly selected QEMU non-RT inspection fragment.
FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"
SRC_URI:append:qemuarm64 = " ${@'file://nonrt.cfg' if d.getVar('QTOOTH_KERNEL_PROFILE') == 'nonrt' else ''}"
# The upstream printk feature overrides TTY_PRINTK after local fragments.
KERNEL_FEATURES:remove:qemuarm64 = "${@'features/debug/printk.scc' if d.getVar('QTOOTH_KERNEL_PROFILE') == 'nonrt' else ''}"
