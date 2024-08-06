#[derive(Copy, Clone, Debug)]
pub enum PitchMode {
    Constant,
    PortamentoQuadratic,
}

impl Default for PitchMode {
    fn default() -> Self {
        Self::Constant
    }
}

impl PitchMode {
    pub fn from_u32(x: u32) -> Self {
        match x {
            1 => Self::PortamentoQuadratic,
            _ => Self::Constant,
        }
    }
}

#[derive(Copy, Clone, Default)]
pub struct Pitch {
    current: f32,
    target: f32,
    speed: f32,
    mode: PitchMode
}

#[derive(Copy, Clone, Debug)]
pub enum PitchCommand {
    SetTarget(u16),
    SetSpeed(u16),
    SetMode(PitchMode),
    Finish,
}

impl Pitch {
    pub fn send_command(&mut self, command: PitchCommand) {
        match command {
            PitchCommand::SetTarget(f) => self.target = f as f32 / 0x0010 as f32,
            PitchCommand::SetMode(mode) => self.mode = mode,
            PitchCommand::SetSpeed(speed) => self.speed = speed as f32 / std::u16::MAX as f32,
            PitchCommand::Finish => self.current = self.target,
        }
    }

    pub fn process(&mut self) -> f32 {
        match self.mode {
            PitchMode::Constant => {
                self.current = self.target;
            },
            PitchMode::PortamentoQuadratic => {
                self.current = (self.target - self.current) * self.speed;
            }
        }
        self.current
    }
}
