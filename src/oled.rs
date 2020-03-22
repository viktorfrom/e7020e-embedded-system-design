use ssd1306::{prelude::*, Builder};
extern crate panic_semihosting;
use rtfm::app;

use stm32l0xx_hal::{
    delay::Delay,
    exti::TriggerEdge,
    gpio::{
        gpiob::{PB12, PB13, PB15},
        *,
    },
    pac,
    prelude::*,
    rcc::Config,
    //spi::{self, Mode, NoMiso, Phase, Polarity},
    syscfg,
    timer::Timer,
};

use embedded_graphics::{
    fonts::{Font12x16, Font6x12, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle},
    style::{PrimitiveStyle, PrimitiveStyleBuilder, TextStyle},
};

pub struct Oled {
    pub test: spi,
    pub state: bool,
}

impl Oled {
    pub fn new(test: spi) -> Oled {
        Oled { test: test, state: false }
    }
}
