# E7020E Embedded System Design
Project designed and written in Rust in conjunction with the E7020E Embedded System Design course at Luleå University of Technology. 

## Project description
The purpose of this project is to create a breathalyzer to estimate the blood alchohol content (BAC) in a person's breath. 
To do this a prototype PCB will be designed connected to an alchohol sensor, a button to start the breathalyzer and a small display to show the alchohol permille detected. Over LoRa this device will also alert a ThinkBoard server that this user's breath contains too much alchohol, i.e. they are about to pass out. 

The goal is to package this is in a simple and safe to use way for users to interact with. The data that is sent to the ThinkBoard server could be seen as sensitive data and would be ideal to encrypt it before transmitting it. 

### Limitations
There is no sure way to calibrate this device as the project team does not have access to a real industry-grade breathalyzer. Thus this device can only very roughly estimate the BAC of a person's breath and SHOULD NOT be trusted in any serious situation where it is critical to know the real BAC.

## Components / shopping list
* [Murata CMWX1ZZABZ-078](https://www.digikey.com/product-detail/en/murata-electronics/CMWX1ZZABZ-078/490-16143-1-ND/6834151)
* [Grove - Alchohol sensor](https://www.elfa.se/en/grove-alcohol-sensor-seeed-studio-101020044/p/30069826)
* [SSD1306 based display](https://cdon.se/hem-tradgard/oled-display-0-96-tum-vit-128x64-pixlar-ssd1306-spi-p50506639)
* [65100516121 USB mini-B](https://www.elfa.se/en/socket-horizontal-mini-usb-smd-wuerth-elektronik-65100516121/p/14257103)
* [Pin headers](https://www.elfa.se/en/wr-phd-straight-male-pcb-header-through-hole-54mm-wuerth-elektronik-61300611121/p/30024526)
* [LM1117IMP-ADJ voltage regulator](https://www.elfa.se/en/ldo-voltage-regulator-800ma-sot-223-texas-instruments-lm1117imp-adj-nopb/p/30019193)
* [2x Electrolytic capacitors 10uF](https://www.elfa.se/en/radial-electrolytic-capacitor-10uf-20-50vdc-rnd-components-rnd-150ksk050m100d11p50/p/30146087)
* [2x Buttons](https://www.elfa.se/en/print-key-50-ma-12-vdc-te-connectivity-1825910/p/13566549)

## Requirements
* Rustup 1.14.0+

## Getting started
### Running tests
Download and install Rustup from,
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
source: https://rustup.rs

## Authors
* Viktor From - vikfro-6@student.ltu.se - [viktorfrom](https://github.com/viktorfrom)
* Mark Hakansson - marhak-6@student.ltu.se - [markhakansson](https://github.com/markhakansson)
* Kyle Drysdale 

## License
Licensed under the MIT license. See [LICENSE](LICENSE) for details.
