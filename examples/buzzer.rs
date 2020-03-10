//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use stm32l0xx_hal as hal;
use cortex_m::peripheral::{DWT, syst, Peripherals};
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
    stm32,
    timer
};

#[rtfm::app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        INT: stm32::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        BUZZER: gpioa::PA3<Output<PushPull>>,
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

        // Acquire the GPIOB peripheral. This also enables the clock for GPIOB in
        // the RCC register.
        let gpioa = cx.device.GPIOA.split(&mut rcc);
        let gpiob = cx.device.GPIOB.split(&mut rcc);
        let gpioc = cx.device.GPIOC.split(&mut rcc);

        // Reset button
        let reset = gpioc.pc0.into_push_pull_output();

        // Configure inputs
        let sx1276_dio0 = gpiob.pb4.into_pull_up_input();
        let button = gpioa.pa4.into_pull_up_input();

        // External interrupt
        let exti = cx.device.EXTI;

        // Configure external interrupt for button
        exti.listen(
            &mut syscfg,
            button.port(),
            button.pin_number(),
            TriggerEdge::Falling,  
        );

        let sck = gpiob.pb3;
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7;
        let nss = gpioa.pa15.into_push_pull_output();

        // Configure outputs
        let mut buzzer = gpioa.pa3.into_push_pull_output();

        // Return the initialised resources.
        init::LateResources {
            INT: exti,
            BUTTON: button,
            BUZZER: buzzer,
        }
    }
 
    #[task(binds = EXTI4_15, priority = 2, resources = [BUTTON, INT], spawn = [button_event])]
    fn exti4_15(cx: exti4_15::Context) {
        cx.resources.INT.clear_irq(cx.resources.BUTTON.pin_number());
        cx.spawn.button_event().unwrap();        
    }

    #[task(priority = 1, resources = [BUZZER, STATE])]
    fn button_event(cx: button_event::Context) {
        if *cx.resources.STATE {
            hprintln!("set high").unwrap();
            *cx.resources.STATE = false;
        } else {
            hprintln!("set low").unwrap();
            *cx.resources.STATE = true;
        }
    }

    #[task(resources = [BUZZER, BUZZER_ON])]
    fn buzzer_on(cx: buzzer_on::Context) {

    }

    #[task(resources = [BUZZER, BUZZER_ON])]
    fn buzzer_off(cx: buzzer_off::Context) {

    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};