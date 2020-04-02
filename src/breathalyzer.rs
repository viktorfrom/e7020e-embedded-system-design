use stm32l0xx_hal::{
    adc::Adc,
    gpio::{
        gpioa::{PA2, PA5},
        *,
    },
    prelude::*,
};

#[derive(Debug)]
pub enum BAC {
    NONE,
    LOW,
    MEDIUM,
    HIGH,
    VERY_HIGH,
    DEATH
}

pub struct Breathalyzer {
    pub heater: PA5<Output<PushPull>>,
    pub dat: PA2<Analog>,
    pub adc: Adc,
    pub state: bool,
}

impl Breathalyzer {
    pub fn new(heater: PA5<Input<Floating>>, dat: PA2<Input<Floating>>, adc: Adc) -> Breathalyzer {
        Breathalyzer {
            heater: heater.into_push_pull_output(),
            dat: dat.into_analog(),
            adc: adc,
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

    /// Reads the value from ADC
    pub fn read(&mut self) -> u16 {
        let value: u16 = self.adc.read(&mut self.dat).unwrap();
        value
    }
}
