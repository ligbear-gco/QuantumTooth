# @file
# @brief Retain Pi BSP initialization commands while excluding unused BlueZ tools.
PACKAGECONFIG:append:raspberrypi4-64 = " tools deprecated readline"

## @brief Trim Pi tools to the commands referenced by btuart and bthelper.
## @return Nonzero if expected initialization executables were not installed.
do_install:append:raspberrypi4-64() {
    for qtooth_tool in ${D}${bindir}/*; do
        case "$(basename "$qtooth_tool")" in
            hciattach|hciconfig|hcitool|bluetoothctl) ;;
            *) rm -f "$qtooth_tool" ;;
        esac
    done
    rm -f ${D}${systemd_user_unitdir}/mpris-proxy.service
    for qtooth_tool in hciattach hciconfig hcitool bluetoothctl; do
        test -x ${D}${bindir}/$qtooth_tool || bbfatal "Missing Pi Bluetooth initialization command: $qtooth_tool"
    done
}

## @brief Remove the empty tools directory for selections with all tools disabled.
## @return Nonzero for an unexpected directory removal error.
do_install:append() {
    if [ -d "${D}${bindir}" ]; then
        rmdir --ignore-fail-on-non-empty "${D}${bindir}"
    fi
}
