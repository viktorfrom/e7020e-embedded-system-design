//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

extern crate panic_semihosting;

use cortex_m::peripheral::{syst, Peripherals, DWT};
use cortex_m_semihosting::hprintln;
use stm32l0xx_hal as hal;

use stm32l0xx_hal::{
    adc, exti::TriggerEdge, gpio::*, pac, prelude::*, rcc, rcc::Config, spi, stm32, syscfg, timer,
};

#[rtfm::app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        EXT: pac::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        BUZZER: gpioa::PA3<Output<PushPull>>,
        #[init(false)]
        BUZZER_ON: bool,
        #[init(false)]
        STATE: bool,
        #[init(false)]
        PWM_ON: bool,
        TIMER_PWM: timer::Timer<pac::TIM2>,
        TIMER_INTERVAL: timer::Timer<pac::TIM3>,
        RCC: rcc::Rcc,
    }

    #[init]
    fn init(cx: init::Context) -> init::LateResources {
        // Configure the clock.
        let mut rcc = cx.device.RCC.freeze(Config::hsi16());
        let mut syscfg = syscfg::SYSCFG::new(cx.device.SYSCFG, &mut rcc);

        // Timeout is the frequency I think?
        let mut tim2 = timer::Timer::tim2(cx.device.TIM2, 1000.hz(), &mut rcc);
        let mut tim3 = timer::Timer::tim3(cx.device.TIM3, 100.ms(), &mut rcc);
        tim2.listen();
        tim3.listen();

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
            EXT: exti,
            BUTTON: button,
            BUZZER: buzzer,
            TIMER_PWM: tim2,
            TIMER_INTERVAL: tim3,
            RCC: rcc,
        }
    }

    #[task(binds = EXTI4_15, priority = 2, resources = [BUTTON, EXT], spawn = [button_event])]
    fn exti4_15(cx: exti4_15::Context) {
        cx.resources.EXT.clear_irq(cx.resources.BUTTON.pin_number());
        cx.spawn.button_event().unwrap();
    }

    #[task(binds = TIM2, priority = 1, resources = [BUZZER, STATE, TIMER_PWM, BUZZER_ON, PWM_ON])]
    fn tim2(cx: tim2::Context) {
        cx.resources.TIMER_PWM.clear_irq();

        if *cx.resources.STATE && *cx.resources.PWM_ON {
            if *cx.resources.BUZZER_ON {
                *cx.resources.BUZZER_ON = false;
                cx.resources.BUZZER.set_high().unwrap();
            } else {
                *cx.resources.BUZZER_ON = true;
                cx.resources.BUZZER.set_low().unwrap();
            }
        }
    }

    #[task(binds = TIM3, priority = 1, resources = [TIMER_PWM, TIMER_INTERVAL, STATE, PWM_ON])]
    fn tim3(cx: tim3::Context) {
        cx.resources.TIMER_INTERVAL.clear_irq();
        *cx.resources.PWM_ON = !*cx.resources.PWM_ON;
    }

    #[task(priority = 1, resources = [BUZZER, STATE])]
    fn button_event(cx: button_event::Context) {
        if *cx.resources.STATE {
            *cx.resources.STATE = false;
        } else {
            *cx.resources.STATE = true;
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};
