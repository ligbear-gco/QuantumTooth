## Why

The user has received the HW-1 Raspberry Pi 4 Model B with 4 GB RAM. QEMU prevalidation is complete, but physical G1 remains open; current deployment images have no usable remote login path.

## What Changes

- Add diagnostic-only public-key SSH access and Ethernet DHCP with IPv4 link-local fallback for the direct Windows-to-Pi cable. Discover the selected address through Wireshark; postpone Windows static-address troubleshooting.
- Document WSL build, Windows custom-image flashing/verification, first boot, SSH attribution and physical peripheral checks without using QEMU derivatives or consuming Bluetooth's UART.
- Collect physical HW-4 facts and matched BASE-3 RT/non-RT observations. Keep acceptance pending until the user supplies the physical evidence; distinguish base-image and diagnostic measurements.
- Preserve locked build inputs, previous emulation evidence, base-image access policy and both architecture/hardware authorities.

## Capabilities

### New Capabilities

- `platform-physical-g1`: Physical diagnostic access, deployment runbook and attributable hardware acceptance evidence.

### Modified Capabilities

None. Existing emulation requirements remain unchanged.

## Impact

Affects meta-qtooth-platform diagnostic packaging, physical bring-up documentation and evidence under ignored build roots. Conforms to DESIGN BASE-1..3/G1 and HARDWARE HW-1..4; no SoT revision or G2..5 implementation.

Prerequisites: Linux Docker, Imager/OpenSSH/Wireshark as appropriate to the host,
spare microSD/reader, specified Pi supply and cooling. Physical boot, controller
binding and all four measurement runs are now verified. Supply-current acceptance
uses the operator's explicitly authorized assumption; no numeric reading is invented.

Learning outcomes: trace firmware → kernel → systemd/services; distinguish Ethernet address selection from packet observation and SSH authentication; attribute and compare physical RT/non-RT measurements without claiming worst-case bounds.
