#![allow(unused)]

use super::command;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FilterMode {
    AllPass,

    LowPass6,
    LowPass12,
    LowPass18,
    LowPass24,

    HighPass6,
    HighPass12,
    HighPass18,
    HighPass24,

    BandPass6,
    BandPass12,
    BandPass18,
    BandPass24,
}

impl FilterMode {
    pub fn from_u32(x: u32) -> Self {
        match x {
            1  => Self::LowPass6,
            2  => Self::LowPass12,
            3  => Self::LowPass18,
            4  => Self::LowPass24,

            5  => Self::HighPass6,
            6  => Self::HighPass12,
            7  => Self::HighPass18,
            8  => Self::HighPass24,

            9  => Self::BandPass6,
            10 => Self::BandPass12,
            11 => Self::BandPass18,
            12 => Self::BandPass24,

            _ => Self::AllPass,
        }
    }
}

pub struct Filter {
    a: [f32; 2],
    b: [f32; 3],
    resonance: f32,
    mode: FilterMode
}

#[derive(Copy, Clone, Debug)]
pub enum FilterCommand {
    Reset,
    SetMode(FilterMode),
    SetResonance(u16),
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            a: [0.0; 2],
            b: [0.0; 3],
            resonance: 0.0,
            mode: FilterMode::AllPass,
        }
    }
}

impl Filter {
    pub fn compute(&mut self, x: f32, dt: f32) -> f32 {
        match self.mode {
            FilterMode::AllPass => x,
            _ => panic!("Unimplemented filter mode: {:?}", self.mode)
        }
    }

    pub fn send_command(&mut self, command: FilterCommand) {
        match command {
            FilterCommand::Reset => {
                self.a = [0.0; 2];
                self.b = [0.0; 3];
            }
            FilterCommand::SetMode(mode) => self.mode = mode,
            FilterCommand::SetResonance(resonance) => self.resonance = resonance as f32 / std::u16::MAX as f32,
        }
    }
}
