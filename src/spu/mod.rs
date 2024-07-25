mod oscillator;
mod envelope;
mod sampler;
mod engine;
mod command;
mod param;
mod filter;
mod voice;
mod pitch;

use std::{borrow::BorrowMut, collections::VecDeque, sync::{atomic::{self, AtomicU32}, mpsc, Arc}, time::Duration};

use command::SpuCommand;
use super::command_list::{parse_commandlist_header, CommandListHeaderError};
use cpal::{traits::{DeviceTrait, HostTrait, StreamTrait}, FromSample, SampleFormat, SampleRate, Stream, StreamConfig};
use parking_lot::Mutex;
use static_init::dynamic;

use crate::{interrupt_controller::{InterruptType, INTERRUPT_CONTROLLER}, machine::{Machine, ReadResult, WriteResult}, pointer_queue::PointerQueue};

use self::engine::Engine;

#[dynamic]
static SPU_REGISTERS: SpuRegisters = SpuRegisters::new();

#[dynamic]
static STAGING_COMMAND_QUEUE: Mutex<VecDeque<(u32, SpuCommand)>> = Mutex::new(VecDeque::with_capacity(0x10000));

#[dynamic]
static SPU_QUEUE: Mutex<PointerQueue> = Mutex::new(PointerQueue::new());

thread_local! {
    static SPU_QUEUE_LOCAL: mpsc::Sender<(u32, u32)> = SPU_QUEUE.lock().make_tx();
}

struct SpuRegisters {
    sample_rate: AtomicU32,
    run_mode: AtomicU32,
    sample_counter: AtomicU32,
    submission_error: AtomicU32,
}

const SUBMISSION_ERROR_HEADER_NOT_IN_RAM: u32 = 1;
const SUBMISSION_ERROR_LIST_NOT_IN_RAM: u32 = 2;
const SUBMISSION_ERROR_LIST_TOO_LONG: u32 = 3;
const SUBMISSION_ERROR_INVALID_COMMAND: u32 = 4;

const RUN_MODE_STOPPED: u32 = 0;
const RUN_MODE_RUN: u32 = 1;
const RUN_MODE_WRITE_MASK: u32 = RUN_MODE_RUN;

const SAMPLE_RATE_16000: u32 = 0;
const SAMPLE_RATE_32000: u32 = 1;
const SAMPLE_RATE_41000: u32 = 2;
const SAMPLE_RATE_48000: u32 = 3;
const SAMPLE_RATE_WRITE_MASK: u32 = SAMPLE_RATE_48000;

impl SpuRegisters {
    pub fn new() -> Self {
        Self {
            sample_rate: AtomicU32::new(SAMPLE_RATE_16000),
            run_mode: AtomicU32::new(RUN_MODE_STOPPED),
            sample_counter: AtomicU32::new(0),
            submission_error: AtomicU32::new(0),
        }
    }
}

struct SoundProcessContext {
    pub conversion_buffer: Vec<i16>,
    pub output_sample_rate: f32,
    pub machine: Arc<Machine>,
    pub command_queues: [VecDeque<SpuCommand>; 4],
    pub engine: Engine,
}

pub struct SpuStreamHandle {
    #[allow(unused)]
    stream: Stream,
}

pub fn spu_init(machine: &Arc<Machine>) -> Option<SpuStreamHandle> {
    {
        let queue = SPU_QUEUE.lock().take_rx();
        let machine = machine.clone();
        std::thread::spawn(move || spu_command_thread(queue, machine));
    }
    let host = cpal::default_host();
    let device = host.default_output_device().expect("no default output device found!");
    let supported_configs = device.supported_output_configs().expect("failed to get supported output configs");
    let mut config = None;
    let mut format = SampleFormat::F32;
    for supported_config in supported_configs {
        if supported_config.min_sample_rate().0 <= 48000 && supported_config.max_sample_rate().0 >= 48000 && supported_config.channels() == 2 {
            let buffer_size = match supported_config.buffer_size() {
                cpal::SupportedBufferSize::Range { min, max } => 128.min(*max).min(*min),
                cpal::SupportedBufferSize::Unknown => 128,
            };
            config = Some(StreamConfig {
                channels: 2,
                buffer_size: cpal::BufferSize::Fixed(buffer_size),
                sample_rate: SampleRate(48000)
            });
            format = supported_config.sample_format();
            break;
        }
    }
    if let Some(config) = config {
        let timeout = None;//Some(Duration::from_millis(200));
        let mut process_context = SoundProcessContext {
            conversion_buffer: Vec::new(),
            output_sample_rate: config.sample_rate.0 as f32,
            machine: machine.clone(),
            command_queues: [(); 4].map(|_| VecDeque::new()),
            engine: Engine::default()
        };
        let stream_result = match format {
            SampleFormat::F32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<f32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::F64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<f64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I8  => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i8>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I16 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i16>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U8  => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u8>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U16 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u16>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            _ => { println!("SPU: unknown audio sample format"); return None; },
        };
        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                println!("SPU: failed to build output stream: {:?}", error);
                return None;
            }
        };
        let play_result = stream.play().expect("SPU: failed to start output stream");
        Some(SpuStreamHandle {
            stream
        })
    } else {
        println!("SPU: No audio config found!");
        None
    }
}

fn sound_process_generic<T: FromSample<i16>>(samples: &mut [T], context: &mut SoundProcessContext) {
    let spu_sample_rate = match SPU_REGISTERS.sample_rate.load(atomic::Ordering::Acquire) {
        SAMPLE_RATE_16000 => 16000.0,
        SAMPLE_RATE_32000 => 32000.0,
        SAMPLE_RATE_41000 => 41000.0,
        SAMPLE_RATE_48000 => 48000.0,
        _ => unreachable!()
    };
    let spu_over_output_sample_rate = spu_sample_rate / context.output_sample_rate as f32;
    let source_sample_length: usize = ((samples.len() >> 1) as f32 * spu_over_output_sample_rate).ceil() as usize;

    context.conversion_buffer.resize(source_sample_length * 2, 0);
    let conv_buffer = &mut context.conversion_buffer[0..source_sample_length * 2];
    sound_process(conv_buffer, &context.machine, &mut context.command_queues, &mut context.engine, spu_sample_rate);

    for i_dst in 0..(samples.len() >> 1) {
        let i_src = ((i_dst as f32 * spu_over_output_sample_rate).floor() as usize).clamp(0, source_sample_length - 1);
        samples[(i_dst << 1) | 0] = T::from_sample_(conv_buffer[(i_src << 1) | 0]);
        samples[(i_dst << 1) | 1] = T::from_sample_(conv_buffer[(i_src << 1) | 1]);
    }
}

fn sound_process(samples: &mut [i16], machine: &Arc<Machine>, command_queues: &mut [VecDeque<SpuCommand>; 4], engine: &mut Engine, sample_rate: f32) {
    match SPU_REGISTERS.run_mode.load(atomic::Ordering::Acquire) {
        RUN_MODE_STOPPED => {
            samples.fill(0);
        },
        RUN_MODE_RUN => {
            let dt = 1.0 / sample_rate;
            let regs = &SPU_REGISTERS;
            let sample_count = samples.len() / 2;
            for i in 0..sample_count {
                let i_left = (i << 1) | 0;
                let i_right = (i << 1) | 1;
                let (left, right) = spu_render(regs, machine, command_queues, engine, dt);
                samples[i_left] = left;
                samples[i_right] = right;
                
            }
        },
        _ => unreachable!()
    }
}

fn spu_render(regs: &SpuRegisters, machine: &Arc<Machine>, command_queues: &mut [VecDeque<SpuCommand>; 4], engine: &mut Engine, dt: f32) -> (i16, i16) {
    {
        let mut staging_queue = STAGING_COMMAND_QUEUE.try_lock();
        if let Some(staging_queue) = staging_queue.as_mut() {
            while let Some((queue, command)) = staging_queue.pop_front() {
                command_queues[queue as usize].push_back(command);
            }
        }
    }
    let sample_counter = regs.sample_counter.fetch_add(1, atomic::Ordering::AcqRel);
    for command_queue in &mut command_queues[..] {
        loop {
            if let Some(command) = command_queue.front_mut() {
                let mut replacement = None;
                let pop = match command {
                    SpuCommand::ResetSampleCounter { reset_value } => {
                        regs.sample_counter.store(*reset_value, atomic::Ordering::Release);
                        true
                    },
                    SpuCommand::WaitSampleCounter { count } => {
                        let result = sample_counter >= *count;
                        result
                    },
                    SpuCommand::WriteFlag { address, value, interrupt } => {
                        machine.write_u32(*address, *value);
                        if *interrupt {
                            INTERRUPT_CONTROLLER.trigger_interrupt(InterruptType::Spu);
                        }
                        true
                    },
                    SpuCommand::Stop => {
                        SPU_REGISTERS.run_mode.store(RUN_MODE_STOPPED, atomic::Ordering::Release);
                        true
                    },
                    SpuCommand::Envelope { envelope_command, target } => {
                        engine.envelope_command(*target, *envelope_command);
                        true
                    },
                    SpuCommand::Oscillator { oscillator_command, target } => {
                        engine.oscillator_command(*target, *oscillator_command);
                        true
                    },
                    SpuCommand::Filter { filter_command, target } => {
                        engine.filter_command(*target, *filter_command);
                        true
                    },
                    SpuCommand::Pitch { pitch_command, target } => {
                        engine.pitch_command(*target, *pitch_command);
                        true
                    },
                    SpuCommand::SetMix { channel, mix } => {
                        engine.set_mix(*channel, *mix);
                        true
                    },
                    SpuCommand::NoteOn { target, frequency } => {
                        engine.pitch_command(*target, pitch::PitchCommand::SetTarget(*frequency));
                        engine.envelope_command(*target, envelope::EnvelopeCommand::On);
                        true
                    },
                    SpuCommand::RelativeWaitSampleCounter { count } => {
                        replacement = Some(SpuCommand::WaitSampleCounter { count: *count + sample_counter });
                        false
                    },
                    SpuCommand::Sampler { target, sampler_command } => {
                        engine.sampler_command(*target, *sampler_command);
                        true
                    }
                };
                if pop {
                    println!("SPU: {:?}", command);
                    command_queue.pop_front();
                } else if let Some(replacement) = replacement {
                    *command = replacement;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }
    engine.process(dt, machine)
}

fn spu_command_thread(queue: mpsc::Receiver<(u32, u32)>, machine: Arc<Machine>) {
    let mut commands = Vec::new();
    loop {
        match queue.recv() {
            Ok((queue_index, command_list_address)) => {
                parse_command_list(command_list_address, &machine, &mut commands);
                let mut staging_queue = STAGING_COMMAND_QUEUE.lock();
                for command in commands.drain(..).into_iter() {
                    staging_queue.push_back((queue_index, command));
                }
            },
            Err(_) => {
                return
            },
        }
    }
}

fn parse_command_list(address: u32, machine: &Arc<Machine>, commands: &mut Vec<SpuCommand>) {
    let command_list = match parse_commandlist_header(address, machine) {
        Ok(command_list) => command_list,
        Err(error) => {
            match error {
                CommandListHeaderError::HeaderNotInRam => {
                    SPU_REGISTERS.submission_error.store(SUBMISSION_ERROR_HEADER_NOT_IN_RAM, atomic::Ordering::Release);
                },
                CommandListHeaderError::ListNotInRam => {
                    SPU_REGISTERS.submission_error.store(SUBMISSION_ERROR_LIST_NOT_IN_RAM, atomic::Ordering::Release);
                },
                CommandListHeaderError::ListTooLong => {
                    SPU_REGISTERS.submission_error.store(SUBMISSION_ERROR_LIST_TOO_LONG, atomic::Ordering::Release);
                },
            }
            return;
        }
    };
    let mut offset = 0;
    while let Some((offset_next, command)) = SpuCommand::read(&command_list, offset) {
        offset = offset_next;
        commands.push(command);
    }
    if offset != command_list.len() as u32 {
        SPU_REGISTERS.submission_error.store(SUBMISSION_ERROR_INVALID_COMMAND, atomic::Ordering::Release);
    }
}

pub fn spu_write_u32(offset: u32, value: u32) -> WriteResult {
    match offset {
        0x00 => SPU_REGISTERS.run_mode.store(value & RUN_MODE_WRITE_MASK, atomic::Ordering::Release),
        0x04 => SPU_REGISTERS.sample_counter.store(0, atomic::Ordering::Release),
        0x08 => SPU_REGISTERS.sample_rate.store(value & SAMPLE_RATE_WRITE_MASK, atomic::Ordering::Release),
        0x0C => SPU_REGISTERS.submission_error.store(0, atomic::Ordering::Release),
        0x10 => {
            SPU_QUEUE_LOCAL.with(|queue| {
                queue.send((0, value)).unwrap();
            });
        },
        0x14 => {
            SPU_QUEUE_LOCAL.with(|queue| {
                queue.send((1, value)).unwrap();
            });
        },
        0x18 => {
            SPU_QUEUE_LOCAL.with(|queue| {
                queue.send((2, value)).unwrap();
            });
        },
        0x1C => {
            SPU_QUEUE_LOCAL.with(|queue| {
                queue.send((3, value)).unwrap();
            });
        },
        _ => return WriteResult::InvalidAddress,
    }
    WriteResult::Ok
}

pub fn spu_write_u16(offset: u32, value: u16) -> WriteResult {
    match offset {
        0x00 => SPU_REGISTERS.run_mode.store((value as u32 & RUN_MODE_WRITE_MASK), atomic::Ordering::Release),
        0x04 => SPU_REGISTERS.sample_counter.store(0, atomic::Ordering::Release),
        0x08 => SPU_REGISTERS.sample_rate.store((value as u32 & SAMPLE_RATE_WRITE_MASK), atomic::Ordering::Release),
        0x0C => SPU_REGISTERS.submission_error.store(0, atomic::Ordering::Release),
        _ => return WriteResult::InvalidAddress
    }
    WriteResult::Ok
}

pub fn spu_write_u8(offset: u32, value: u8) -> WriteResult {
    match offset {
        0x00 => SPU_REGISTERS.run_mode.store((value as u32 & RUN_MODE_WRITE_MASK), atomic::Ordering::Release),
        0x04 => SPU_REGISTERS.sample_counter.store(0, atomic::Ordering::Release),
        0x08 => SPU_REGISTERS.sample_rate.store((value as u32 & SAMPLE_RATE_WRITE_MASK) as u32, atomic::Ordering::Release),
        0x0C => SPU_REGISTERS.submission_error.store(0, atomic::Ordering::Release),
        _ => return WriteResult::InvalidAddress
    }
    WriteResult::Ok
}

pub fn spu_read_u32(offset: u32) -> ReadResult<u32> {
    ReadResult::Ok(match offset {
        0x00 => SPU_REGISTERS.run_mode.load(atomic::Ordering::Acquire),
        0x04 => SPU_REGISTERS.sample_counter.load(atomic::Ordering::Acquire),
        0x08 => SPU_REGISTERS.sample_rate.load(atomic::Ordering::Acquire),
        0x0C => SPU_REGISTERS.submission_error.load(atomic::Ordering::Acquire),
        _ => return ReadResult::InvalidAddress
    })
}

pub fn spu_read_u16(offset: u32) -> ReadResult<u16> {
    spu_read_u32(offset).map(|x| x as u16)
}

pub fn spu_read_u8(offset: u32) -> ReadResult<u8> {
    spu_read_u32(offset).map(|x| x as u8)
}
