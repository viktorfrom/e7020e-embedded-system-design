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
    spi::{self, Mode, NoMiso, Phase, Polarity},
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
    pub cs: PB12<Input<Floating>>,
    pub sck: PB13<Input<Floating>>,
    pub mosi: PB15<Input<Floating>>,
    pub spi: PB15<Input<Floating>>,
    pub state: bool,
}

impl Oled {
    pub fn new(
        cs: PB12<Input<Floating>>,
        sck: PB13<Input<Floating>>,
        mosi: PB15<Input<Floating>>,
        cx: cx,
        rcc: rcc,
    ) -> Oled {
        Oled {
            cs: cs,
            sck: sck,
            mosi: mosi,
            spi: cx
                .device
                .SPI2
                .spi((sck, NoMiso, mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc),
            state: false,
        }
    }
}