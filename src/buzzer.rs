use stm32l0xx_hal::{
    gpio::{*, gpioa::PA3},
    prelude::*,
};

pub struct Buzzer {
    pub pin: PA3<Output<PushPull>>,
    pub on: bool,
    pub enabled: bool
}

impl Buzzer {
    pub fn new(
        pin: PA3<Input<Floating>>
    ) -> Buzzer {
        Buzzer {
            pin: pin.into_push_pull_output(),
            on: false,
            enabled: false
        }
    }

    pub fn enable(&mut self) {
        self.enabled = true;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
        self.on = false;
        self.pin.set_low().unwrap();
    }

    pub fn toggle_pwm(&mut self) {
        if self.enabled {
            if self.on {
                self.pin.set_low().unwrap();
            } else {
                self.pin.set_high().unwrap();
            }
            self.on = !self.on;
        }
    }
}