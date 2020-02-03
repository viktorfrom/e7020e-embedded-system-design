# E7020E Embedded System Design
Project designed and written in Rust in conjunction with the E7020E Embedded System Design course at Luleå University of Technology. 

## Project description
The purpose of this project is to create a breathalyzer to estimate the blood alchohol content (BAC) in a person's breath. 
To do this a prototype PCB will be designed connected to an alchohol sensor, a button to start the breathalyzer and a small display to show the alchohol permille detected. Over LoRa this device will also alert a ThinkBoard server that this user's breath contains too much alchohol, i.e. they are about to pass out. 

The goal is to package this is in a simple and safe to use way for users to interact with. The data that is sent to the ThinkBoard server could be seen as sensitive data and would be ideal to encrypt it before transmitting it. 

### Limitations
There is no sure way to calibrate this device as the project team does not have access to a real industry-grade breathalyzer. Thus this device can only very roughly estimate the BAC of a person's breath and SHOULD NOT be trusted in any serious situation where it is critical to know the real BAC.

## Components
* [Murata CMWX1ZZABZ-078](https://www.digikey.com/product-detail/en/murata-electronics/CMWX1ZZABZ-078/490-16143-1-ND/6834151)
* [Grove - Alchohol sensor](https://www.elfa.se/en/grove-alcohol-sensor-seeed-studio-101020044/p/30069826)
* Some sort of display like [LCD-S301C31TR ](https://www.digikey.se/product-detail/en/lumex-opto-components-inc/LCD-S301C31TR/67-1788-ND/469773)

## Requirements
* Rustup 1.14.0+

## Getting started
### Running tests
Download and install Rustup from,
```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```
source: https://rustup.rs

## Nucleo - Installation
The STM32 Nucleo Board requires initial setup before code can be interfaced with and eventually run code in the PCB. Nucleo software can be found at, https://docs.rust-embedded.org/discovery/index.html.

### rustc & cargo
Install rustup by following the instructions at https://rustup.rs.

If you already have rustup installed double check that you are on the stable channel and your stable toolchain is up to date. rustc -V should return a date newer than the one shown below:
```
rustc -V
rustc 1.31.0 (abe02cefd 2018-12-04)
```
### itmdump 
```
cargo install itm --vers 0.3.1
itmdump -V
itmdump 0.3.1
```

### cargo-binutils

```
rustup component add llvm-tools-preview
```

```
cargo install cargo-binutils --vers 0.1.4
```

```
cargo size -- -version
LLVM (http://llvm.org/):
  LLVM version 8.0.0svn
  Optimized build.
  Default target: x86_64-unknown-linux-gnu
  Host CPU: skylake
```

### Ubuntu / Debian
```
sudo apt-get install \
  bluez \
  rfkill
```

### MacOS
All the tools can be install using Homebrew:
```
brew cask install gcc-arm-embedded

brew install minicom openocd
```

### Build it
For the F3, we'll to use the thumbv7em-none-eabihf target. Before cross compiling you have to download pre-compiled version of the standard library (a reduced version of it actually) for your target. That's done using rustup:
```
rustup target add thumbv7em-none-eabihf
```



## Authors
* Viktor From - vikfro-6@student.ltu.se - [viktorfrom](https://github.com/viktorfrom)
* Mark Hakansson - marhak-6@student.ltu.se - [markhakansson](https://github.com/markhakansson)
* Kyle Drysdale 

## License
Licensed under the MIT license. See [LICENSE](LICENSE) for details.
