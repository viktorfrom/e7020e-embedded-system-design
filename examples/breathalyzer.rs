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
        EXT: pac::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        HEATER: gpioa::PA5<Output<PushPull>>,
        DAT: gpioa::PA2<Analog>,
        ADC: adc::Adc,
        TIMER: timer::Timer<pac::TIM2>,
        #[init(false)]
        BREATHALYZER_ON: bool
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

        // Configure timer
        let mut tim2 = timer::Timer::tim2(cx.device.TIM2, 1000.ms(), &mut rcc);
        tim2.listen();

        // Configure ADC2 (PA2)
        let adc = adc::Adc::new(cx.device.ADC, &mut rcc);

        // Configure breathalyzer pins
        let mut heater = gpioa.pa5.into_push_pull_output();
        let dat = gpioa.pa2.into_analog();

        // Configure button input
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

        // Start heating the alchohol sensor (needs warmup)
        heater.set_low();

        init::LateResources {
            EXT: exti,
            BUTTON: button,
            HEATER: heater,
            DAT: dat,
            ADC: adc,
            TIMER: tim2
        }
    }

    #[task(binds = EXTI4_15, priority = 2, resources = [BUTTON, EXT, BREATHALYZER_ON])]
    fn exti4_15(cx: exti4_15::Context) {
        cx.resources.EXT.clear_irq(cx.resources.BUTTON.pin_number());
        *cx.resources.BREATHALYZER_ON = !*cx.resources.BREATHALYZER_ON;
    }

    #[task(binds = TIM2, priority = 2, resources = [ADC, DAT, BREATHALYZER_ON])]
    fn breathalyzer(cx: breathalyzer::Context) {
        if *cx.resources.BREATHALYZER_ON {
            let value: u16 = cx.resources.ADC.read(cx.resources.DAT).unwrap();
            hprintln!("Value: {:#}", value).unwrap();            
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};