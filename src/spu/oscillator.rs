#[derive(Copy, Clone, Debug)]
pub enum Waveform {
    Sin,
    Triangle,
    Square,
    SuperSaw,
}

impl Waveform {
    pub fn from_u32(val: u32) -> Self {
        match val {
            1 => Self::Triangle,
            2 => Self::Sin,
            3 => Self::SuperSaw,
            _ => Self::Square,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            Self::Square => 0,
            Self::Triangle => 1,
            Self::Sin => 2,
            Self::SuperSaw => 3,
        }
    }
}

pub struct Oscillator {
    pub params: [i16; 4],
    pub phases: [f32; 5],
}

impl Oscillator {
    pub fn compute(&mut self, waveform: Waveform, dt: f32, f: f32, params: &[f32]) -> f32 {
        match waveform {
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
                self.phases[0] += f * dt;
                self.phases[0] = self.phases[0] % 1.0;
                if self.phases[0] > 0.5 {
                    - (self.phases[0] - 0.5) * 4.0 + 1.0
                } else {
                    self.phases[0] * 4.0 - 1.0
                }
            },
            Waveform::SuperSaw => {
                let spread = params[0];
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
            }
        }
    }
}
