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
* [4-pin connector to sensor](https://www.elfa.se/en/grove-universal-pin-connector-seeed-studio-110990030-10pcs-pack/p/30069939)
* [SSD1306 based display](https://cdon.se/hem-tradgard/oled-display-0-96-tum-vit-128x64-pixlar-ssd1306-spi-p50506639)
* [65100516121 USB mini-B](https://www.elfa.se/en/socket-horizontal-mini-usb-smd-wuerth-elektronik-65100516121/p/14257103)
* [Pin headers](https://www.elfa.se/en/wr-phd-straight-male-pcb-header-through-hole-54mm-wuerth-elektronik-61300611121/p/30024526)
* [LM1117IMP-ADJ voltage regulator](https://www.elfa.se/en/ldo-voltage-regulator-800ma-sot-223-texas-instruments-lm1117imp-adj-nopb/p/30019193)
* [2x Electrolytic capacitors 10uF](https://www.elfa.se/en/aluminium-electrolytic-capacitor-10-uf-50-20-vs-panasonic-eee1ha100wr/p/30108011)
* [2x Buttons](https://www.elfa.se/en/print-key-50-ma-12-vdc-te-connectivity-1437565/p/13566525)
* [1x Piezo buzzer](https://www.elfa.se/en/piezo-buzzer-70-db-khz-15-murata-pkm13epyh4000-a0/p/13787082)
* [1x Antenna](https://www.elfa.se/en/micro-coaxial-straight-socket-micro-coaxial-connector-50ohm-6ghz-molex-73412-0110/p/30076410)

## Requirements
* Rustup 1.14.0+
* rustc 1.31.0+
* GNU ARM embedded toolchain (check your package manager or manually install it)
* OpenOCD

## Setup

### rustc & cargo
Install rustup by following the instructions at https://rustup.rs.

If you already have rustup installed double check that you are on the stable channel and your stable toolchain is up to date. rustc -V should return a date newer than the one shown below:
```
rustc -V
rustc 1.31.0 (abe02cefd 2018-12-04)
```
Then install the following tools for rustup and cargo:
```
rustup component add llvm-tools-preview
```
```
rustup target add thumbv6m-none-eabi
```
```
cargo install cargo-binutils
```
### Linux
Below are the packages you will need for Linux. Names might vary depending on your distrobution, you might need to install it manually if you can't find it using your distrobution's package manager.
```
openocd
arm-none-eabi-gdb
gcc / gcc-c++ (as well as their respective dev packages)
llvm-8.0 (and its dev package)
```

### MacOS
All the tools can be install using Homebrew:
```
brew cask install gcc-arm-embedded

brew install openocd
```

## Build instructions
For the F3, we'll to use the thumbv7em-none-eabihf target. Before cross compiling you have to download pre-compiled version of the standard library (a reduced version of it actually) for your target. That's done using rustup:
```
rustup target add thumbv7em-none-eabihf
```



## Authors
* Viktor From - vikfro-6@student.ltu.se - [viktorfrom](https://github.com/viktorfrom)
* Mark Hakansson - marhak-6@student.ltu.se - [markhakansson](https://github.com/markhakansson)
* Kyle Drysdale - kyldry-5@student.ltu.se  - [KyleLouisDrysdale](https://github.com/KyleLouisDrysdale)

## License
Licensed under the MIT license. See [LICENSE](LICENSE) for details.
