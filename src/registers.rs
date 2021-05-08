const VOICE_REG_OFFSET: u8 = 0x07;

pub enum VoiceRegister {
    Freq = 0x00,
    Pwm = 0x02,
    Control = 0x04,
    AttackDecay = 0x05,
    SustainRelease = 0x06,
}

pub enum FilterRegister {
    CutoffLow = 0x15,
    CutoffHigh = 0x16,
    Resonance = 0x17,
    Mode = 0x18,
}

impl VoiceRegister {
    // Get register address
    pub fn addr(self, voice: usize) -> u8 {
        (self as u8) + VOICE_REG_OFFSET * (voice as u8)
    }
}

impl FilterRegister {
    pub fn addr(self) -> u8 {
        self as u8
    }
}
