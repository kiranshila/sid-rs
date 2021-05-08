use ux::{u11, u12, u4};

#[derive(Copy, Clone)]
pub enum VoiceShape {
    Square,
    Triangle,
    Sawtooth,
    Noise,
}

#[derive(Copy, Clone)]
pub enum FilterKind {
    ThreeOff,
    HighPass,
    BandPass,
    LowPass,
}

#[derive(Copy, Clone)]
pub struct Envelope {
    attack: u4,
    decay: u4,
    sustain: u4,
    release: u4,
}

#[derive(Copy, Clone)]
pub struct Control {
    shape: VoiceShape,
    test: bool,
    ring_mod: bool,
    sync: bool,
    gate: bool,
}

#[derive(Copy, Clone)]
pub struct Voice {
    frequency: u16,
    pwm: u12,
    envelope: Envelope,
    control: Control,
}

#[derive(Copy, Clone)]
pub struct FilterTargets {
    voice1: bool,
    voice2: bool,
    voice3: bool,
    external: bool,
}

#[derive(Copy, Clone)]
pub struct Filter {
    frequency: u11,
    resonance: u4,
    volume: u4,
    kind: FilterKind,
    targets: FilterTargets,
}

#[derive(Copy, Clone)]
pub struct SIDState {
    voices: [Voice; 3],
    filter: Filter,
}

impl SIDState {
    // Creates a new SID chip state with sane default values
    pub fn new() -> Self {
        let voice = Voice {
            frequency: 7217,
            pwm: u12::new(2048),
            envelope: Envelope {
                attack: u4::new(0),
                decay: u4::new(0),
                sustain: u4::new(7),
                release: u4::new(0),
            },
            control: Control {
                shape: VoiceShape::Square,
                test: false,
                ring_mod: false,
                sync: false,
                gate: false,
            },
        };
        let filter = Filter {
            frequency: u11::new(0),
            resonance: u4::new(0),
            volume: u4::new(7),
            kind: FilterKind::LowPass,
            targets: FilterTargets {
                voice1: false,
                voice2: false,
                voice3: false,
                external: false,
            },
        };
        SIDState {
            voices: [voice, voice, voice],
            filter,
        }
    }
    pub fn filter(&self) -> &Filter {
        &self.filter
    }
    pub fn voice(&self, voice: usize) -> &Voice {
        &self.voices[voice]
    }
}

impl Default for SIDState {
    fn default() -> Self {
        Self::new()
    }
}
impl Control {
    pub fn payload(&self) -> u8 {
        let mut byte: u8 = 0;
        match self.shape {
            VoiceShape::Noise => byte |= 1 << 7,
            VoiceShape::Square => byte |= 1 << 6,
            VoiceShape::Sawtooth => byte |= 1 << 5,
            VoiceShape::Triangle => byte |= 1 << 4,
        }
        if self.test {
            byte |= 1 << 3;
        }
        if self.ring_mod {
            byte |= 1 << 2;
        }
        if self.sync {
            byte |= 1 << 1;
        }
        if self.gate {
            byte |= 1
        }
        byte
    }
}

impl Envelope {
    pub fn payload(&self) -> [u8; 2] {
        let mut ad_byte: u8 = 0;
        ad_byte |= u8::from(self.attack) << 4;
        ad_byte |= u8::from(self.decay);
        let mut sr_byte: u8 = 0;
        sr_byte |= u8::from(self.sustain) << 4;
        sr_byte |= u8::from(self.release);
        [ad_byte, sr_byte]
    }
}

impl Voice {
    pub fn payload(&self) -> [u8; 7] {
        // Creates a byte array for sending over SPI
        let freq_low = (self.frequency & 0xFF) as u8;
        let freq_high = (self.frequency >> 8) as u8;
        let pwm_low = (u16::from(self.pwm) & 0x00FF) as u8;
        let pwm_high = ((u16::from(self.pwm) & 0x0F00) >> 8) as u8;
        let control = self.control.payload();
        let envelope = self.envelope.payload();
        [
            freq_low,
            freq_high,
            pwm_low,
            pwm_high,
            control,
            envelope[0],
            envelope[1],
        ]
    }
}

impl Filter {
    pub fn payload(&self) -> [u8; 4] {
        let freq_low = (u16::from(self.frequency) & 0x7) as u8;
        let freq_high = ((u16::from(self.frequency) & 0x7F8) >> 3) as u8;
        let mut res_filt = u8::from(self.resonance) << 4;
        if self.targets.voice1 {
            res_filt |= 1
        }
        if self.targets.voice2 {
            res_filt |= 1 << 1
        }
        if self.targets.voice3 {
            res_filt |= 1 << 2
        }
        if self.targets.external {
            res_filt |= 1 << 3
        }
        let mut mode_vol: u8 = 0;
        mode_vol |= u8::from(self.volume);
        match self.kind {
            FilterKind::ThreeOff => mode_vol |= 1 << 7,
            FilterKind::HighPass => mode_vol |= 1 << 6,
            FilterKind::BandPass => mode_vol |= 1 << 5,
            FilterKind::LowPass => mode_vol |= 1 << 4,
        }
        [freq_low, freq_high, res_filt, mode_vol]
    }
}
