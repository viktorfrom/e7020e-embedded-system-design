// use cortex_m_semihosting::hprintln;

use stm32l0xx_hal::{
    adc::Adc,
    gpio::{
        gpioa::{PA2, PA5},
        *,
    },
    prelude::*,
};

#[derive(Debug, Clone)]
pub enum BAC {
    NONE,
    LOW,
    MEDIUM,
    HIGH,
}

pub struct Breathalyzer {
    pub heater: PA5<Output<PushPull>>,
    pub dat: PA2<Analog>,
    pub adc: Adc,
    pub curr_val: u16,
    pub state: bool,
}

impl Breathalyzer {
    pub fn new(heater: PA5<Input<Floating>>, dat: PA2<Input<Floating>>, adc: Adc) -> Breathalyzer {
        Breathalyzer {
            heater: heater.into_push_pull_output(),
            dat: dat.into_analog(),
            adc: adc,
            curr_val: 0,
            state: false,
        }
    }

    /// Turns on the breathalyzer by starting the heater
    pub fn on(&mut self) {
        self.state = true;
        self.heater.set_low().unwrap();
    }

    /// Shuts down the heater
    pub fn off(&mut self) {
        self.state = false;
        self.heater.set_high().unwrap();
    }

    /// Calculates value from ADC
    pub fn read(&mut self) -> BAC {
        let val: u16 = self.adc.read(&mut self.dat).unwrap();
        //hprintln!("{:#} / {:#} = {:#}", val, self.curr_val, (val * 100) / self.curr_val).unwrap();

        if ((val * 100) / self.curr_val) >= 90 {
            return BAC::LOW;
        } else if ((val * 100) / self.curr_val) >= 80 {
            return BAC::MEDIUM;
        } else if ((val * 100) / self.curr_val) >= 65 {
            return BAC::HIGH;
        } else {
            return BAC::NONE;
        }
    }

    /// Reads the value from ADC
    pub fn read_curr(&mut self) -> u16 {
        let value: u16 = self.adc.read(&mut self.dat).unwrap();
        return value;
    }
}
