use std::collections::VecDeque;
use std::sync::Arc;
use crate::Machine;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LoopMode {
    Infinite,
    Finite(u32),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ChannelCount {
    Stereo,
    Mono,
}

pub struct Sampler {
    channel_count: ChannelCount,
    start_address: u32,
    sample_count: u32,

    running: bool,
    index: u32,

    loop_mode: LoopMode,
    loop_count: u32,

    status_request_queue: VecDeque<u32>,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SamplerCommand {
    Setup {
        channel_count: ChannelCount,
        sample_count: u32,
        start_address: u32,
    },
    SetLoopMode(LoopMode),
    Start,
    Continue,
    Pause,
    GetStatus(u32),
}

impl Sampler {
    pub fn new() -> Self {
        Self {
            running: false,

            channel_count: ChannelCount::Mono,
            start_address: 0,
            sample_count: 0,

            index: 0,
            loop_mode: LoopMode::Finite(0),
            loop_count: 0,

            status_request_queue: VecDeque::new()
        }
    }

    pub fn send_command(&mut self, command: SamplerCommand) {
        match command {
            SamplerCommand::Setup {
                channel_count,
                sample_count,
                start_address
            } => {
                self.channel_count = channel_count;
                self.sample_count = sample_count;
                self.start_address = start_address;
            },
            SamplerCommand::SetLoopMode(loop_mode) => {
                self.loop_mode = loop_mode;
            },
            SamplerCommand::Start => {
                self.running = true;
                self.index = 0;
                self.loop_count = 0;
            },
            SamplerCommand::Continue => {
                self.running = true;
            },
            SamplerCommand::Pause => {
                self.running = false;
            },
            SamplerCommand::GetStatus(status_struct_addr) => {
                self.status_request_queue.push_back(status_struct_addr);
            },
        }
    }

    fn write_status(running: bool, loop_count: u32, index: u32, addr: u32, machine: &Arc<Machine>) {
        if !machine.write_u32(addr + 4, if running { 1 } else { 0 }).is_ok() { return };
        if !machine.write_u32(addr + 8, index).is_ok() { return };
        if !machine.write_u32(addr + 12, loop_count).is_ok() { return };
        if !machine.write_u32(addr, 1).is_ok() { return };
    }

    pub fn process(&mut self, machine: &Arc<Machine>) -> (i16, i16) {
        if let Some(status_struct_addr) = self.status_request_queue.pop_front() {
            Self::write_status(self.running, self.loop_count, self.index, status_struct_addr, machine);
        }
        if self.running {
            if self.index >= self.sample_count {
                self.loop_count += 1;
                match self.loop_mode {
                    LoopMode::Finite(count) => {
                        if self.loop_count > count {
                            self.running = false;
                            return (0, 0);
                        }
                    },
                    _ => {}
                }
                self.index = 0;
            }
            let (l, r) = match self.channel_count {
                ChannelCount::Mono => {
                    let x = machine.read_u16(self.start_address + (self.index << 1)).unwrap_or(0) as i16;
                    (x, x)
                },
                ChannelCount::Stereo => {
                    let l = machine.read_u16(self.start_address + (self.index << 2) + 0).unwrap_or(0) as i16;
                    let r = machine.read_u16(self.start_address + (self.index << 2) + 2).unwrap_or(0) as i16;
                    (l, r)
                }
            };
            self.index += 1;
            (l, r)
        } else {
            (0, 0)
        }
    }
}
