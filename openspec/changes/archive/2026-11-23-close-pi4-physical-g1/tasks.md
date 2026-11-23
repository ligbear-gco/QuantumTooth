## 1. Learn and prepare diagnostic access

- [x] 1.1 Teach packet discovery versus address assignment and record confirmed bench facts, unresolved Windows state and source-of-truth hashes; verify a physical bring-up runbook and preflight inventory exist.
- [x] 1.2 Add diagnostic-only public-key SSH and DHCPv4/IPv4LL configuration; verify package ownership, restrictive key permissions, key-only authentication, root account handling and unchanged base/UART policy from recipes and parsed metadata.
- [x] 1.3 Build RT/non-RT diagnostic images after preserving previous evidence; verify manifests, final rootfs policy and matching kernel/DT/module identities, recording failures instead of claiming hardware success.

## 2. Build, flash and inspect the real board

- [x] 2.1 Export the identified partitioned deployment image and SHA-256 to Windows; verify the copy hash, Imager custom-image selection and successful write/read-back verification on the spare card.
- [x] 2.2 Boot with Wireshark on physical Ethernet; retain DHCP/ARP capture and actual PC/Pi addresses, verify TCP/22 and public-key SSH, and record firmware/board/RAM/kernel/configuration/module/service evidence.
- [x] 2.3 Verify physical Bluetooth controller/firmware initialization, microSD/USB operation, and I2C1/SPI0/GPIO support through enabled DT nodes and driver binding; retain unresolved hardware failures and distinguish G3..5 bench tests.
- [x] 2.4 Correct the observed missing CMAC runtime dependency; verify matched module ABI, rebuilt image manifest/module content, temporary hardware loading and Bluetooth power-cycle results, then confirm a clean corrected-image boot without a new CMAC error.

## 3. Measure and explain physical G1

- [x] 3.1 Confirm adequate cooling and establish a concrete BASE-2-compatible base-image collection path before dependent baseline measurements; verify the method and collection overhead are documented without installing diagnostic SSH/test suites into base.
- [x] 3.2 Collect matched physical RT/non-RT base/diagnostic sizes, settled RAM, defined boot interval and at least three diagnostic idle/load scheduling trials per profile/condition; verify raw outputs, identities, temperatures/clocks and failed attempts are retained with common workload parameters.
- [x] 3.3 Compare all observations with the physical handoff checklist, explain attribution/addressing/measurement lessons, and report G1 passed only when every required physical check is supported; verify OpenSpec validation, protected hashes and final evidence review. Supply-current acceptance uses the operator's explicit assumption, disclosed in verification.md; no numeric reading is invented.
