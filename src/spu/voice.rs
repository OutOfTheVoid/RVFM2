use super::{command, filter::{Filter, FilterCommand, FilterMode}, oscillator::{Oscillator, OscillatorCommand}, pitch::{Pitch, PitchCommand}};


pub enum VoiceCommand {
    Oscillator(OscillatorCommand),
    Filter(FilterCommand),
    Pitch(PitchCommand)
}

pub struct Voice {
    oscillator: Oscillator,
    filter: Filter,
    pitch: Pitch,
}

impl Default for Voice {
    fn default() -> Self {
        Self {
            oscillator: Oscillator::default(),
            filter: Filter::default(),
            pitch: Pitch::default(),
        }
    }
}

impl Voice {
    pub fn send_command(&mut self, command: VoiceCommand) {
        match command {
            VoiceCommand::Oscillator(oscillator_command) =>
                self.oscillator.send_command(oscillator_command),
            VoiceCommand::Filter(filter_command) =>
                self.filter.send_command(filter_command),
            VoiceCommand::Pitch(pitch_command) =>
                self.pitch.send_command(pitch_command),
        }
    }

    pub fn process(&mut self, dt: f32) -> f32 {
        let pitch = self.pitch.process();
        let osc = self.oscillator.compute(dt, pitch);
        let filt = self.filter.compute(osc, dt);
        filt
    }
}
