# @file
# @brief Diagnostic-only Ethernet discovery and provisioned public-key SSH access.
SUMMARY = "QuantumTooth physical diagnostic access policy"
LICENSE = "MIT"
LIC_FILES_CHKSUM = "file://${COMMON_LICENSE_DIR}/MIT;md5=0835ade698e0bcf8506ecda2f7b4f302"

# Public authorization input is supplied locally, outside tracked metadata.
FILESEXTRAPATHS:prepend := "${THISDIR}/../../../../local/ssh:"
SRC_URI = "file://20-qtooth-wired.network file://10-qtooth-diagnostic.conf file://authorized_keys"

inherit allarch

RDEPENDS:${PN} = "openssh-sshd systemd-networkd"

## @brief Reject missing authorization, private material and malformed SSH wire data.
## @return Success only for plain Ed25519 public-key entries; comments are ignored.
python do_validate_operator_key() {
    import base64
    from pathlib import Path
    path = Path(d.getVar("UNPACKDIR")) / "authorized_keys"
    try:
        entries = [line.split() for line in path.read_text(encoding="ascii").splitlines()
                   if line.strip() and not line.lstrip().startswith("#")]
        if not entries:
            raise ValueError("no public authorization entries")
        prefix = b"\x00\x00\x00\x0bssh-ed25519\x00\x00\x00\x20"
        for entry in entries:
            if len(entry) < 2 or entry[0] != "ssh-ed25519":
                raise ValueError("expected plain Ed25519 public-key entries only")
            wire = base64.b64decode(entry[1], validate=True)
            if len(wire) != len(prefix) + 32 or not wire.startswith(prefix):
                raise ValueError("invalid Ed25519 public-key wire format")
    except (OSError, UnicodeError, ValueError) as error:
        bb.fatal("Provision a verified public key in ignored local/ssh/authorized_keys: %s" % error)
}
addtask validate_operator_key after do_unpack before do_configure

## @brief Install Ethernet policy and a root-owned SSH public key/configuration.
## @return Zero on successful installation, otherwise a failing install status.
do_install() {
    install -d ${D}${sysconfdir}/systemd/network ${D}${sysconfdir}/ssh/sshd_config.d
    install -m 0644 ${UNPACKDIR}/20-qtooth-wired.network ${D}${sysconfdir}/systemd/network/
    install -m 0644 ${UNPACKDIR}/10-qtooth-diagnostic.conf ${D}${sysconfdir}/ssh/sshd_config.d/
    install -m 0600 ${UNPACKDIR}/authorized_keys ${D}${sysconfdir}/ssh/qtooth_authorized_keys
}

FILES:${PN} = "${sysconfdir}/systemd/network/20-qtooth-wired.network \
              ${sysconfdir}/ssh/sshd_config.d/10-qtooth-diagnostic.conf \
              ${sysconfdir}/ssh/qtooth_authorized_keys"
