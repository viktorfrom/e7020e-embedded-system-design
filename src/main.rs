//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

mod breathalyzer;
mod buzzer;

extern crate panic_semihosting;

use crate::breathalyzer::Breathalyzer;
use crate::buzzer::Buzzer;
use cortex_m::peripheral::DWT;
use stm32l0xx_hal as hal;
// hprintln is very resource demanding, only use for testing non-time critical things!
//use cortex_m_semihosting::hprintln;

use stm32l0xx_hal::{
    adc, exti::TriggerEdge, gpio::*, pac, prelude::*, rcc::Config, spi, syscfg, timer,
};

#[rtfm::app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        EXT: pac::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        TIMER_BREATH: timer::Timer<pac::TIM2>,
        TIMER_PWM: timer::Timer<pac::TIM3>,
        TIMER_PWM_INTERVAL: timer::Timer<pac::TIM21>,
        BREATHALYZER: Breathalyzer,
        BUZZER: Buzzer,
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

        // Configure timers
        let mut tim2 = timer::Timer::tim2(cx.device.TIM2, 1000.ms(), &mut rcc);
        let mut tim3 = timer::Timer::tim3(cx.device.TIM3, 1000.hz(), &mut rcc);
        let mut tim21 = timer::Timer::tim21(cx.device.TIM21, 1000.ms(), &mut rcc);

        // External interrupt
        let exti = cx.device.EXTI;

        // Configure interrupts
        exti.listen(
            &mut syscfg,
            button.port(),
            button.pin_number(),
            TriggerEdge::Falling,
        );

        tim2.listen();
        tim3.listen();

        // Initialize OLED
        // let sck = gpiob.pb3;
        // let miso = gpioa.pa6;
        // let mosi = gpioa.pa7;
        // let nss = gpioa.pa15.into_push_pull_output();

        // Initialize modules
        let mut buzzer = Buzzer::new(gpioa.pa3);
        let mut breathalyzer = Breathalyzer::new(gpioa.pa5, gpioa.pa2, adc);

        // Return the initialised resources.
        init::LateResources {
            EXT: exti,
            BUTTON: button,
            TIMER_BREATH: tim2,
            TIMER_PWM: tim3,
            TIMER_PWM_INTERVAL: tim21,
            BREATHALYZER: breathalyzer,
            BUZZER: buzzer,
        }
    }

    // Handles the button press
    #[task(binds = EXTI4_15, priority = 5, resources = [BUTTON, EXT, BUZZER, BREATHALYZER, TIMER_PWM_INTERVAL])]
    fn button_event(cx: button_event::Context) {
        cx.resources.EXT.clear_irq(cx.resources.BUTTON.pin_number());

        if cx.resources.BUZZER.enabled {
            cx.resources.BUZZER.disable();
            cx.resources.TIMER_PWM_INTERVAL.unlisten();
        } else {
            cx.resources.BUZZER.enable();
            cx.resources.TIMER_PWM_INTERVAL.reset();
            cx.resources.TIMER_PWM_INTERVAL.listen();
        }

        if cx.resources.BREATHALYZER.state {
            cx.resources.BREATHALYZER.off();
        } else {
            cx.resources.BREATHALYZER.on();
        }
    }

    // Polls the alcohol sensor
    #[task(binds = TIM2, priority = 5, resources = [BREATHALYZER, TIMER_BREATH])]
    fn sensor_poll(cx: sensor_poll::Context) {
        cx.resources.TIMER_BREATH.clear_irq();

        if cx.resources.BREATHALYZER.state {
            let value: u16 = cx.resources.BREATHALYZER.read();
            //hprintln!("Value: {:#}", value).unwrap();
        }
    }

    // Toggles the buzzer's PWM according to the set frequency
    #[task(binds = TIM3, priority = 5, resources = [BUZZER, TIMER_PWM])]
    fn buzzer_pwm(cx: buzzer_pwm::Context) {
        cx.resources.TIMER_PWM.clear_irq();

        if cx.resources.BUZZER.enabled {
            //hprintln!("hi").unwrap();
            cx.resources.BUZZER.toggle_pwm();
        }
    }

    // Toggles buzzer beep intervals
    #[task(binds = TIM21, priority = 5, resources = [BUZZER, TIMER_PWM_INTERVAL])]
    fn buzzer_interval(cx: buzzer_interval::Context) {
        cx.resources.TIMER_PWM_INTERVAL.clear_irq();

        if cx.resources.BUZZER.enabled {
            cx.resources.BUZZER.disable();
        } else {
            cx.resources.BUZZER.enable();
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};
