#![cfg_attr(not(test), no_std)]
#![no_main]
#![allow(deprecated)]

mod longfi_bindings;

extern crate panic_semihosting;

use hal::{
    exti::TriggerEdge,
    gpio::*,
    pac,
    prelude::*,
    rcc::Config,
    spi,
    syscfg,
    timer::Timer
};
use longfi_bindings::AntennaSwitches;
use longfi_device;
use longfi_device::LongFi;
use longfi_device::{ClientEvent, RfConfig, RfEvent};
use stm32l0xx_hal as hal;
use communicator::{Message, Channel};
use heapless::consts::*;

use cortex_m_semihosting::hprintln;

#[rtfm::app(device = stm32l0xx_hal::pac)]
const APP: () = {
    static mut INT: pac::EXTI = ();
    static mut SX1276_DIO0: gpiob::PB4<Input<PullUp>> = ();
    static mut BUFFER: [u8; 512] = [0; 512];
    static mut LONGFI: LongFi = ();
    static mut STATE: bool = false;
    static mut COUNTER_1: u32 = 0;
    static mut COUNTER_2: u32 = 0;

    static mut UPLINK_TIMER: Timer<pac::TIM2> = ();

    #[init(resources = [BUFFER])]
    fn init() -> init::LateResources {
        // Configure the clock.
        let mut rcc = device.RCC.freeze(Config::hsi16());
        let mut syscfg = syscfg::SYSCFG::new(device.SYSCFG, &mut rcc);

        // Acquire the GPIOB peripheral. This also enables the clock for GPIOB in
        // the RCC register.
        let gpioa = device.GPIOA.split(&mut rcc);
        let gpiob = device.GPIOB.split(&mut rcc);
        let gpioc = device.GPIOC.split(&mut rcc);

        let exti = device.EXTI;

        // Configure PB4 as input.
        let sx1276_dio0 = gpiob.pb4.into_pull_up_input();
        // Configure the external interrupt on the falling edge for the pin 2.
        exti.listen(
            &mut syscfg,
            sx1276_dio0.port(),
            sx1276_dio0.pin_number(),
            TriggerEdge::Rising,
        );

        let sck = gpiob.pb3;
        let miso = gpioa.pa6;
        let mosi = gpioa.pa7;
        let nss = gpioa.pa15.into_push_pull_output();
        longfi_bindings::set_spi_nss(nss);

        // Initialise the SPI peripheral.
        let mut _spi = device
            .SPI1
            .spi((sck, miso, mosi), spi::MODE_0, 1_000_000.hz(), &mut rcc);

        let reset = gpioc.pc0.into_push_pull_output();
        longfi_bindings::set_radio_reset(reset);

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

        longfi_radio.set_buffer(resources.BUFFER);

        longfi_radio.receive();

        let mut uplink_timer = device.TIM2.timer(1.hz(), &mut rcc);
        //uplink_timer.listen();

        // Return the initialised resources.
        init::LateResources {
            INT: exti,
            SX1276_DIO0: sx1276_dio0,
            LONGFI: longfi_radio,
            UPLINK_TIMER: uplink_timer
        }
    }

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

    #[interrupt(priority = 1, resources = [SX1276_DIO0, INT], spawn = [radio_event])]
    fn EXTI4_15() {
        resources.INT.clear_irq(resources.SX1276_DIO0.pin_number());
        spawn.radio_event(RfEvent::DIO0).unwrap();
    }

    // This should not be a separate task from TIM2
    #[task(capacity = 4, priority = 2, resources = [LONGFI])]
    fn send_ping() {
        static mut STATE: bool = false;
        hprintln!("pinging...").unwrap();
        let packet: [u8; 16] = [
            0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xa6, 0xa7, 0xa8, 0xa9, 0xaa, 0xab, 0xac, 0xad, 0xae, 0xaf,
        ];

        resources.LONGFI.send(&packet);
    }

    #[interrupt(resources = [UPLINK_TIMER], spawn = [send_ping])]
    fn TIM2() {
        resources.UPLINK_TIMER.clear_irq();
        spawn.send_ping().ok();
    }

    // Interrupt handlers used to dispatch software tasks
    extern "C" {
        fn USART4_USART5();
    }
};

// Example application: increment counter:
fn application(
    message: Message,
    counter_1: &mut u32,
    counter_2: &mut u32,
    state: &mut bool,
) -> heapless::Vec<u8, U90> {
    let data = if let Channel::One = message.channel {
        *counter_1 += message.data;
        *counter_1
    } else {
        *counter_2 += message.data;
        *counter_2
    };

    let response = Message {
        id: message.id,
        channel: message.channel,
        data,
    };
    let binary = Message::serialize(&response).unwrap();

    binary
}
