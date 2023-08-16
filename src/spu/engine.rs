use super::{oscillator::*, envelope::*, command::*, filter::Filter};

struct Voice {
    oscillator: Oscillator,
    envelope: Envelope,
    filter: Filter,
}

pub struct Engine {
    poly_voices_0: [Voice; 4],
    poly_waveform_0: Waveform,

    poly_voices_1: [Voice; 4],
    poly_waveform_1: Waveform,

    mono_voices: [Voice; 4],
    mono_waveforms: [Waveform; 4],


}