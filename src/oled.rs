use ssd1306::{interface::{DisplayInterface, SpiInterface}, prelude::*, Builder, Error};
extern crate panic_semihosting;
use rtfm::app;


use stm32l0xx_hal::{
    delay::Delay,
    exti::TriggerEdge,
    gpio::{
        gpiob::{PB12, PB13, PB15, PB8, PB9},
        *,
    },
    pac,
    pac::SPI2,
    prelude::*,
    rcc::Config,
    spi::{self, Mode, NoMiso, Phase, Polarity, Spi},
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
    //pub spi: Spi<SPI2, (PB13<Input<Floating>>, NoMiso, PB15<Input<Floating>>)>,
    pub pb9: PB9<Output<PushPull>>,
    pub delay: Delay,
    pub disp: GraphicsMode<SpiInterface<Spi<SPI2, (PB13<Input<Floating>>, NoMiso, PB15<Input<Floating>>)>, PB8<Output<PushPull>>>>,
    pub state: bool,
}

impl Oled {
    pub fn new(
        spi: Spi<SPI2, (PB13<Input<Floating>>, NoMiso, PB15<Input<Floating>>)>,
        pb8: PB8<Input<Floating>>,
        pb9: PB9<Input<Floating>>,
        delay: Delay,
    ) -> Oled {
        Oled {
            //spi: spi,
            pb9: pb9.into_push_pull_output(),
            disp: Builder::new()
                .connect_spi(spi, pb8.into_push_pull_output())
                .into(),
            delay: delay,
            state: false,
        }
    }

    /// Turns on the oled
    pub fn on(&mut self) {
        let mut res = &mut self.pb9;

        //self.disp.reset(&mut res, &mut self.delay).unwrap();
        self.disp.init().unwrap();

        self.disp.clear();

        let style1 = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(2)
            .fill_color(BinaryColor::On)
            .build();

        let style2 = PrimitiveStyleBuilder::new()
            .stroke_color(BinaryColor::On)
            .stroke_width(2)
            .fill_color(BinaryColor::Off)
            .build();

        Circle::new(Point::new(27, 23), 5)
            .into_styled(style2)
            .draw(&mut self.disp);

        Rectangle::new(Point::new(10, 20), Point::new(25, 35))
            .into_styled(style1)
            .draw(&mut self.disp);

        Rectangle::new(Point::new(10, 15), Point::new(25, 20))
            .into_styled(style2)
            .draw(&mut self.disp);

        let t1 = Text::new("~ Breathalyzer", Point::new(35, 16))
            .into_styled(TextStyle::new(Font6x12, BinaryColor::On));

        let t2 = Text::new(" 0.0002", Point::new(35, 35))
            .into_styled(TextStyle::new(Font12x16, BinaryColor::On));

        t1.draw(&mut self.disp);
        t2.draw(&mut self.disp);

        self.disp.flush().unwrap();

        self.state = true;
    }
}
