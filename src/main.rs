//#![cfg_attr(not(test), no_std)]
#![no_main]
#![no_std]

mod breathalyzer;
mod buzzer;
mod longfi_bindings;
mod oled;

use longfi_bindings::AntennaSwitches;
use longfi_device::{self, ClientEvent, LongFi, RfConfig, RfEvent};
use communicator::{Message, Channel};
use heapless::consts::*;
use core::str::from_utf8;

use crate::breathalyzer::{Breathalyzer, BAC};
use crate::buzzer::Buzzer;
use crate::oled::Oled;

use stm32l0xx_hal as hal;

//#[cfg(not(debug_assertions))]
//use panic_halt as _;

// Debug imports
#[cfg(debug_assertions)]
extern crate panic_semihosting;
#[cfg(debug_assertions)]
use cortex_m_semihosting::hprintln;


use stm32l0xx_hal::{
    adc,
    delay::Delay,
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
        #[init(0)]
        COUNT: u16,
        #[init(false)]
        BUZZER_ON: bool,
        #[init(false)]
        MEASURING: bool,
        #[init(0)]
        TIME_COUNTER: u16,

        EXT: pac::EXTI,
        BUTTON: gpiob::PB2<Input<PullUp>>,
        TIMER_BREATH: timer::Timer<pac::TIM2>,
        TIMER_PWM: timer::Timer<pac::TIM3>,
        TIMER_SEC: timer::Timer<pac::TIM21>,
        TIMER_WARM_UP: timer::Timer<pac::TIM22>,
        BREATHALYZER: Breathalyzer,
        BUZZER: Buzzer,
        LONGFI: LongFi,
        RADIO_EXTI: gpiob::PB4<Input<PullUp>>,
        OLED: Oled,
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
        let button = gpiob.pb2.into_pull_up_input();
        let radio_int = gpiob.pb4.into_pull_up_input();

        // Configure timers
        let mut tim2 = timer::Timer::tim2(cx.device.TIM2, 1000.ms(), &mut rcc);
        let mut tim3 = timer::Timer::tim3(cx.device.TIM3, 1000.hz(), &mut rcc);
        let mut tim21 = timer::Timer::tim21(cx.device.TIM21, 1000.ms(), &mut rcc);
        let mut tim22 = timer::Timer::tim22(cx.device.TIM22, 1000.ms(), &mut rcc);

        // External interrupt
        let exti = cx.device.EXTI;

        // Configure interrupts
        exti.listen(
            &mut syscfg,
            radio_int.port(),
            radio_int.pin_number(),
            TriggerEdge::Rising,
        );

        exti.listen(
            &mut syscfg,
            button.port(),
            button.pin_number(),
            TriggerEdge::Falling,
        );

        tim2.listen();
        tim3.listen();
        tim21.listen();
        tim22.listen();

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
        let mut delay = Delay::new(cx.core.SYST, rcc.clocks);

        // Initialise the SPI peripheral.
        let mut spi =
            cx.device
                .SPI2
                .spi((sck, NoMiso, mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc);

        // Initialize modules
        let mut buzzer = Buzzer::new(gpioa.pa3);
        let mut breathalyzer = Breathalyzer::new(gpioa.pa5, gpioa.pa2, adc);
        breathalyzer.on();
        let mut oled = Oled::new(spi, gpiob.pb8, gpiob.pb9, delay);

        let test: f32 = 0.5;

        // Return the initialised resources.
        init::LateResources {
            EXT: exti,
            BUTTON: button,
            TIMER_BREATH: tim2,
            TIMER_PWM: tim3,
            TIMER_SEC: tim21,
            TIMER_WARM_UP: tim22,
            BREATHALYZER: breathalyzer,
            BUZZER: buzzer,
            LONGFI: longfi_radio,
            RADIO_EXTI: radio_int,
            OLED: oled,
        }
    }

    // External interrupt for the button
    #[task(binds = EXTI2_3, priority = 3, spawn = [button_event], resources = [EXT, BUTTON])]
    fn exti2_3(cx: exti2_3::Context) {
        hprintln!("exti2_3").unwrap();
        cx.resources.EXT.clear_irq(cx.resources.BUTTON.pin_number());
        cx.spawn.button_event().unwrap();
    }

    // External interrupt for the radio
    #[task(binds = EXTI4_15, priority = 3, spawn = [radio_event], resources = [EXT, RADIO_EXTI])]
    fn exti4_15(cx: exti4_15::Context) {
        hprintln!("exti4_15").unwrap();
        cx.resources.EXT.clear_irq(cx.resources.RADIO_EXTI.pin_number());
        cx.spawn.radio_event(RfEvent::DIO0).unwrap();
    }

    #[task(capacity = 4, priority = 2, spawn = [button_event], resources = [BUFFER, LONGFI])]
    fn radio_event(cx: radio_event::Context, event: RfEvent) {
        let mut longfi_radio = cx.resources.LONGFI;
        let client_event = longfi_radio.handle_event(event);

        match client_event {
            ClientEvent::ClientEvent_TxDone => {
                longfi_radio.receive();
            },
            ClientEvent::ClientEvent_Rx => {
                let rx_packet = longfi_radio.get_rx();

                {
                    let buf = unsafe {
                        core::slice::from_raw_parts(rx_packet.buf, rx_packet.len as usize)
                    };

                    let message = Message::deserialize(buf);

                    if let Some(message) = message {
                        // Let's assume we only have permission to use ID 6:
                        hprintln!("hi").unwrap();
                        if message.id == 6 {
                            cx.spawn.button_event().unwrap();
                        }
                    }
                }

                longfi_radio.set_buffer(cx.resources.BUFFER);
                longfi_radio.receive();
            }
            ClientEvent::ClientEvent_None => {}
        }
    }

    #[task(priority = 2, resources = [LONGFI])]
    fn send_radio_message(cx: send_radio_message::Context, bac: BAC) {
        let bac_convert = match bac {
            BAC::NONE => 0,
            BAC::LOW => 1,
            BAC::MEDIUM => 2,
            BAC::HIGH => 3,
            BAC::VERY_HIGH => 4,
            BAC::DEATH => 5
        };

        // Wrap the message in a way such that ThingsBoard can handle it
        let message = Message {
            id: 6,
            data: bac_convert,
            channel: Channel::One
        };
    
        let binary = Message::serialize(&message).unwrap();
        cx.resources.LONGFI.send(&binary);
    }

    // Handles the button press
    #[task(priority = 2, spawn = [send_radio_message], resources = [BUTTON, BUZZER, BREATHALYZER, OLED, MEASURING, TIMER_SEC])]
    fn button_event(cx: button_event::Context) {
        let mut value: BAC = BAC::NONE;

        if !*cx.resources.MEASURING {
            if cx.resources.BUZZER.enabled {
                cx.resources.BUZZER.disable();
                value = cx.resources.BREATHALYZER.read();

                let val = match value {
                    BAC::NONE => "NONE",
                    BAC::LOW => "LOW",
                    BAC::MEDIUM => "MEDIUM",
                    BAC::HIGH => "HIGH",
                    BAC::VERY_HIGH => "VERY HIGH",
                    BAC::DEATH => "DEATH"
                };
                cx.resources.OLED.on(val);

                // Send radio message, place this wherever it is needed
                cx.spawn.send_radio_message(value).unwrap();
            } else {
                cx.resources.OLED.on("Reading");
                // constant beep
                cx.resources.BUZZER.enable();
                cx.resources.TIMER_SEC.listen();
                *cx.resources.MEASURING = true;
            }
        }
    }

    // Polls the alcohol sensor
    #[task(binds = TIM2, priority = 2, resources = [TIMER_BREATH, BREATHALYZER])]
    fn sensor_poll(mut cx: sensor_poll::Context) {
        cx.resources.TIMER_BREATH.clear_irq();
        let val = cx.resources.BREATHALYZER.read_curr();

        if !(val < cx.resources.BREATHALYZER.curr_val) {
            cx.resources.BREATHALYZER.curr_val = cx.resources.BREATHALYZER.read_curr();
        }
    }

    // Toggles the buzzer's PWM according to the set frequency
    #[task(binds = TIM3, resources = [BUZZER, TIMER_PWM])]
    fn buzzer_pwm(mut cx: buzzer_pwm::Context) {
        cx.resources.TIMER_PWM.clear_irq();
        cx.resources.BUZZER.lock(|BUZZER| BUZZER.toggle_pwm());
    }

    #[task(binds = TIM21, spawn = [button_event], resources = [BUZZER, TIME_COUNTER, TIMER_SEC, MEASURING])]
    fn stop_measuring(mut cx: stop_measuring::Context) {
        cx.resources.TIMER_SEC.lock(|TIMER_SEC| TIMER_SEC.clear_irq());
        let mut measuring: bool = cx.resources.MEASURING.lock(|MEASURING| return *MEASURING);
        let mut counter: u16 = *cx.resources.TIME_COUNTER;
        let mut complete: bool = false;

        if measuring {
            if counter >= 3 {
                cx.resources.MEASURING.lock(|MEASURING| {
                    *MEASURING = false;
                });
                complete = true;
            } else {
                counter += 1;
                *cx.resources.TIME_COUNTER = counter;
            }
        }

        if complete {
            *cx.resources.TIME_COUNTER = 0;
            cx.spawn.button_event().unwrap();
        }
    }

    // Device warm up
    #[task(binds = TIM22, priority = 2, resources = [OLED, BREATHALYZER, COUNT, TIMER_WARM_UP])]
    fn warm_up(cx: warm_up::Context) {
        cx.resources.TIMER_WARM_UP.clear_irq();

        if *cx.resources.COUNT != 10 {
            cx.resources.OLED.on("Warming up");
            *cx.resources.COUNT += 1;
        } else {
            cx.resources.OLED.on("Ready");
            cx.resources.TIMER_WARM_UP.unlisten();
            *cx.resources.COUNT = 0;
        }
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART1();
        fn USART2();
        fn USART4_USART5();
    }
};
