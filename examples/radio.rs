#![cfg_attr(not(test), no_std)]
#![no_main]
#![allow(deprecated)]

use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use longfi_device::{AntPinsMode, Spi};
use nb::block;
use stm32l0xx_hal as hal;
use stm32l0xx_hal::gpio::gpioa::*;
use stm32l0xx_hal::gpio::{Floating, Input, Output, PushPull};
use stm32l0xx_hal::pac::SPI1;

mod longfi_bindings {
    pub struct AntennaSwitches<Rx, TxRfo, TxBoost> {
        rx: Rx,
        tx_rfo: TxRfo,
        tx_boost: TxBoost,
    }

    impl<Rx, TxRfo, TxBoost> AntennaSwitches<Rx, TxRfo, TxBoost>
    where
        Rx: embedded_hal::digital::v2::OutputPin,
        TxRfo: embedded_hal::digital::v2::OutputPin,
        TxBoost: embedded_hal::digital::v2::OutputPin,
    {
        pub fn new(rx: Rx, tx_rfo: TxRfo, tx_boost: TxBoost) -> AntennaSwitches<Rx, TxRfo, TxBoost> {
            AntennaSwitches {
                rx,
                tx_rfo,
                tx_boost,
            }
        }

        pub fn set_sleep(&mut self) {
            self.rx.set_low().ok();
            self.tx_rfo.set_low().ok();
            self.tx_boost.set_low().ok();
        }

        pub fn set_tx(&mut self) {
            self.rx.set_low().ok();
            self.tx_rfo.set_low().ok();
            self.tx_boost.set_high().ok();
        }

        pub fn set_rx(&mut self) {
            self.rx.set_high().ok();
            self.tx_rfo.set_low().ok();
            self.tx_boost.set_low().ok();
        }
    }

    type AntSw = AntennaSwitches<
        stm32l0xx_hal::gpio::gpioa::PA1<stm32l0xx_hal::gpio::Output<stm32l0xx_hal::gpio::PushPull>>,
        stm32l0xx_hal::gpio::gpioc::PC2<stm32l0xx_hal::gpio::Output<stm32l0xx_hal::gpio::PushPull>>,
        stm32l0xx_hal::gpio::gpioc::PC1<stm32l0xx_hal::gpio::Output<stm32l0xx_hal::gpio::PushPull>>,
    >;

    static mut ANT_SW: Option<AntSw> = None;

    pub fn set_antenna_switch(pin: AntSw) {
        unsafe {
            ANT_SW = Some(pin);
        }
    }

    pub extern "C" fn set_antenna_pins(mode: AntPinsMode, _power: u8) {
        unsafe {
            if let Some(ant_sw) = &mut ANT_SW {
                match mode {
                    AntPinsMode::AntModeTx => {
                        ant_sw.set_tx();
                    }
                    AntPinsMode::AntModeRx => {
                        ant_sw.set_rx();
                    }
                    AntPinsMode::AntModeSleep => {
                        ant_sw.set_sleep();
                    }
                    _ => (),
                }
            }
        }
    }

    static mut EN_TCXO: Option<stm32l0xx_hal::gpio::gpioa::PA8<Output<PushPull>>> = None;
    pub fn set_tcxo_pins(pin: stm32l0xx_hal::gpio::gpioa::PA8<Output<PushPull>>) {
        unsafe {
            EN_TCXO = Some(pin);
        }
    }

    #[no_mangle]
    pub extern "C" fn set_tcxo(value: bool) -> u8 {
        unsafe {
            if let Some(pin) = &mut EN_TCXO {
                if value {
                    pin.set_high().unwrap();
                } else {
                    pin.set_high().unwrap();
                }
            }
        }
        6
    }

    #[no_mangle]
    pub extern "C" fn spi_in_out(s: *mut Spi, out_data: u8) -> u8 {
        let spi: &mut hal::spi::Spi<
            SPI1,
            (
                PA3<Input<Floating>>,
                PA6<Input<Floating>>,
                PA7<Input<Floating>>,
            ),
        > = unsafe {
            &mut *((*s).Spi.Instance
                as *mut hal::spi::Spi<
                    SPI1,
                    (
                        PA3<Input<Floating>>,
                        PA6<Input<Floating>>,
                        PA7<Input<Floating>>,
                    ),
                >)
        };

        spi.send(out_data).unwrap();
        let in_data = block!(spi.read()).unwrap();

        in_data
    }

    static mut SPI_NSS: Option<stm32l0xx_hal::gpio::gpioa::PA15<Output<PushPull>>> = None;

    pub fn set_spi_nss(pin: stm32l0xx_hal::gpio::gpioa::PA15<Output<PushPull>>) {
        unsafe {
            SPI_NSS = Some(pin);
        }
    }

    #[no_mangle]
    pub extern "C" fn spi_nss(value: bool) {
        unsafe {
            if let Some(pin) = &mut SPI_NSS {
                if value {
                    pin.set_high().unwrap();
                } else {
                    pin.set_low().unwrap();
                }
            }
        }
    }
    static mut RESET: Option<stm32l0xx_hal::gpio::gpioc::PC0<Output<PushPull>>> = None;

    pub fn set_radio_reset(pin: stm32l0xx_hal::gpio::gpioc::PC0<Output<PushPull>>) {
        unsafe {
            RESET = Some(pin);
        }
    }

    #[no_mangle]
    pub extern "C" fn radio_reset(value: bool) {
        unsafe {
            if let Some(pin) = &mut RESET {
                if value {
                    pin.set_low().unwrap();
                } else {
                    pin.set_high().unwrap();
                }
            }
        }
    }

    #[no_mangle]
    pub extern "C" fn delay_ms(ms: u32) {
        cortex_m::asm::delay(ms);
    }

    #[no_mangle]
    pub extern "C" fn get_random_bits(_bits: u8) -> u32 {
        0x1
    }
}

extern crate panic_semihosting;

use hal::{
    exti::TriggerEdge,
    gpio::*,
    pac,
    prelude::*,
    rcc::Config,
    spi,
    syscfg,
};
use longfi_bindings::AntennaSwitches;
use longfi_device;
use longfi_device::LongFi;
use longfi_device::{ClientEvent, RfConfig, RfEvent};
use stm32l0xx_hal as hal;
use communicator::{Message, Channel};
use heapless::consts::*;

#[rtfm::app(device = stm32l0xx_hal::pac)]
const APP: () = {
    static mut INT: pac::EXTI = ();
    static mut SX1276_DIO0: gpiob::PB4<Input<PullUp>> = ();
    static mut BUFFER: [u8; 512] = [0; 512];
    static mut LONGFI: LongFi = ();
    static mut LED: gpiob::PB2<Output<PushPull>> = ();
    static mut STATE: bool = false;
    static mut COUNTER_1: u32 = 0;
    static mut COUNTER_2: u32 = 0;

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

        let en_tcxo = gpioa.pa8.into_push_pull_output();
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

        // Configure PB5 as output.
        let mut led = gpiob.pb2.into_push_pull_output();
        led.set_low().ok();

        // Return the initialised resources.
        init::LateResources {
            INT: exti,
            SX1276_DIO0: sx1276_dio0,
            LONGFI: longfi_radio,
            LED: led,
        }
    }

    #[task(capacity = 4, priority = 2, resources = [BUFFER, LONGFI, LED, STATE, COUNTER_1, COUNTER_2])]
    fn radio_event(event: RfEvent) {
        let longfi_radio = resources.LONGFI;
        let client_event = longfi_radio.handle_event(event);
        match client_event {
            ClientEvent::ClientEvent_TxDone => {
                longfi_radio.receive();
            }
            ClientEvent::ClientEvent_Rx => {
                let rx_packet = longfi_radio.get_rx();

                {
                    let buf = unsafe {
                        core::slice::from_raw_parts(rx_packet.buf, rx_packet.len as usize)
                    };
                    let message = Message::deserialize(buf);

                    if let Some(message) = message {
                        // Let's assume we only have permission to use ID 2:
                        if message.id != 2 {
                            longfi_radio.set_buffer(resources.BUFFER);
                            longfi_radio.receive();
                            return;
                        }

                        let binary = application(
                            message,
                            resources.COUNTER_1,
                            resources.COUNTER_2,
                            resources.STATE,
                            resources.LED
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
    led: &mut gpiob::PB2<Output<PushPull>>,
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

    // toggle the LED
    if *state {
        led.set_low().unwrap();
        *state = false;
    } else {
        led.set_high().unwrap();
        *state = true;
    }

    binary
}