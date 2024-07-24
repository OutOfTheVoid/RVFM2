use std::{iter::Filter, sync::Arc};

use crate::machine::{Machine, ReadResult};

use super::{super::command_list::CommandList, envelope::EnvelopeCommand, filter::{FilterCommand, FilterMode}, oscillator::{OscillatorCommand, Waveform}, pitch::{PitchCommand, PitchMode}, sampler::{LoopMode, SamplerCommand}};

pub const SPU_COMMAND_RESET_SAMPLE_COUNTER   : u8 = 0x00;
pub const SPU_COMMAND_WAIT_SAMPLE_COUNTER    : u8 = 0x01;
pub const SPU_COMMAND_WRITE_FLAG             : u8 = 0x02;
pub const SPU_COMMAND_COMMAND_STOP           : u8 = 0x04;
pub const SPU_COMMAND_ENVELOPE_COMMAND       : u8 = 0x05;
pub const SPU_COMMAND_ENVELOPE_PARAM         : u8 = 0x06;
pub const SPU_COMMAND_OSCILLATOR_COMMAND     : u8 = 0x07;
pub const SPU_COMMAND_OSCILLATOR_PARAM       : u8 = 0x08;
pub const SPU_COMMAND_FILTER_COMMAND         : u8 = 0x09;
pub const SPU_COMMAND_FILTER_PARAM           : u8 = 0x0A;
pub const SPU_COMMAND_PITCH_COMMAND          : u8 = 0x0B;
pub const SPU_COMMAND_PITCH_PARAM            : u8 = 0x0C;
pub const SPU_COMMAND_SET_MIX                : u8 = 0x0D;
pub const SPU_COMMAND_NOTE_ON                : u8 = 0x0E;
pub const SPU_COMMAND_RELWAIT_SAMPLE_COUNTER : u8 = 0x0F;
pub const SPU_COMMAND_SAMPLER_PARAM          : u8 = 0x10;
pub const SPU_COMMAND_SAMPLER_COMMAND        : u8 = 0x11;

#[derive(Debug)]
pub enum SpuCommand {
    ResetSampleCounter {
        reset_value: u32
    },
    WaitSampleCounter {
        count: u32
    },
    WriteFlag {
        address: u32,
        value: u32,
        interrupt: bool
    },
    Stop,
    Envelope {
        envelope_command: EnvelopeCommand,
        target: u8
    },
    Oscillator {
        oscillator_command: OscillatorCommand,
        target: u8,
    },
    Filter {
        filter_command: FilterCommand,
        target: u8,
    },
    Pitch {
        pitch_command: PitchCommand,
        target: u8,
    },
    SetMix {
        channel: u16,
        mix: i16,
    },
    NoteOn {
        target: u8,
        frequency: u16,
    },
    RelativeWaitSampleCounter {
        count: u32,
    },
    Sampler {
        target: u8,
        sampler_command: SamplerCommand,
    },
}

impl SpuCommand {
    pub fn read(command_list: &CommandList, offset: u32) -> Option<(u32, Self)> {
        let command = command_list.read_u8(offset)?;
        Some(
            match command {
                SPU_COMMAND_RESET_SAMPLE_COUNTER =>
                    (offset + 5, SpuCommand::ResetSampleCounter {
                        reset_value: command_list.read_u32(offset + 1)?
                    }),
                SPU_COMMAND_WAIT_SAMPLE_COUNTER =>
                    (offset + 5, SpuCommand::WaitSampleCounter {
                        count: command_list.read_u32(offset + 1)?
                    }),
                SPU_COMMAND_WRITE_FLAG =>
                    (offset + 10, SpuCommand::WriteFlag {
                        interrupt: command_list.read_u8(offset + 1)? != 0,
                        address: command_list.read_u32(offset + 2)?,
                        value: command_list.read_u32(offset + 6)?,
                    }),
                SPU_COMMAND_COMMAND_STOP =>
                    (offset + 1, SpuCommand::Stop),
                SPU_COMMAND_ENVELOPE_COMMAND =>
                    (offset + 3, SpuCommand::Envelope {
                        target: command_list.read_u8(offset + 1)?,
                        envelope_command: match command_list.read_u8(offset + 2)? {
                            0 => EnvelopeCommand::Mute,
                            1 => EnvelopeCommand::Off,
                            2 => EnvelopeCommand::On,
                            _ => None?
                        },
                    }),
                SPU_COMMAND_ENVELOPE_PARAM => {
                    let target = command_list.read_u8(offset + 1)?;
                    let (param_bytes, envelope_command) = match command_list.read_u8(offset + 2)? {
                        0 => (4, EnvelopeCommand::SetAttack(command_list.read_u32(offset + 3)?)),
                        1 => (4, EnvelopeCommand::SetDecay(command_list.read_u32(offset + 3)?)),
                        2 => (4, EnvelopeCommand::SetRelease(command_list.read_u32(offset + 3)?)),
                        3 => (2, EnvelopeCommand::SetSustain(command_list.read_u16(offset + 3)? as i16)),
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Envelope {
                        target,
                        envelope_command
                    })
                },
                SPU_COMMAND_OSCILLATOR_COMMAND => {
                    let target = command_list.read_u8(offset + 1)?;
                    let oscillator_command = match command_list.read_u8(offset + 2)? {
                        0 => OscillatorCommand::Reset,
                        _ => None?
                    };
                    (offset + 3, SpuCommand::Oscillator {
                        target,
                        oscillator_command,
                    })
                },
                SPU_COMMAND_OSCILLATOR_PARAM => {
                    let target = command_list.read_u8(offset + 1)?;
                    let (param_bytes, oscillator_command) = match command_list.read_u8(offset + 2)? {
                        0 => (3, OscillatorCommand::SetParam {
                            param: command_list.read_u8(offset + 3)?,
                            value: command_list.read_u16(offset + 4)? as i16,
                        }),
                        1 => (3, OscillatorCommand::SetPhase {
                            phase: command_list.read_u8(offset + 3)?,
                            value: command_list.read_u16(offset + 4)? as i16,
                        }),
                        2 => (1, OscillatorCommand::SetWaveform(Waveform::from_u32(command_list.read_u8(offset + 3)? as u32))),
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Oscillator {
                        oscillator_command,
                        target
                    })
                },
                SPU_COMMAND_FILTER_COMMAND => {
                    let target = command_list.read_u8(offset + 1)?;
                    let (param_bytes, filter_command) = match command_list.read_u8(offset + 2)? {
                        0 => (0, FilterCommand::Reset),
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Filter { filter_command, target })
                },
                SPU_COMMAND_FILTER_PARAM => {
                    let target = command_list.read_u8(offset + 1)?;
                    let (param_bytes, filter_command) = match command_list.read_u8(offset + 2)? {
                        0 => (1, FilterCommand::SetMode(FilterMode::from_u32(command_list.read_u8(offset + 3)? as u32))),
                        1 => (2, FilterCommand::SetResonance(command_list.read_u16(offset + 3)?)),
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Filter {
                        filter_command,
                        target
                    })
                },
                SPU_COMMAND_PITCH_COMMAND => {
                    let target = command_list.read_u8(offset + 1)?;
                    let pitch_command = match command_list.read_u8(offset + 2)? {
                        0 => PitchCommand::Finish,
                        _ => None?
                    };
                    (offset + 3, SpuCommand::Pitch { pitch_command, target })
                },
                SPU_COMMAND_PITCH_PARAM => {
                    let target = command_list.read_u8(offset + 1)?;
                    let (param_bytes, pitch_command) = match command_list.read_u8(offset + 2)? {
                        0 => (2, PitchCommand::SetTarget(command_list.read_u16(offset + 3)?)),
                        1 => (2, PitchCommand::SetSpeed(command_list.read_u16(offset + 3)?)),
                        2 => (1, PitchCommand::SetMode(PitchMode::from_u32(command_list.read_u8(offset + 3)? as u32))),
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Pitch {
                        pitch_command,
                        target
                    })
                },
                SPU_COMMAND_SET_MIX => {
                    let channel = command_list.read_u8(offset + 1)? as u16;
                    let mix = command_list.read_u16(offset + 2)? as i16;
                    (offset + 4, SpuCommand::SetMix{
                        channel,
                        mix
                    })
                },
                SPU_COMMAND_NOTE_ON => {
                    let target = command_list.read_u8(offset + 1)?;
                    let frequency = command_list.read_u16(offset + 2)?;
                    (offset + 4, SpuCommand::NoteOn {
                        target,
                        frequency
                    })
                },
                SPU_COMMAND_RELWAIT_SAMPLE_COUNTER => {
                    let count = command_list.read_u32(offset + 1)?;
                    (offset + 5, SpuCommand::RelativeWaitSampleCounter {
                        count
                    })
                },
                SPU_COMMAND_SAMPLER_PARAM => {
                    let subcommand = command_list.read_u8(offset + 1)?;
                    let target = command_list.read_u8(offset + 2)?;
                    let (param_bytes, sampler_command) = match subcommand {
                        0 => {
                            let channel_count = command_list.read_u8(offset + 3)?;
                            let sample_count = command_list.read_u32(offset + 4)?;
                            let start_address = command_list.read_u32(offset + 8)?;
                            (9, SamplerCommand::Setup {
                                channel_count: match channel_count {
                                    1 => super::sampler::ChannelCount::Stereo,
                                    _ => super::sampler::ChannelCount::Mono,
                                },
                                sample_count,
                                start_address
                            })
                        },
                        1 => {
                            let loop_mode = command_list.read_u32(offset + 3)?;
                            let loop_mode = match loop_mode {
                                0xFFFF_FFFF => LoopMode::Infinite,
                                _ => LoopMode::Finite(loop_mode)
                            };
                            (4, SamplerCommand::SetLoopMode(loop_mode))
                        },
                        _ => None?
                    };
                    (offset + 3 + param_bytes, SpuCommand::Sampler {
                        target,
                        sampler_command
                    })
                },
                SPU_COMMAND_SAMPLER_COMMAND => {
                    let subcommand = command_list.read_u8(offset + 1)?;
                    let target = command_list.read_u8(offset + 2)?;
                    let sampler_command = match subcommand {
                        0 => SamplerCommand::Start,
                        1 => SamplerCommand::Continue,
                        2 => SamplerCommand::Pause,
                        _ => None?
                    };
                    (offset + 3, SpuCommand::Sampler {
                        target,
                        sampler_command
                    })
                },
                _ => None?,
            }
        )
    }
}
