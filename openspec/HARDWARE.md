# QuantumTooth hardware

Physical SoT, baseline 2026-11-08. Frozen for future feature work; revisions require explicit user authorization. [DESIGN.md](DESIGN.md) owns software, behavior and validation. This is a planned setup, not a claim that boards have been acquired or tested.

## HW-1 — equipment and capacity

| Item | Selected baseline / physical constraints |
|---|---|
| Linux target | Raspberry Pi 4 Model B, 4 GB LPDDR4; BCM2711, four 64-bit Cortex-A72 cores, nominal 1.5 GHz baseline; actual revision/clocks recorded at bring-up |
| Pi interfaces | Integrated Bluetooth 5.0/BLE via HCI UART; microSD boot/rootfs; 40-pin 3.3 V GPIO header; I2C1, SPI0 and GPIO IRQ allocated below; USB/Ethernet for setup |
| FPGA | iCEBreaker-compatible iCE40UP5K-SG48 board; reference: iCEBreaker v1.1. 5,280 LUTs, 8 DSP 16x16 MACs, 120 Kbit EBR (~15 KiB), 1 Mbit SPRAM (128 KiB); 12 MHz reference clock/PLL |
| FPGA storage/I/O | Board documentation lists 128 Mbit configuration flash; v1.1 adds 64 Mbit QSPI PSRAM. Verify exact subrevision/parts before use. Pmod headers need a mapped cable; no Pi 40-pin compatibility |
| Phone | User's Google Pixel 7a, Android 17; BLE central/client. Foreground app receives/decrypts; no FPGA responsibility |
| Programmable target | Planned Raspberry Pi Pico (RP2040), non-W: dual Cortex-M0+ up to 133 MHz, 264 KB SRAM, 2 MB flash; Rust firmware emulates an I2C device at 7-bit address 0x42 |
| Healthy target | Planned TMP102 breakout powered at 3.3 V; ADD0 tied to GND selects 7-bit address 0x48; normal polling provides a shared-bus continuity reference |
| Recovery circuit | FPGA-controlled open-drain reset stage to Pico RUN; e.g. small N-MOSFET with gate pulldown. FPGA request HIGH asserts RUN LOW; defaults to reset-released while FPGA is unconfigured/disarmed |
| Bench | Four-channel scope on SDA, SCL, physical RESET_N and FPGA DECISION; logic analyzer can additionally capture IRQ. Workstation, USB cables, short mapped harness and microSD |
| Power/cooling | Pi regulated USB-C 5 V/3 A supply and cooling; FPGA/Pico powered through their specified USB inputs. Account for USB load if Pi powers them |

External memory stores data/weights; it does not increase LUT/DSP capacity. Fit, achieved clock and complete inference latency require synthesis/place-and-route evidence.

![Physical setup and independent data paths](../docs/diagrams/hardware.svg)

## HW-2 — reference harness

BCM numbering and Pi physical header positions are both given. Pmod names denote reference-board connector signals, not FPGA package pins. Verify the compatible board's schematic/header mapping before wiring and programming.

| Net | Pi BCM / physical pin | Other endpoints / direction |
|---|---|---|
| SDA | GPIO2 / 3 | Pico GP4; TMP102 SDA; FPGA P1B1 input-only tap |
| SCL | GPIO3 / 5 | Pico GP5; TMP102 SCL; FPGA P1B2 input-only tap |
| SPI0 SCLK | GPIO11 / 23 | FPGA P1A1 input |
| SPI0 MOSI | GPIO10 / 19 | FPGA P1A2 input |
| SPI0 MISO | GPIO9 / 21 | FPGA P1A3 output; high impedance when deselected |
| SPI0 CE0_N | GPIO8 / 24 | FPGA P1A4 input |
| FPGA action IRQ | GPIO25 / 22 | FPGA P1B3 output, HIGH while unread post-action notifications remain; pulldown keeps unconfigured FPGA inactive |
| RESET_REQ | No Pi GPIO connection | FPGA P1B4 output to open-drain stage to Pico RUN; HIGH request asserts physical RESET_N LOW |
| DECISION marker | No Pi GPIO connection | FPGA P1B7 output to scope/analyzer; qualification marker defined by DESIGN HWE-7; default LOW |
| Sensor power | 3V3 / 1 | TMP102 V+ only; check breakout pull-ups and decoupling |
| Ground | GND / 6 | Common Pi/FPGA/Pico/sensor/analyzer ground; Pmod GND pins per connector |

The monitored I2C bus connects the Pi controller and both targets. FPGA SPI is a separate slave/control link, not the configuration-flash bus; a custom kernel driver owns CE0, so do not also bind spidev there. USB programming is separate. GPIO24/physical 18 is unassigned: remove the former Pi reset connection; reset has one owner, the FPGA.

## HW-3 — electrical and timing constraints

- All inter-board signals use compatible 3.3 V logic; Pi GPIO is not 5 V tolerant. Keep separately regulated 3.3 V rails separate; Pmod power pins are not signal pins.
- SDA/SCL are open-drain with pull-ups to the monitored 3.3 V rail. Account for Pi and breakout pull-ups in parallel; verify effective resistance, bus capacitance and rise time. FPGA taps remain high impedance, including reset/configuration states.
- Start I2C at 100 kHz; qualify 400 kHz separately. Start runtime SPI at a conservative 1 MHz and qualify its actual timing. Select FPGA sampling clock/filtering from measured timing; synchronize asynchronous inputs and account for capture latency.
- Keep the harness short with ground returns; analyzer probes/taps add loading. Do not claim analog degradation prediction from digital samples alone.
- Pico fault modes may introduce timing variation, not-ready responses, supported clock stretching or stuck-low behavior. Fault injection only pulls bus lines LOW/releases them; never drives them HIGH. RESET must release the injected condition.
- Reset and IRQ do not traverse I2C. FPGA P1B4 controls the target reset stage; IRQ only reports a completed pulse to the Pi. Linux is absent from the reset-assertion path. A stuck line does not by itself imply a whole-kernel crash.
- Probe RESET_N at Pico RUN, after the reset stage; a gate-output edge alone does not measure physical reset assertion. Choose scope/analyzer sample rate and probe loading for the shortest interval measured; record timing uncertainty and FPGA tick rate. Refer to DESIGN HWE-7 for interval definitions.
- Verify safe boot, unpowered-board behavior and absence of back-powering before connecting mixed power states. Reserve the Pi Bluetooth UART/pinmux; debugging must not steal it.

## HW-4 — facts to record at bring-up

Record Pi revision, iCEBreaker v1.1 subrevision/schematic and fitted memories, breakout pull-ups, cable length, supply current/temperature, actual clock rates and signal captures. These are verification gates, not assumed measurements. A wiring/part change needs an explicitly authorized SoT revision; measurements belong in a separate bring-up report.

## References

[Pi 4 datasheet](https://datasheets.raspberrypi.com/rpi4/raspberry-pi-4-datasheet.pdf) · [iCEBreaker board](https://docs.icebreaker-fpga.org/hardware/icebreaker/) · [v1.1 changes](https://1bitsquared.com/products/icebreaker) · [UP5K resources](https://www.latticesemi.com/Products/FPGAandCPLD/iCE40UltraPlus) · [Pico](https://www.raspberrypi.com/documentation/microcontrollers/pico-series.html) · [TMP102](https://www.ti.com/product/TMP102) · [I2C electrical specification](https://www.nxp.com/docs/en/user-guide/UM10204.pdf)

The figure's adjacent .puml is its PlantUML source; this document owns physical assignments.
