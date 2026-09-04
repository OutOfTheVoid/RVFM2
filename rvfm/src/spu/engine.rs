use std::sync::Arc;

use crate::{machine::Machine, spu::lfo::LfoCommand};

use super::{sampler::*, command::*, envelope::*, filter::{Filter, FilterCommand}, oscillator::*, pitch::PitchCommand, voice::{Voice, VoiceCommand}, SAMPLE_RATE_16000};

pub struct Engine {
    voices: [Voice; 16],
    voice_envelopes: [Envelope; 16],
    filter_envelopes: [Envelope; 16],
    samplers: [Sampler; 32],
    mix_coefficients: [(i16, i16); 48],
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            voices: [(); 16].map(|_| Voice::default()),
            voice_envelopes: [(); 16].map(|_| Envelope::default()),
            filter_envelopes: [(); 16].map(|_| Envelope::default()),
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
            let envelope_scale = self.voice_envelopes[v].process();
            let filter_envelope_scale = self.filter_envelopes[v].process().unwrap_or(0) as f32 / 32768.0;
            match envelope_scale {
                Some(x) => {
                    let (yl, yr) = self.mix_coefficients[v as usize];
                    let (yl, yr) = (fixed_to_float(yl), fixed_to_float(yr));
                    let x = fixed_to_float(x);
                    let x = x * x; // give the envelope exponential scaling to linearly map volume
                    let a = self.voices[v].process(dt, filter_envelope_scale);
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
             0..=15 => self.voice_envelopes [target as usize     ].send_command(command),
            16..=31 => self.filter_envelopes[target as usize % 16].send_command(command),
            32..=47 => {
                let index = target as usize % 16;
                self.voice_envelopes[index].send_command(command);
                self.filter_envelopes[index].send_command(command);
            },
            0xFF => {
                self.voice_envelopes.iter_mut().for_each(|e| e.send_command(command));
                self.filter_envelopes.iter_mut().for_each(|e| e.send_command(command));
            },
            0xFE => self.voice_envelopes.iter_mut().for_each(|e| e.send_command(command)),
            0xFD => self.filter_envelopes.iter_mut().for_each(|e| e.send_command(command)),
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

    pub fn lfo_command(&mut self, target: u8, command: LfoCommand) {
        let target_voice = target >> 2;
        match target_voice {
            0..=15 => self.voices[target_voice as usize].send_command(VoiceCommand::Lfo(command, (target & 3) as usize)),
            0x3F => self.voices.iter_mut().for_each(|voice| voice.send_command(VoiceCommand::Lfo(command, (target & 3) as usize))),
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