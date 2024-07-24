use bytemuck::cast_slice_mut;

#[derive(Copy, Clone, Debug)]
pub enum Waveform {
    Sin,
    Triangle,
    Square,
    SuperSaw,
    Noise
}

impl Default for Waveform {
    fn default() -> Self {
        Waveform::Square
    }
}

impl Waveform {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::Triangle,
            2 => Self::Sin,
            3 => Self::SuperSaw,
            4 => Self::Noise,
            _ => Self::Square,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Square   => 0,
            Self::Triangle => 1,
            Self::Sin      => 2,
            Self::SuperSaw => 3,
            Self::Noise    => 4,
        }
    }
}

pub struct Oscillator {
    pub waveform: Waveform,
    pub params: [f32; 4],
    pub phases: [f32; 5],
}

impl Default for Oscillator {
    fn default() -> Self {
        Oscillator {
            waveform: Waveform::Square,
            params: [0.0; 4],
            phases: [0.0; 5],
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum OscillatorCommand {
    SetWaveform(Waveform),
    Reset,
    SetPhase { phase: u8, value: i16 },
    SetParam { param: u8, value: i16 },
}

impl Oscillator {
    pub fn compute(&mut self, dt: f32, f: f32) -> f32 {
        match self.waveform {
            Waveform::Sin => {
                self.phases[0] += f * dt;
                self.phases[0] = self.phases[0] % 1.0;
                (self.phases[0] * std::f32::consts::TAU).sin()
            },
            Waveform::Square => {
                self.phases[0] += f * dt;
                self.phases[0] = self.phases[0] % 1.0;
                let transition = self.params[0] as f32 + 32767.5 / 65535.0;
                if self.phases[0] < transition { 1.0 } else { - 1.0 }
            },
            Waveform::Triangle => {
                let value = if self.phases[0] >= 0.5 {
                    - self.phases[0] * 4.0 + 3.0
                } else {
                    self.phases[0] * 4.0 - 1.0
                };
                self.phases[0] += f * dt;
                self.phases[0] = self.phases[0] % 1.0;
                value
            },
            Waveform::SuperSaw => {
                let spread = self.params[0];
                let spread_sq = spread * spread;
                let f = [
                    f * spread_sq,
                    f * spread,
                    f,
                    f / spread,
                    f / spread_sq,
                ];
                let mut total: f32 = 0.0;
                for i in 0..5 {
                    self.phases[i] += dt * f[i];
                    self.phases[i] %= 1.0;
                    total += self.phases[i] - 0.5;
                }
                total * 0.4
            },
            Waveform::Noise => {
                /*
                uint64_t xorshift64(struct xorshift64_state *state)
                {
                uint64_t x = state->a;
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                return state->a = x;
                }
                 */
                let bits = cast_slice_mut::<_, u64>(&mut self.phases);
                let mut x = bits[0];
                x ^= x << 13;
                x ^= x >> 7;
                x ^= x << 17;
                bits[0] = x;
                (x as i16 as f32 - 0.5) / 32767.0
            },
        }
    }

    pub fn send_command(&mut self, command: OscillatorCommand) {
        match command {
            OscillatorCommand::Reset => {
                self.params = [0.0; 4];
                self.phases = [0.0; 5];
            },
            OscillatorCommand::SetParam { param, value } => {
                if param < 4 {
                    self.params[param as usize] = (value as f32) / 256.0;
                }
            },
            OscillatorCommand::SetPhase { phase, value } => {
                if phase < 5 {
                    self.phases[phase as usize] = std::f32::consts::TAU * (value as f32) / (i16::MAX as f32);
                }
            },
            OscillatorCommand::SetWaveform(waveform) => {
                self.waveform = waveform;
            }
        }
    }
}
