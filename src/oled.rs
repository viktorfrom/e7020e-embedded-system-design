use ssd1306::{interface::SpiInterface, prelude::*, Builder};
extern crate panic_semihosting;

use stm32l0xx_hal::{
    delay::Delay,
    gpio::{
        gpiob::{PB13, PB15, PB8, PB9},
        *,
    },
    pac::SPI2,
    spi::{NoMiso, Spi},
};

use embedded_graphics::{
    fonts::{Font8x16, Font6x12, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::{Circle, Rectangle},
    style::{PrimitiveStyleBuilder, TextStyle},
};

pub struct Oled {
    //pub spi: Spi<SPI2, (PB13<Input<Floating>>, NoMiso, PB15<Input<Floating>>)>,
    pub pb9: PB9<Output<PushPull>>,
    pub delay: Delay,
    pub disp: GraphicsMode<
        SpiInterface<
            Spi<SPI2, (PB13<Input<Floating>>, NoMiso, PB15<Input<Floating>>)>,
            PB8<Output<PushPull>>,
        >,
    >,
    pub style1:
        embedded_graphics::style::PrimitiveStyle<embedded_graphics::pixelcolor::BinaryColor>,
    pub style2:
        embedded_graphics::style::PrimitiveStyle<embedded_graphics::pixelcolor::BinaryColor>,
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
            pb9: pb9.into_push_pull_output(),
            disp: Builder::new()
                .connect_spi(spi, pb8.into_push_pull_output())
                .into(),
            delay: delay,
            style1: PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(2)
                .fill_color(BinaryColor::On)
                .build(),
            style2: PrimitiveStyleBuilder::new()
                .stroke_color(BinaryColor::On)
                .stroke_width(2)
                .fill_color(BinaryColor::Off)
                .build(),
            state: false,
        }
    }


    pub fn on(&mut self, input: &str) {
        let res = &mut self.pb9;
        let message: &str;

        self.disp.reset(res, &mut self.delay).unwrap();
        self.disp.init().unwrap();

        self.disp.clear();

        Circle::new(Point::new(27, 23), 5)
            .into_styled(self.style2)
            .draw(&mut self.disp);

        Rectangle::new(Point::new(10, 20), Point::new(25, 35))
            .into_styled(self.style1)
            .draw(&mut self.disp);

        Rectangle::new(Point::new(10, 15), Point::new(25, 20))
            .into_styled(self.style2)
            .draw(&mut self.disp);

        let t1 = Text::new("Breathalyzer", Point::new(40, 16))
            .into_styled(TextStyle::new(Font6x12, BinaryColor::On));
            
        if input != "" {
            message = input;
        } else {
            message = "Ready";
        }

        let t2 = Text::new( message, Point::new(40, 35))
            .into_styled(TextStyle::new(Font8x16, BinaryColor::On));

        t1.draw(&mut self.disp);
        t2.draw(&mut self.disp);

        self.disp.flush().unwrap();

        self.state = true;
    }

    pub fn off(&mut self) {
        let res = &mut self.pb9;

        self.disp.reset(res, &mut self.delay).unwrap();
        self.disp.init().unwrap();

        self.disp.clear();

        // clear display

        self.disp.flush().unwrap();

        self.state = false;
    }
}
