//! Prints "Hello, world!" on the host console using semihosting

#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m_rt::entry;
use cortex_m_semihosting::hprintln;
use stm32l0xx_hal as stm32;

#[rtfm::app(device = stm32l0xx_hal::pac)]
const APP: () = {
    // code here
    #[init]
    fn init(_: init::Context) {}
};
