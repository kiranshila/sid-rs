#![no_std]
#![allow(dead_code)]

// Bring in our register definitions as a module
mod state;

use core::u8;

pub use state::*;

// Bring in the flagset crate
extern crate flagset;

// Bring in the hal traits
use embedded_hal as hal;

// SPI for the data transfer and a few chip control lines
use hal::blocking::spi;
use hal::digital::v2::OutputPin;

// Delay
use hal::blocking::delay::DelayUs;

// SID Chip Constants
const FREQ: u32 = 1_000_000;
const VOICE_REG_START: u8 = 0;
const VOICE_REG_OFFSET: u8 = 7;
const FILTER_REG_START: u8 = 21;
const NUM_VOICES: usize = 3;

/// SID Chip, as represented by its interface
pub struct Sid<SPI, CS, DELAY> {
    spi: SPI,
    cs: CS,
    delay: DELAY,
    state: SidState,
}

impl<SPI, CS, E, PinError, DELAY> Sid<SPI, CS, DELAY>
where
    SPI: spi::Write<u8, Error = E>,
    CS: OutputPin<Error = PinError>,
    DELAY: DelayUs<u16>,
{
    /// Returns a new `SID` instance with the default initial values
    pub fn new(spi: SPI, cs: CS, delay: DELAY) -> Result<Self, E> {
        Ok(Sid {
            spi,
            cs,
            delay,
            state: Default::default(),
        })
    }
    // Low Level SPI interface
    fn write_reg(&mut self, addr: u8, value: u8) {
        // Ensure the reset bit is 0
        // Format is [ADDR RES DATA 0 0]
        let bytes = [addr << 1, value];
        self.cs.set_low().ok();
        self.spi.write(&bytes).ok();
        self.cs.set_high().ok();
        // Delay for 2 clock cycles to make sure the SID got it
        self.delay.delay_us(2);
    }
    fn write_regs(&mut self, start_addr: u8, values: &[u8]) {
        for (i, value) in values.iter().enumerate() {
            self.write_reg(start_addr + i as u8, *value);
        }
    }
    /// Resets the chip by cycling its reset line
    pub fn reset(&mut self) {
        // Ensure the reset bit is 1
        let bytes = [1u8, 0u8];
        self.cs.set_low().ok();
        self.spi.write(&bytes).ok();
        // From the datasheet, hold ~RES low for 10 cycles
        // Make it 12, to ensure we didn't hit it in between cycles
        self.delay.delay_us(12);
        self.cs.set_high().ok();
    }
    // Actual interface
    pub fn write_filter(&mut self) {
        // Create buffer
        let mut buf = [0u8; 4];
        // Fill contents
        self.state.filter.payload(&mut buf);
        self.write_regs(FILTER_REG_START, &buf)
    }
    pub fn write_voice(&mut self, voice: usize) {
        // Create buffer
        let mut buf = [0u8; 7];
        // Fill contents
        self.state.voices[voice].payload(&mut buf);
        self.write_regs(VOICE_REG_START + voice as u8 * VOICE_REG_OFFSET, &buf)
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
