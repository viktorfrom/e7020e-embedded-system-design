use embedded_hal::digital::v2::OutputPin;
use embedded_hal::spi::FullDuplex;
use longfi_device::{AntPinsMode, Spi};
use nb::block;
use stm32l0xx_hal as hal;
use stm32l0xx_hal::gpio::gpioa::*;
use stm32l0xx_hal::gpio::{Floating, Input, Output, PushPull};
use stm32l0xx_hal::pac::SPI1;

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

static mut EN_TCXO: Option<stm32l0xx_hal::gpio::gpioa::PB5<Output<PushPull>>> = None;
pub fn set_tcxo_pins(pin: stm32l0xx_hal::gpio::gpioa::PB5<Output<PushPull>>) {
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
