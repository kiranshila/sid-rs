#![no_std]
#![allow(dead_code)]

// Bring in our register definitions as a module
mod registers;
mod state;

use registers::{FilterRegister, VoiceRegister};
pub use state::*;

use embedded_hal as hal;

// SPI for the data transfer and a few chip control lines
use hal::blocking::spi;
use hal::digital::v2::OutputPin;

// Delay
use hal::blocking::delay::DelayMs;

// SID Chip Constants
const SID_FREQ: u32 = 1_000_000;
const NUM_VOICES: usize = 3;

/// SID Chip, as represented by its interface
pub struct Sid<SPI, CS, RES, DELAY> {
    spi: SPI,
    cs: CS,
    reset: RES,
    delay: DELAY,
    state: SIDState,
}

impl<SPI, CS, RES, E, PinError, DELAY> Sid<SPI, CS, RES, DELAY>
where
    SPI: spi::Write<u8, Error = E>,
    CS: OutputPin<Error = PinError>,
    RES: OutputPin<Error = PinError>,
    DELAY: DelayMs<u16>,
{
    /// Returns a new `SID` instance with the default initial values
    pub fn new(spi: SPI, cs: CS, reset: RES, delay: DELAY) -> Result<Self, E> {
        Ok(Sid {
            spi,
            cs,
            reset,
            delay,
            state: SIDState::new(),
        })
    }
    // Low Level SPI interface
    fn write_reg(&mut self, addr: u8, value: u8) {
        let bytes = [addr & 0x1F, value];
        self.cs.set_low().ok();
        self.spi.write(&bytes).ok();
        self.cs.set_high().ok();
    }
    fn write_regs(&mut self, start_addr: u8, values: &[u8]) {
        for (i, value) in values.iter().enumerate() {
            self.write_reg(start_addr + (i as u8), *value);
        }
    }
    /// Resets the chip by cycling its reset line
    pub fn reset(&mut self) {
        // From the datasheet, hold ~RES low for 10 cycles
        self.reset.set_low().ok();
        self.delay.delay_ms(10);
        self.reset.set_high().ok();
    }
    // Actual interface
    pub fn write_filter(&mut self) {
        self.write_regs(
            FilterRegister::CutoffLow.addr(), // Starting position
            &self.state.filter().payload(),
        )
    }
    pub fn write_voice(&mut self, voice: usize) {
        self.write_regs(
            VoiceRegister::Freq.addr(voice), // Starting position
            &self.state.voice(voice).payload(),
        )
    }
    pub fn write_voices(&mut self) {
        for i in 0..NUM_VOICES {
            self.write_voice(i);
        }
    }
    pub fn initialize(&mut self) {
        self.write_voices();
        self.write_filter();
    }
}
