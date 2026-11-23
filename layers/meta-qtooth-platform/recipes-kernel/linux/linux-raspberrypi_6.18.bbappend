# @file
# @brief Inspectable Pi profiles without altering the BSP source or physical UART.
FILESEXTRAPATHS:prepend := "${THISDIR}/${PN}:"
SRC_URI:append:raspberrypi4-64 = " file://inspect.cfg file://${QTOOTH_KERNEL_PROFILE}.cfg"

## @brief Deploy final configuration and audit evidence through kernel sstate.
## @return Nonzero if a required build artifact cannot be preserved.
do_deploy:append:raspberrypi4-64() {
    install -m 0644 ${B}/.config ${DEPLOYDIR}/qtooth-kernel-${KERNEL_IMAGE_NAME}.config
    install -m 0644 ${B}/include/config/kernel.release ${DEPLOYDIR}/qtooth-kernel-${KERNEL_IMAGE_NAME}.release
    ln -sf qtooth-kernel-${KERNEL_IMAGE_NAME}.config ${DEPLOYDIR}/qtooth-kernel.config
    ln -sf qtooth-kernel-${KERNEL_IMAGE_NAME}.release ${DEPLOYDIR}/qtooth-kernel.release
    install -d ${DEPLOYDIR}/qtooth-kernel-audit
    for qtooth_audit in ${S}/.kernel-meta/cfg/*.txt ${T}/log.do_kernel_config*; do
        if [ -f "$qtooth_audit" ]; then
            install -m 0644 "$qtooth_audit" ${DEPLOYDIR}/qtooth-kernel-audit/
        fi
    done
    git -C ${S} rev-parse HEAD > ${DEPLOYDIR}/qtooth-kernel-source.txt
    git -C ${S} status --short >> ${DEPLOYDIR}/qtooth-kernel-source.txt
}
