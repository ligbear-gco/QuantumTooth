# QuantumTooth shopping list

Based on [openspec/HARDWARE.md](openspec/HARDWARE.md), assuming delivery to France.
Product links checked on **2026-11-08**. Recheck stock, VAT and delivery costs at
checkout. Skip anything already owned or ordered.

Use **DigiKey France** for most parts and **1BitSquared Europe** for the FPGA and
logic analyzer. The scope is a DigiKey Marketplace item shipped separately by
Siglent, so two storefronts do not imply two parcels.

## Main order — DigiKey France

| Component | Quantity | Purchase link / selection |
|---|---:|---|
| Raspberry Pi 4 Model B, **4 GB** | 1 | [Raspberry Pi SC0194(9)](https://www.digikey.fr/en/products/detail/raspberry-pi/SC0194-9/10258781) |
| Official EU USB-C power supply, **5.1 V / 3 A** | 1 | [Raspberry Pi SC1412](https://www.digikey.fr/en/products/detail/raspberry-pi/SC1412/10258762) |
| Pi cooling | 1 | [Official fan and heatsink, SC0448](https://www.digikey.fr/en/products/detail/raspberry-pi/SC0448/13281681), with its [compatible case, SC0230](https://www.digikey.fr/en/products/detail/raspberry-pi/SC0230/10258767) |
| microSD card | 1 | [Raspberry Pi 64 GB, SC1629](https://www.digikey.fr/en/products/detail/raspberry-pi/SC1629/24627140). Reuse an existing suitable card if available; the baseline specifies no capacity. |
| Original Raspberry Pi Pico, **RP2040, non-W** | 1 | [SC0915](https://www.digikey.fr/en/products/detail/raspberry-pi/SC0915/13684020); choose single-unit **cut tape** |
| TMP102 temperature-sensor breakout | 1 | [SparkFun SEN-13314](https://www.digikey.fr/fr/products/detail/sparkfun-electronics/13314/6797646) |
| N-channel MOSFET for the reset stage | 2–3, including spares | [onsemi 2N7000, through-hole](https://www.digikey.fr/en/products/detail/onsemi/2N7000/244278); candidate requiring verification at 3.3 V |
| Pulldown resistors | 10, including spares | [10 kΩ, ¼ W, through-hole](https://www.digikey.fr/en/products/detail/stackpole-electronics-inc/CF14JT10K0/1830374); proposed value for reset-gate and IRQ pulldowns |
| Breadboard | 1 | [Adafruit 64, 400 contacts](https://www.digikey.fr/en/products/detail/adafruit-industries-llc/64/7241427) |
| Breakaway male headers, 2.54 mm | 2 × 40 pins | [Sullins PREC040SAAN-RC](https://www.digikey.fr/en/products/detail/sullins-connector-solutions/PREC040SAAN-RC/2774814), for Pico and breakout connections |
| Short jumper wires | Assortment | [Female–female](https://www.digikey.fr/en/products/detail/adafruit-industries-llc/1950/6827084), [male–female](https://www.digikey.fr/en/products/detail/adafruit-industries-llc/1954/6827087), and [male–male](https://www.digikey.fr/en/products/detail/adafruit-industries-llc/1956/6827089) |
| USB-A → USB-C **data** cable | 1 | [Adafruit 4474](https://www.digikey.fr/en/products/detail/adafruit-industries-llc/4474/11587355), for iCEBreaker v1.1 |
| USB-A → micro-USB **data** cable | 1 | [Eaton U050-003](https://www.digikey.fr/fr/products/detail/eaton-tripp-lite/U050-003/4439403), for Pico |
| Four-channel oscilloscope, if unavailable to borrow | 1 | [Siglent SDS1104X-E, 100 MHz, four analog channels](https://www.digikey.fr/fr/products/detail/siglent-technologies/SDS1104X-E/29268554). Listed with four passive probes; sold through DigiKey Marketplace by Siglent, with separate shipping terms. |

## Second order — 1BitSquared Europe

| Component | Quantity | Purchase link / selection |
|---|---:|---|
| iCEBreaker FPGA board | 1 | [iCEBreaker v1.1b](https://1bitsquared.de/en/products/icebreaker), shown in stock when checked |
| Logic analyzer for FPGA capture validation | 1 | [BitMagic Basic](https://1bitsquared.de/en/products/bitmagic-basic), eight channels, up to 24 MHz; select the **mini test hooks** option |

## Assembly and reuse notes

- Confirm the iCEBreaker subrevision, fitted memories and supplied headers. Its
  Pmod connectors need individually mapped wiring to the Pi; they are not Pi
  40-pin-compatible connectors.
- The MOSFET and resistor choices are proposed implementation parts. Verify the
  reset stage at 3.3 V, including reset-released defaults and unpowered-board
  behavior, under HARDWARE HW-1..3. This list does not revise the hardware baseline
  or establish electrical qualification.
- The bare Pico and TMP102 breakout need headers soldered. Reuse or borrow a
  soldering iron, solder, microSD reader, Ethernet cable and workstation. Reuse
  your existing Pixel 7a.
- Power the TMP102 at 3.3 V and check its address selection and pull-ups against
  HW-2..3. Keep separately regulated 3.3 V rails separate and share ground.
- The scope measures SDA, SCL, physical RESET_N and FPGA DECISION simultaneously.
  The logic analyzer adds digital capture capacity. Qualify sample rate and probe
  loading for the shortest interval measured; these product listings do not
  establish timing accuracy.
