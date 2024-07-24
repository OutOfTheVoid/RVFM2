enum EnvelopStage {
    Attack(u32),
    Decay(u32),
    Sustain,
    Release(u32),
    Idle,
}

pub struct Envelope {
    attack: u32,
    decay: u32,
    sustain: i16,
    release: u32,
    stage: EnvelopStage,
    active: bool,
}

impl Default for Envelope {
    fn default() -> Self {
        Self {
            attack: 0,
            decay: 0,
            sustain: 0,
            release: 0,
            stage: EnvelopStage::Idle,
            active: false,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum EnvelopeCommand {
    SetAttack(u32),
    SetDecay(u32),
    SetSustain(i16),
    SetRelease(u32),
    On,
    Off,
    Mute,
}

impl Envelope {
    pub fn process(&mut self) -> Option<i16> {
        let (stage_next, x) = match self.stage {
            EnvelopStage::Idle => return None,
            EnvelopStage::Attack(t) => {
                if t >= self.attack {
                    (EnvelopStage::Decay(0), 0x7FFF)
                } else {
                    let x = ((0x7FFFi32 * (t as i32)) / (self.attack as i32)) as i16;
                    (EnvelopStage::Attack(t + 1), x)
                }
            },
            EnvelopStage::Decay(t) => {
                if t >= self.decay {
                    (if self.active { EnvelopStage::Sustain } else { EnvelopStage::Release(0) }, self.sustain)
                } else {
                    let x = self.sustain + ((0x7FFFi32 - self.sustain as i32) * ((self.decay - t) as i32) / (self.decay as i32)) as i16;
                    (EnvelopStage::Decay(t + 1), x)
                }
            },
            EnvelopStage::Sustain => {
                if self.active {
                    (EnvelopStage::Sustain, self.sustain)
                } else {
                    (EnvelopStage::Release(0), self.sustain)
                }
            },
            EnvelopStage::Release(t) => {
                if t >= self.release {
                    (EnvelopStage::Idle, 0)
                } else {
                    let x = (self.sustain as i32 * ((self.release - t) as i32)) / (self.release as i32);
                    (EnvelopStage::Release(t + 1), x as i16)
                }
            },
        };
        self.stage = stage_next;
        Some(x)
    }

    pub fn send_command(&mut self, command: EnvelopeCommand) {
        match command {
            EnvelopeCommand::SetAttack(value) => self.attack = value,
            EnvelopeCommand::SetDecay(value) => self.decay = value,
            EnvelopeCommand::SetSustain(value) => self.sustain = value,
            EnvelopeCommand::SetRelease(value) => self.release = value,
            EnvelopeCommand::On => {
                self.stage = EnvelopStage::Attack(0);
                self.active = true;
            },
            EnvelopeCommand::Off => {
                self.active = false;
            },
            EnvelopeCommand::Mute => {
                self.stage = EnvelopStage::Idle;
                self.active = false;
            },
        }
    }
}
