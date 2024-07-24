use std::sync::Arc;

use crate::machine::Machine;

use super::{sampler::*, command::*, envelope::*, filter::{Filter, FilterCommand}, oscillator::*, pitch::PitchCommand, voice::{Voice, VoiceCommand}, SAMPLE_RATE_16000};

pub struct Engine {
    voices: [Voice; 16],
    envelopes: [Envelope; 16],
    samplers: [Sampler; 32],
    mix_coefficients: [(i16, i16); 48],
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            voices: [(); 16].map(|_| Voice::default()),
            envelopes: [(); 16].map(|_| Envelope::default()),
            mix_coefficients: [(0, 0); 48],
            samplers: [(); 32].map(|_| Sampler::new()),
        }
    }
}

const FLOAT_FIXED_SCALE: f32 = std::i16::MAX as f32;
const FIXED_FLOAT_SCALE: f32 = 1.0 / (FLOAT_FIXED_SCALE);

fn fixed_to_float(x: i16) -> f32 {
    (x as f32) * FIXED_FLOAT_SCALE
}

fn float_to_fixed(f: f32) -> i16 {
    (f * FLOAT_FIXED_SCALE) as i32 as i16
}

impl Engine {
    pub fn process(&mut self, dt: f32, machine: &Arc<Machine>) -> (i16, i16) {
        let mut sample = (0, 0);
        for v in 0..=15 {
            let envelope_scale = self.envelopes[v].process();
            match envelope_scale {
                Some(x) => {
                    let (yl, yr) = self.mix_coefficients[v as usize];
                    let (yl, yr) = (fixed_to_float(yl), fixed_to_float(yr));
                    let x = fixed_to_float(x);
                    let x = x * x; // give the envelope exponential scaling to linearly map volume
                    let a = self.voices[v].process(dt);
                    let ax = a * x;
                    sample.0 += float_to_fixed(yl * ax);
                    sample.1 += float_to_fixed(yr * ax);
                },
                None => {}
            }
        }
        for s in 0..=31 {
            let (a_l, a_r) = self.samplers[s].process(machine);
            let (yl, yr) = self.mix_coefficients[s as usize + 16];
            sample.0 += ((a_l as i32 * yl as i32) / std::i16::MAX as i32) as i16;
            sample.1 += ((a_r as i32 * yr as i32) / std::i16::MAX as i32) as i16;
        }
        sample
    }

    pub fn envelope_command(&mut self, target: u8, command: EnvelopeCommand) {
        match target {
            0..=15 => self.envelopes[target as usize].send_command(command),
            0xFF => self.envelopes.iter_mut().for_each(|e| e.send_command(command)),
            _ => {}
        }
    }

    pub fn oscillator_command(&mut self, target: u8, command: OscillatorCommand) {
        match target {
            0..=15 => self.voices[target as usize].send_command(VoiceCommand::Oscillator(command)),
            0xFF => self.voices.iter_mut().for_each(|e| e.send_command(VoiceCommand::Oscillator(command.clone()))),
            _ => {}
        }
    }

    pub fn filter_command(&mut self, target: u8, command: FilterCommand) {
        match target {
            0..=15 => self.voices[target as usize].send_command(VoiceCommand::Filter(command)),
            0xFF => self.voices.iter_mut().for_each(|e| e.send_command(VoiceCommand::Filter(command.clone()))),
            _ => {}
        }
    }

    pub fn pitch_command(&mut self, target: u8, command: PitchCommand) {
        match target {
            0..=15 => self.voices[target as usize].send_command(VoiceCommand::Pitch(command)),
            0xFF => self.voices.iter_mut().for_each(|e| e.send_command(VoiceCommand::Pitch(command.clone()))),
            _ => {}
        }
    }

    pub fn sampler_command(&mut self, target: u8, command: SamplerCommand) {
        match target {
            0..=31 => self.samplers[target as usize].send_command(command),
            0xFF => self.samplers.iter_mut().for_each(|s| s.send_command(command)),
            _ => {}
        }
    }

    pub fn set_mix(&mut self, channel: u16, value: i16) {
        let right = channel & 1 != 0;
        let voice = channel >> 1;
        match voice {
            0..=47 => {
                if right {
                    self.mix_coefficients[voice as usize].1 = value;
                } else {
                    self.mix_coefficients[voice as usize].0 = value;
                }
            },
            0xFF => {
                self.mix_coefficients.fill((value, value));
            },
            _ => {}
        }
    }
}