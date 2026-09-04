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
    mode: PitchMode,
    lfo: Option<usize>,
    mod_depth: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum PitchCommand {
    SetTarget(u16),
    SetSpeed(u16),
    SetMode(PitchMode),
    Finish,
    SetLfo(Option<usize>),
    SetModDepth(u16),
}

impl Pitch {
    pub fn send_command(&mut self, command: PitchCommand) {
        match command {
            PitchCommand::SetTarget(f) => self.target = f as f32 / 0x0010 as f32,
            PitchCommand::SetMode(mode) => self.mode = mode,
            PitchCommand::SetSpeed(speed) => self.speed = speed as f32 / std::u16::MAX as f32,
            PitchCommand::Finish => self.current = self.target,
            PitchCommand::SetLfo(lfo) => self.lfo = lfo,
            PitchCommand::SetModDepth(mod_depth) => self.mod_depth = (mod_depth as f32 / 32768.0) - 1.0,
        }
    }

    pub fn process(&mut self, lfos: [f32; 3]) -> f32 {
        match self.mode {
            PitchMode::Constant => {
                self.current = self.target;
            },
            PitchMode::PortamentoQuadratic => {
                self.current = (self.target - self.current) * self.speed;
            },
        }
        if let Some(lfo) = self.lfo {
            let lfo_value = lfos[lfo];
            self.current * (1.0 + lfo_value * self.mod_depth)
        } else {
            self.current
        }
    }
}
