# E7030E Embedded System Design
Project designed and written in Rust in conjunction with the E7020E Embedded System Design course at Luleå University of Technology. 

## Project description
The purpose of this project is to create a breathalyzer to estimate the blood alchohol content (BAC) in a person's breath. 
To do this a prototype PCB will be designed connected to an alchohol sensor, a button to start the breathalyzer and a small display to show the alchohol permille detected. Over LoRa this device will also alert a ThinkBoard server that this user's breath contains too much alchohol, i.e. they are about to pass out. 

The goal is to package this is in a simple and safe to use way for users to interact with. The data that is sent to the ThinkBoard server could be seen as sensitive data and would be ideal to encrypt it before transmitting it. 

### Limitations
There is no sure way to calibrate this device as the project team does not have access to a real industry-grade breathalyzer. Thus this device can only very roughly estimate the BAC of a person's breath and SHOULD NOT be trusted in any serious situation where it is critical to know the real BAC.

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
