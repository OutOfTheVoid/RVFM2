mod oscillator;
mod envelope;
mod sampler;
mod engine;
mod command;
mod param;
mod filter;

use std::{sync::{atomic::{AtomicU32, self}, Arc}, time::Duration};

use cpal::{FromSample, traits::{HostTrait, DeviceTrait, StreamTrait}, SampleRate, StreamConfig, SampleFormat};
use parking_lot::{RwLock, Condvar, Mutex};
use static_init::dynamic;

use crate::machine::Machine;

#[dynamic]
static SPU_REGISTERS: SpuRegisters = SpuRegisters::new();

struct SpuRegisters {
    sample_rate: AtomicU32,
    run_mode: AtomicU32,
    sample_counter: AtomicU32,
}

const RUN_MODE_STOPPED: u32 = 0;
const RUN_MODE_RUN: u32 = 1;
const RUN_MODE_WRITE_MASK: u32 = RUN_MODE_RUN;

const SAMPLE_RATE_16000: u32 = 0;
const SAMPLE_RATE_32000: u32 = 1;
const SAMPLE_RATE_41000: u32 = 2;
const SAMPLE_RATE_48000: u32 = 3;

impl SpuRegisters {
    pub fn new() -> Self {
        Self {
            sample_rate: AtomicU32::new(SAMPLE_RATE_16000),
            run_mode: AtomicU32::new(RUN_MODE_STOPPED),
            sample_counter: AtomicU32::new(0),
        }
    }
}

struct SoundProcessContext {
    pub conversion_buffer: Vec<i16>,
    pub output_sample_rate: f32,
    pub machine: Arc<Machine>,
}

pub fn spu_init(machine: &Arc<Machine>) {
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
        let timeout = Some(Duration::from_millis(200));
        let mut process_context = SoundProcessContext {
            conversion_buffer: Vec::new(),
            output_sample_rate: config.sample_rate.0 as f32,
            machine: machine.clone(),
        };
        let stream_result = match format {
            SampleFormat::F32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<f32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::F64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<f64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I8 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i8>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I16 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i16>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::I64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<i64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U8 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u8>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U16 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u16>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U32 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u32>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            SampleFormat::U64 => device.build_output_stream(&config,move |buffer, _info| sound_process_generic::<u64>(buffer, &mut process_context), |error| println!("SPU: output stream error: {:?}", error), timeout),
            _ => { println!("SPU: unknown audio sample format"); return },
        };
        let stream = match stream_result {
            Ok(stream) => stream,
            Err(error) => {
                println!("SPU: failed to build output stream: {:?}", error);
                return;
            }
        };
        stream.play().expect("SPU: failed to start output stream");
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

    context.conversion_buffer.resize(source_sample_length, 0);
    sound_process(&mut context.conversion_buffer[0..source_sample_length], &context.machine);

    for i in 0..samples.len() {
        let i_spu = ((i >> 1) as f32 * spu_over_output_sample_rate).floor() as usize;
        samples[i] = T::from_sample_(context.conversion_buffer[i_spu + (i & 1)]);
    }
}

fn sound_process(samples: &mut [i16], machine: &Machine) {
    match SPU_REGISTERS.run_mode.load(atomic::Ordering::Acquire) {
        RUN_MODE_STOPPED => {
            samples.fill(0);
        },
        RUN_MODE_RUN => {
            let sample_count = samples.len() / 2;
            for i in 0..sample_count {
                let i_left = (i << 1) | 0;
                let i_right = (i << 1) | 0;
                let (left, right) = spu_render();
                samples[i_left] = left;
                samples[i_right] = right;
                SPU_REGISTERS.sample_counter.fetch_add(1, atomic::Ordering::AcqRel);
            }
        },
        _ => unreachable!()
    }
}

fn spu_render() -> (i16, i16) {
    (0, 0)
}
