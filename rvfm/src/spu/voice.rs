use crate::spu::lfo::{Lfo, LfoCommand};

use super::{command, filter::{Filter, FilterCommand, FilterMode}, oscillator::{Oscillator, OscillatorCommand}, pitch::{Pitch, PitchCommand}};


#[derive(Copy, Clone, Debug)]
pub enum VoiceCommand {
    Lfo(LfoCommand, usize),
    Oscillator(OscillatorCommand),
    Filter(FilterCommand),
    Pitch(PitchCommand)
}

pub struct Voice {
    lfos: [Lfo; 3],
    oscillator: Oscillator,
    filter: Filter,
    pitch: Pitch,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            lfos: [(); 3].map(|_| Lfo::default()),
            oscillator: Oscillator::default(),
            filter: Filter::default(),
            pitch: Pitch::default(),
        }
    }
}

impl Voice {
    pub fn send_command(&mut self, command: VoiceCommand) {
        match command {
            VoiceCommand::Lfo(lfo_command, index) => {
                match index {
                    3 => self.lfos.iter_mut().for_each(|lfo| lfo.send_command(lfo_command)),
                    _ => self.lfos[index].send_command(lfo_command),
                }
            }
            VoiceCommand::Oscillator(oscillator_command) =>
                self.oscillator.send_command(oscillator_command),
            VoiceCommand::Filter(filter_command) =>
                self.filter.send_command(filter_command),
            VoiceCommand::Pitch(pitch_command) =>
                self.pitch.send_command(pitch_command),
        }
    }

    pub fn process(&mut self, dt: f32, filter_env: f32) -> f32 {
        let lfos = self.lfos.each_mut().map(|lfo| lfo.process(dt));
        let pitch = self.pitch.process(lfos);
        let osc = self.oscillator.compute(dt, pitch);
        let filt = self.filter.compute(osc, dt, filter_env, pitch);
        filt
    }
}
