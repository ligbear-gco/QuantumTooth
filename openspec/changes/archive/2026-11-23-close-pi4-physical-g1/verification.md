# Physical G1 closeout — 2026-11-23

**G1 accepted under the operator-authorized supply-current assumption.** The
operator directs treating current as measured and within the board supply's
seller-certified range. No numeric reading or instrument trace is supplied;
the supply's 5 V/3 A rating is not substituted for measured consumption.
DESIGN BASE-1..3/G1 and HARDWARE HW-1..4 remain unchanged.

## Reviewed physical evidence

- Pi 4 Model B Rev 1.5, revision c03115, 4 GB; 32 GB microSD, regulated USB-C
  supply, heatsink/fan and direct Ethernet. All four Imager read-back verifications
  confirmed by the operator.
- Four attributed physical runs: RT/non-RT × base/diagnostic. Configuration,
  boot files, module ABI, packages, services and board identity match their
  recorded build inputs. Instrumented copies preserve original images and
  base access policy; observer overhead is disclosed.
- Bluetooth firmware/controller and CMAC initialize cleanly; LE roles/features
  present. I2C1 at 100 kHz and SPI0 bind correctly; CE0 is free of stock spidev.
  GPIO support, microSD operation and external USB reader binding are verified.
  Empty reader enumeration does not demonstrate a storage transfer.
- Twelve successful diagnostic trials: three 60-second idle/load pairs per
  profile, 720000 samples, no histogram overflow. Histogram/JSON counts agree;
  FIFO 80/CPU3, load coverage and child exit statuses checked. RT does not show
  consistent improvement in every statistic; no deadline bound is inferred.
- Four RAM/boot observations and all 270 raw result files retained with hash
  inventories under ignored build evidence. RAM is sampled three times per
  boot; boot duration excludes EEPROM/firmware. Unsynchronized target calendar
  dates are not used for timing. Initial failures remain in the evidence.

| Image | Compressed WIC MiB | Median RAM MiB | Kernel → systemd finish s | Idle / loaded max µs |
|---|---:|---:|---:|---:|
| RT base | 54.87 | 112.42 | 6.168 | — |
| Non-RT base | 54.78 | 108.26 | 5.407 | — |
| RT diagnostic | 59.90 | 112.53 | 6.372 | 56 / 337 |
| Non-RT diagnostic | 59.82 | 107.84 | 5.509 | 68 / 323 |

## Implementation checks

- Native access-package build succeeds (523 tasks). Without local credentials,
  base metadata resolves and diagnostic key validation rejects the comment-only
  fallback. Authorization is an ignored operator input; no default identity,
  private keys or live fingerprints are distributed in tracked metadata/docs.
- Protected architecture/hardware and diagram hash checks pass. OpenSpec strict
  validation passed before archive; all seven requirements and their scenarios
  were compared exactly with the synchronized main specification.
- The final course lab consolidates deployment, attribution, RAM/load testing,
  raw-data review and restoration of an ordinary image. The host Python image
  preparer and target shell collector remain complementary maintained tools.
- Final-lab export and release hash verification executed successfully; shell
  examples and five PowerShell examples passed syntax checks. All 270 raw-file
  hashes were rechecked. Publication paths/links and staged privacy cleanup passed.

See [the final lab](../../../../docs/courses/G1_Architecture_and_Practical_Guide.md#final-physical-lab)
for the consolidated procedure and full results.

G2 phone/BLE/PQC delivery and G3..5 FPGA, electrical and recovery timing remain
separate gates. Restoration of the ordinary runtime card is an operator procedure,
not a newly observed physical result. A full RAM integrity qualification is not
claimed from bounded workload tests.

Learning demonstrated: trace firmware/kernel/packages to observed behavior;
distinguish address discovery from connectivity/authentication; compare measured
profiles with explicit instrumentation and statistical limits. Next experiment:
verify a controlled BLE connection in G2, then implement its application protocol.
