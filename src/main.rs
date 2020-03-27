//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

mod breathalyzer;
mod buzzer;
mod longfi_bindings;
//mod oled;

extern crate panic_semihosting;

use longfi_bindings::AntennaSwitches;
use longfi_device::{self, ClientEvent, LongFi, RfConfig, RfEvent};
use communicator::{Message, Channel};
use heapless::consts::*;

use crate::breathalyzer::Breathalyzer;
use crate::buzzer::Buzzer;
//use crate::oled::Oled;
use cortex_m::peripheral::DWT;
use stm32l0xx_hal as hal;
// hprintln is very resource demanding, only use for testing non-time critical things!
//use cortex_m_semihosting::hprintln;

use stm32l0xx_hal::{
    adc,
    exti::TriggerEdge,
    gpio::*,
    pac,
    prelude::*,
    rcc::Config,
    spi::{self, Mode, NoMiso, Phase, Polarity},
    syscfg, 
    timer,
};

#[rtfm::app(device = stm32l0xx_hal::pac, peripherals = true)]
const APP: () = {
    struct Resources {
        #[init([0; 512])]
        BUFFER: [u8; 512],
        EXT: pac::EXTI,
        BUTTON: gpioa::PA4<Input<PullUp>>,
        TIMER_BREATH: timer::Timer<pac::TIM2>,
        TIMER_PWM: timer::Timer<pac::TIM3>,
        TIMER_PWM_INTERVAL: timer::Timer<pac::TIM21>,
        BREATHALYZER: Breathalyzer,
        BUZZER: Buzzer,
        LONGFI: LongFi,
        RADIO_EXTI: gpiob::PB4<Input<PullUp>>,
        //OLED: Oled,
    }

    #[init(resources = [BUFFER])]
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
        let radio_int = gpiob.pb4.into_pull_up_input();

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

        exti.listen(
            &mut syscfg,
            radio_int.port(),
            radio_int.pin_number(),
            TriggerEdge::Rising,
        );

        tim2.listen();
        tim3.listen();

        // Initialize radio.
        let radio_sck = gpiob.pb3;
        let radio_miso = gpioa.pa6;
        let radio_mosi = gpioa.pa7;
        let radio_nss = gpioa.pa15.into_push_pull_output();
        longfi_bindings::set_spi_nss(radio_nss);

        let mut spi1 = cx.device
            .SPI1
            .spi((radio_sck, radio_miso, radio_mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc);

        let radio_reset = gpioc.pc0.into_push_pull_output();
        longfi_bindings::set_radio_reset(radio_reset);
    
        let ant_sw = AntennaSwitches::new(
            gpioa.pa1.into_push_pull_output(),
            gpioc.pc2.into_push_pull_output(),
            gpioc.pc1.into_push_pull_output(),
        );

        longfi_bindings::set_antenna_switch(ant_sw);

        let en_tcxo = gpiob.pb5.into_push_pull_output();
        longfi_bindings::set_tcxo_pins(en_tcxo);

        static mut BINDINGS: longfi_device::BoardBindings = longfi_device::BoardBindings {
            reset: Some(longfi_bindings::radio_reset),
            spi_in_out: Some(longfi_bindings::spi_in_out),
            spi_nss: Some(longfi_bindings::spi_nss),
            delay_ms: Some(longfi_bindings::delay_ms),
            get_random_bits: Some(longfi_bindings::get_random_bits),
            set_antenna_pins: Some(longfi_bindings::set_antenna_pins),
            set_board_tcxo: Some(longfi_bindings::set_tcxo),
        };

        let rf_config = RfConfig {
            oui: 0xBEEF_FEED,
            device_id: 0xABCD,
        };

        let mut longfi_radio = unsafe { LongFi::new(&mut BINDINGS, rf_config).unwrap() };

        longfi_radio.set_buffer(cx.resources.BUFFER);

        longfi_radio.receive();

        // Initialize OLED
        let mut cs = gpiob.pb12.into_push_pull_output();
        cs.set_low().unwrap(); // not sure if needed, did not try without it

        let sck = gpiob.pb13;
        let mosi = gpiob.pb15;

        // Initialise the SPI peripheral.
        let mut spi =
            cx.device
                .SPI2
                .spi((sck, NoMiso, mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc);

        // Initialize modules
        let mut buzzer = Buzzer::new(gpioa.pa3);
        let mut breathalyzer = Breathalyzer::new(gpioa.pa5, gpioa.pa2, adc);
        //let mut oled = Oled::new(spi);

        // Return the initialised resources.
        init::LateResources {
            EXT: exti,
            BUTTON: button,
            TIMER_BREATH: tim2,
            TIMER_PWM: tim3,
            TIMER_PWM_INTERVAL: tim21,
            BREATHALYZER: breathalyzer,
            BUZZER: buzzer,
            LONGFI: longfi_radio,
            RADIO_EXTI: radio_int
            //OLED: oled,
        }
    }

    //#[task(binds = EXTI4_15)]
    //fn exti4_15(cx: exti4_15::Context) {}

    #[task(capacity = 4, priority = 2, resources = [BUFFER, LONGFI, STATE, COUNTER_1, COUNTER_2])]
    fn radio_event(event: RfEvent) {
        let mut longfi_radio = resources.LONGFI;
        let client_event = longfi_radio.handle_event(event);
        match client_event {
            ClientEvent::ClientEvent_TxDone => {
                hprintln!("transmission done").unwrap();
                longfi_radio.receive();
            }
            ClientEvent::ClientEvent_Rx => {
                let rx_packet = longfi_radio.get_rx();

                {
                    let buf = unsafe {
                        core::slice::from_raw_parts(rx_packet.buf, rx_packet.len as usize)
                    };

                    hprintln!("rx len {}", rx_packet.len).unwrap();

                    hprintln!("before").unwrap();
                    let message = Message::deserialize(buf);
                    
                    if let Some(message) = message {
                        hprintln!("parse").unwrap();
                        // Let's assume we only have permission to use ID 2:
                        if message.id != 2 {
                            longfi_radio.set_buffer(resources.BUFFER);
                            longfi_radio.receive();
                            hprintln!("wrong address").unwrap();
                            return;
                        }

                        hprintln!("sending message").unwrap();
                        let binary = application(
                            message,
                            resources.COUNTER_1,
                            resources.COUNTER_2,
                            resources.STATE,
                        );
                        longfi_radio.send(&binary);
                    }
                }

                longfi_radio.set_buffer(resources.BUFFER);
                longfi_radio.receive();
            }
            ClientEvent::ClientEvent_None => {}
        }
    }

    // Handles the button press
    #[task(binds = EXTI4_15, priority = 2, resources = [BUTTON, EXT, BUZZER, BREATHALYZER, TIMER_PWM_INTERVAL])]
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
    #[task(binds = TIM2, priority = 2, resources = [BREATHALYZER, TIMER_BREATH])]
    fn sensor_poll(cx: sensor_poll::Context) {
        cx.resources.TIMER_BREATH.clear_irq();

        if cx.resources.BREATHALYZER.state {
            let value: u16 = cx.resources.BREATHALYZER.read();
            //hprintln!("Value: {:#}", value).unwrap();
        }
    }

    // Toggles the buzzer's PWM according to the set frequency
    #[task(binds = TIM3, priority = 2, resources = [BUZZER, TIMER_PWM])]
    fn buzzer_pwm(cx: buzzer_pwm::Context) {
        cx.resources.TIMER_PWM.clear_irq();

        if cx.resources.BUZZER.enabled {
            //hprintln!("hi").unwrap();
            cx.resources.BUZZER.toggle_pwm();
        }
    }

    // Toggles buzzer beep intervals
    #[task(binds = TIM21, priority = 2, resources = [BUZZER, TIMER_PWM_INTERVAL])]
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
