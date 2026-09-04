#![allow(unused)]

use super::command;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum FilterMode {
    AllPass,
    LowPass,
    BandPass,
    HighPass,
    Muted
}

impl FilterMode {
    pub fn from_u32(x: u32) -> Self {
        match x {
            _ => Self::Muted,
            1 => Self::LowPass,
            2 => Self::BandPass,
            3 => Self::HighPass,
            4 => Self::AllPass,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum CutoffMode {
    FixedFrequency(u16), // 0 - 65535 => 1.0 - 16000.0
    PitchScaled(u16), // 0 - 65535 => 2^-4 - 2^4 (exponentially)
    PitchScaledEnvelope(u16, u16),
}

pub struct Filter {
    lp_state: f32,
    bp_state: f32,
    q: f32, // 0 to 0.707
    cutoff: CutoffMode,
    mode: FilterMode
}

#[derive(Copy, Clone, Debug)]
pub enum FilterCommand {
    Reset,
    SetMode(FilterMode),
    SetQ(u16),
    SetCutoff(CutoffMode),
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            lp_state: 0.0,
            bp_state: 0.0,
            cutoff: CutoffMode::FixedFrequency(0),
            q: 0.0,
            mode: FilterMode::Muted,
        }
    }
}

struct SVFOutput {
    pub highpass: f32,
    pub bandpass: f32,
    pub lowpass: f32,
}

fn state_variable_filter(q: f32, cutoff: f32, sample_rate: f32, input: f32, bandpass_state: &mut f32, lowpass_state: &mut f32) -> SVFOutput {
    let prewarped_frequency = (std::f32::consts::PI * cutoff / sample_rate).tan();
    let damping = 1.0 / q;
    let normalization_factor = 1.0 / (1.0 + damping * prewarped_frequency + prewarped_frequency * prewarped_frequency);
    let highpass = (input - damping * *bandpass_state - damping * *lowpass_state) * normalization_factor;
    let bandpass = prewarped_frequency * highpass + *bandpass_state;
    let lowpass  = prewarped_frequency * bandpass + *lowpass_state;
    *bandpass_state = bandpass;
    *lowpass_state = lowpass;
    SVFOutput {
        highpass,
        bandpass,
        lowpass
    }
}


impl Filter {
    pub fn compute(&mut self, x: f32, dt: f32, env: f32, pitch: f32) -> f32 {
        let cutoff = match self.cutoff {
            CutoffMode::FixedFrequency(freq) => (freq as f32 / 65535.0) * 16000.0,
            CutoffMode::PitchScaled(scale) => pitch * 2.0f32.powf((scale as f32 - 32768.0) / 8192.0),
            CutoffMode::PitchScaledEnvelope(scale_0, scale_1) => {
                let low = (scale_0 as f32 - 32768.0) / 8192.0;
                let high = (scale_1 as f32 - 32768.0) / 8192.0;
                2.0f32.powf(env * (high - low) + low)
            }
        };
        let svf_output = state_variable_filter(self.q, cutoff, 1.0 / dt, x, &mut self.bp_state, &mut self.lp_state);
        match self.mode {
            FilterMode::Muted => 0.0,
            FilterMode::AllPass => x,
            FilterMode::LowPass => svf_output.lowpass,
            FilterMode::BandPass => svf_output.bandpass,
            FilterMode::HighPass => svf_output.highpass,
            _ => panic!("Unimplemented filter mode: {:?}", self.mode)
        }
    }

    pub fn send_command(&mut self, command: FilterCommand) {
        match command {
            FilterCommand::Reset => {
                self.bp_state = 0.0;
                self.lp_state = 0.0;
            }
            FilterCommand::SetMode(mode) => self.mode = mode,
            FilterCommand::SetQ(q) => self.q = (q as f32 / 65535.0) * 0.707,
            FilterCommand::SetCutoff(cutoff) => self.cutoff = cutoff,
        }
    }
}
