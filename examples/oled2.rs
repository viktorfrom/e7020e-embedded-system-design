#![no_std]
#![no_main]

extern crate panic_semihosting;

use rtfm::app;
use ssd1306::{mode::TerminalMode, prelude::*, Builder};
use stm32l0xx_hal as hal;

use hal::{
    delay::Delay,
    exti::TriggerEdge,
    gpio::*,
    pac,
    prelude::*,
    rcc::Config,
    spi::{self, Mode, NoMiso, Phase, Polarity},
    syscfg,
    timer::Timer,
};

use embedded_graphics::{
    fonts::{Font6x8, Text},
    pixelcolor::BinaryColor,
    prelude::*,
    primitives::Circle,
    style::{PrimitiveStyle, TextStyle},
};

use embedded_graphics::{primitives::Rectangle, style::PrimitiveStyleBuilder};

#[app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {}

    #[init]
    fn init(cx: init::Context) {
        // Configure the clock.
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());
        let mut syscfg = syscfg::SYSCFG::new(cx.device.SYSCFG, &mut rcc);

        // Acquire the GPIOB peripheral. This also enables the clock for GPIOB in
        // the RCC register.
        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioc = cx.device.GPIOC.split(&mut rcc);

        let mut cs = gpiob.pb12.into_push_pull_output();
        cs.set_low(); // not sure if needed, did not try without it

        let sck = gpiob.pb13;
        let mosi = gpiob.pb15;

        // Initialise the SPI peripheral.
        let mut spi =
            cx.device
                .SPI2
                .spi((sck, NoMiso, mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc);

        let dc = gpiob.pb8.into_push_pull_output();
        let mut res = gpiob.pb9.into_push_pull_output();

        let mut delay = Delay::new(cx.core.SYST, rcc.clocks);

        let mut disp: GraphicsMode<_> = Builder::new().connect_spi(spi, dc).into();

        disp.reset(&mut res, &mut delay).unwrap();
        disp.init().unwrap();

        disp.clear();

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
            .into_styled(PrimitiveStyle::with_stroke(BinaryColor::On, 1))
            .draw(&mut disp);

        Rectangle::new(Point::new(10, 20), Point::new(25, 35))
            .into_styled(style1)
            .draw(&mut disp);

        Rectangle::new(Point::new(10, 15), Point::new(25, 20))
            .into_styled(style2)
            .draw(&mut disp);

        let t = Text::new("~ Breathalyzer!", Point::new(35, 16))
            .into_styled(TextStyle::new(Font6x8, BinaryColor::On));

        t.draw(&mut disp);

        disp.flush().unwrap();

        // Return the initialised resources.
    }

    extern "C" {
        fn USART4_USART5();
    }
};
