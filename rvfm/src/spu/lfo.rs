#[derive(Copy, Clone, Debug, Default)]
pub enum Waveform {
    #[default]
    DC,
    Sin,
    Triangle,
    Sawtooth,
    Square,
}

impl Waveform {
    pub fn from_u8(x: u8) -> Option<Self> {
        Some(match x {
            0 => Self::DC,
            1 => Self::Sin,
            2 => Self::Triangle,
            3 => Self::Sawtooth,
            4 => Self::Square,
            _ => None?,
        })
    }
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Lfo {
    waveform: Waveform,
    speed: f32,
    theta: f32,
    offset: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum LfoCommand {
    Reset,
    SetSpeed(u16),
    SetWaveform(Waveform),
    SetOffset(u16),
}

impl Lfo {
    pub fn send_command(&mut self, command: LfoCommand) {
        match command {
            LfoCommand::Reset => self.theta = 0.0,
            LfoCommand::SetSpeed(speed) => self.speed = ((speed as f32) / 65535.0) * 100.0,
            LfoCommand::SetWaveform(waveform) => self.waveform = waveform,
            LfoCommand::SetOffset(offset) => self.offset = (offset as f32) / 32768.0 - 1.0,
        }
    }

    pub fn process(&mut self, dt: f32) -> f32 {
        self.theta += dt * self.speed;
        self.theta = self.theta.fract();
        let ac = match self.waveform {
            Waveform::DC => 0.0,
            Waveform::Sin => (self.theta * std::f32::consts::TAU).sin(),
            Waveform::Square => if self.theta >= 0.5 { 1.0 } else { - 1.0 },
            Waveform::Triangle => if self.theta > 0.5 { 3.0 - self.theta * 4.0 } else { self.theta * 4.0 - 1.0 },
            Waveform::Sawtooth => self.theta * 2.0 - 1.0,
        };
        ac + self.offset
    }
}
