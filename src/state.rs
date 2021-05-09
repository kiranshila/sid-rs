use core::convert::TryInto;
use flagset::{flags, FlagSet};
use ux::{u11, u12, u4};

flags! {
    pub enum VoiceShape:u8 {
        Triangle = 0b0001_0000,
        Sawtooth = 0b0010_0000,
        Square   = 0b0100_0000,
        Noise    = 0b1000_0000,
    }
}

flags! {
    pub enum FilterKind:u8 {
        LowPass  = 0b0001_0000,
        BandPass = 0b0010_0000,
        HighPass = 0b0100_0000,
        ThreeOff = 0b1000_0000,
    }
}

#[derive(Copy, Clone)]
pub struct Envelope {
    attack: u4,
    decay: u4,
    sustain: u4,
    release: u4,
}

impl Default for Envelope {
    fn default() -> Self {
        Envelope {
            attack: u4::new(0),
            decay: u4::new(0),
            sustain: u4::new(7),
            release: u4::new(0),
        }
    }
}

flags! {
    pub enum ControlFlag:u8 {
        Gate,
        Sync,
        RingMod,
        Test,
    }
}

#[derive(Copy, Clone)]
pub struct Control {
    shapes: FlagSet<VoiceShape>,
    flags: FlagSet<ControlFlag>,
}

impl Default for Control {
    fn default() -> Self {
        Control {
            shapes: VoiceShape::Square.into(),
            flags: Default::default(),
        }
    }
}

#[derive(Copy, Clone)]
pub struct Voice {
    frequency: u16,
    pwm: u12,
    envelope: Envelope,
    control: Control,
}

impl Default for Voice {
    fn default() -> Self {
        Voice {
            frequency: 7217,
            pwm: u12::new(2048),
            envelope: Default::default(),
            control: Default::default(),
        }
    }
}

flags! {
    pub enum FilterTarget:u8 {
        Voice1,
        Voice2,
        Voice3,
        External,
    }
}

#[derive(Copy, Clone)]
pub struct Filter {
    frequency: u11,
    resonance: u4,
    volume: u4,
    kinds: FlagSet<FilterKind>,
    targets: FlagSet<FilterTarget>,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            frequency: Default::default(),
            resonance: Default::default(),
            volume: u4::new(7),
            kinds: Default::default(),
            targets: Default::default(),
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct SidState {
    pub voices: [Voice; 3],
    pub filter: Filter,
}

impl SidState {
    // Creates a new SID chip state with sane default values
    pub fn new() -> Self {
        Default::default()
    }
}

pub trait Payload {
    type Output;
    fn payload(&self, buf: &mut Self::Output);
}

impl Payload for Control {
    type Output = [u8; 1];
    fn payload(&self, buf: &mut Self::Output) {
        buf[0] = self.shapes.bits() | self.flags.bits();
    }
}

impl Payload for Envelope {
    type Output = [u8; 2];
    fn payload(&self, buf: &mut Self::Output) {
        buf[0] = (u8::from(self.attack) << 4) | u8::from(self.decay);
        buf[1] = (u8::from(self.sustain) << 4) | u8::from(self.release);
    }
}

impl Payload for Voice {
    type Output = [u8; 7];
    fn payload(&self, buf: &mut Self::Output) {
        // Creates a byte array for sending over SPI
        buf[0] = (self.frequency & 0xFF) as u8;
        buf[1] = (self.frequency >> 8) as u8;
        buf[2] = (u16::from(self.pwm) & 0x00FF) as u8;
        buf[3] = ((u16::from(self.pwm) & 0x0F00) >> 8) as u8;
        self.control.payload(&mut buf[4..5].try_into().unwrap());
        self.envelope.payload(&mut buf[5..6].try_into().unwrap());
    }
}

impl Payload for Filter {
    type Output = [u8; 4];
    fn payload(&self, buf: &mut Self::Output) {
        buf[0] = (u16::from(self.frequency) & 0x7) as u8;
        buf[1] = ((u16::from(self.frequency) & 0x7F8) >> 3) as u8;
        buf[2] = u8::from(self.resonance) << 4 | self.targets.bits();
        buf[3] = self.kinds.bits() | u8::from(self.volume);
    }
}
