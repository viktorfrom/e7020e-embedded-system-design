//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

mod breathalyzer;

extern crate panic_semihosting;

use crate::breathalyzer::Breathalyzer;
use stm32l0xx_hal as hal;
use cortex_m::peripheral::DWT;
use cortex_m_semihosting::hprintln;

use stm32l0xx_hal::{
    adc,
    exti::TriggerEdge,
    gpio::*,
    pac,
    prelude::*,
    rcc::Config,
    spi,
    syscfg,
    timer
};

#[rtfm::app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        EXT: pac::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        BUZZER: gpioa::PA3<Output<PushPull>>,
        TIMER: timer::Timer<pac::TIM2>,
        BREATHALYZER: Breathalyzer,
        #[init(false)]
        BUZZER_ON: bool,
        #[init(false)]
        STATE: bool
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Configure the clock.
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());
        let mut syscfg = syscfg::SYSCFG::new(cx.device.SYSCFG, &mut rcc);

        // Configure ADC
        let mut adc = adc::Adc::new(cx.device.ADC, &mut rcc);

        // Acquire the GPIOB peripheral. This also enables the clock for GPIOB in
        // the RCC register.
        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioc = cx.device.GPIOC.split(&mut rcc);

        // Configure inputs
        let button = gpioa.pa4.into_pull_up_input();

       // Configure timer
       let mut tim2 = timer::Timer::tim2(cx.device.TIM2, 1000.ms(), &mut rcc);
       tim2.listen();

        // External interrupt
        let exti = cx.device.EXTI;

        // Configure external interrupt for button
        exti.listen(
            &mut syscfg,
            button.port(),
            button.pin_number(),
            TriggerEdge::Falling,  
        );

        // SPI for OLED
        // let sck = gpiob.pb3;
        // let miso = gpioa.pa6;
        // let mosi = gpioa.pa7;
        // let nss = gpioa.pa15.into_push_pull_output();

        // Configure outputs
        let mut buzzer = gpioa.pa3.into_push_pull_output();

        let mut breathalyzer = Breathalyzer::new(gpioa.pa5, gpioa.pa2, adc);
        breathalyzer.on();
    
        // Return the initialised resources.
        init::LateResources {
            EXT: exti,
            BUTTON: button,
            BUZZER: buzzer,
            TIMER: tim2,
            BREATHALYZER: breathalyzer
        }
    }

    #[task(binds = EXTI4_15, priority = 2, resources = [BUTTON, EXT, BREATHALYZER])]
    fn exti4_15(cx: exti4_15::Context) {
        cx.resources.EXT.clear_irq(cx.resources.BUTTON.pin_number());
        cx.resources.BREATHALYZER.state = !cx.resources.BREATHALYZER.state;
    }

    #[task(binds = TIM2, priority = 2, resources = [BREATHALYZER])]
    fn sensor(cx: sensor::Context) {
        if cx.resources.BREATHALYZER.state {
            let value: u16 = cx.resources.BREATHALYZER.read();
            hprintln!("Value: {:#}", value).unwrap();            
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};