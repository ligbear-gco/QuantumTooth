# platform-physical-g1 Specification

## Purpose

Provide attributable physical Raspberry Pi access and evidence needed to evaluate DESIGN G1 after Pi-specific emulation prevalidation.

## Requirements

### Requirement: Diagnostic public-key access

Physical diagnostic images SHALL provide SSH access using the operator's provisioned public key, with password and keyboard-interactive authentication disabled. Diagnostic root access SHALL permit platform inspection without an empty password or console autologin. Private client keys and pre-generated host private keys MUST NOT be packaged. Base-image access policy and QEMU isolation/access behavior SHALL remain unchanged (BASE-1..2, HW-3).

#### Scenario: Operator logs in
- **WHEN** the diagnostic image boots and the operator connects using the matching private key
- **THEN** SSH provides a root shell, password login is unavailable, and host keys were generated on the target
- **AND** the Bluetooth UART remains allocated to the radio

### Requirement: Operator-neutral provisioning and documentation

Tracked metadata and published documentation SHALL contain no operator key
material, actual host fingerprints, secrets or personal workstation paths.
Diagnostic authorization SHALL be provided through an ignored local input and
validated before installation; the tracked fallback MUST NOT grant access.
Base builds SHALL NOT require operator authorization input. The lab SHALL teach
hash and key verification using commands and placeholders, not live credentials.

#### Scenario: Build without local authorization
- **WHEN** no operator public authorization file has been provisioned
- **THEN** base remains buildable and a diagnostic access build fails with an actionable provisioning error rather than distributing a default identity

### Requirement: Direct Ethernet discovery

Diagnostic Ethernet SHALL request DHCPv4 and support IPv4 link-local address selection when no server answers. The runbook MUST distinguish DHCP discovery packets from an assigned lease, ARP probing from an established address, and packet visibility from bidirectional IP connectivity. Windows static-address repair SHALL NOT block preparation of the image.

#### Scenario: Direct cable without DHCP service
- **WHEN** the Pi and Windows PC are directly connected without a DHCP server
- **THEN** the Pi can select an uncontested 169.254.0.0/16 address and its ARP traffic can be inspected in Wireshark
- **AND** SSH is attempted only after confirming compatible PC addressing and TCP port 22 reachability

### Requirement: Physical deployment and attribution

The project SHALL document WSL build/export and Windows custom-image writing/verification of the partitioned Pi deployment candidate. Emulation DT/storage/login derivatives MUST NOT be flashed. Physical evidence SHALL identify written artifacts, board revision/RAM, firmware/boot path, running configuration/modules and service/peripheral state under G1/HW-4. Unperformed or failing checks MUST remain explicit.

#### Scenario: First hardware inspection
- **WHEN** the learner boots the prepared card and logs in
- **THEN** the learner records `uname -a`, `/proc/1/comm`, `/proc/config.gz`, board identity, network state and service failures against the build inventory
- **AND** Bluetooth initialization and physical I2C/SPI enablement are verified separately from mere package installation

### Requirement: Physical baseline and acceptance boundary

G1 acceptance SHALL require physical BASE-3 size, settled RAM, defined boot-time and repeated idle/load scheduling observations for matched RT/non-RT variants, plus the applicable physical handoff checks. Cooling and the base-image evidence collection path MUST be resolved before dependent measurements; diagnostic observations MUST NOT be presented as base-image measurements. Changes to collection/access policy and their overhead MUST be disclosed. QEMU observations or a successful SSH login alone MUST NOT close G1.

#### Scenario: Evidence remains incomplete
- **WHEN** any required physical observation is missing, including base-image measurements
- **THEN** the report labels G1 open and lists the missing evidence without inventing values or a deadline bound

#### Scenario: Collect base without interactive access
- **WHEN** the learner boots an attributed temporary measurement copy of a base image
- **THEN** a bounded recorder runs after normal startup and exports monotonic boot timestamps, settled RAM samples, board conditions and identity evidence to the boot partition before normal shutdown
- **AND** the original image, package selection, account policy, kernel and device tree are preserved; observer overhead and derivative attribution are disclosed, and no diagnostic SSH or workload suite is added to base
- **AND** diagnostic copies use the same observer and retain repeated scheduling trials, while shortened software fixtures cannot pass physical acceptance

### Requirement: Physical board controller activation

Pi images SHALL select versioned device-tree configuration that enables HW-2 I2C1 at an initial 100 kHz and SPI0 with CE0 on GPIO8. The I2C character-device interface SHALL be available after normal boot. Stock spidev clients MUST remain disabled so the future FPGA client can own CE0. This configuration MUST preserve the Bluetooth UART and existing kernel identity; installed modules alone MUST NOT establish controller readiness.

#### Scenario: Inspect the enabled buses after reboot
- **WHEN** the learner boots the bus-enabled candidate and inspects its running device tree and platform drivers
- **THEN** I2C1 and SPI0 are enabled, their drivers are bound, `/dev/i2c-1` and SPI master 0 are present, and the I2C clock property is 100000 Hz
- **AND** no stock spidev client owns CE0, Bluetooth remains available, and slave/electrical transactions remain subject to their later bench gates

### Requirement: Learner-run first-boot experiment

The runbook SHALL identify prerequisites, host/Windows/target command contexts, expected observations and collection commands, teaching boot attribution, address selection and key authentication. It SHALL include an optional explain-back question without making it an acceptance gate.

#### Scenario: Explain observed behavior
- **WHEN** the learner captures first-boot network traffic and inspects the running configuration
- **THEN** the runbook explains why a DHCP Discover is not a lease and why a matching PREEMPT_RT configuration proves kernel selection independently of latency results
