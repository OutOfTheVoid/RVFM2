mod core;
mod command_list;
mod command;
pub mod types;
mod texture;
mod buffer;

use std::{sync::{mpsc::{self, Receiver, TryRecvError}, Arc}};
use parking_lot::Mutex;
use static_init::dynamic;
use crate::{machine::{WriteResult, ReadResult}, ui::main_window::MainWindow};

use self::{command_list::parse_commandlist_header, core::Core};

use super::Machine;

struct GpuQueue {
    pub rx: Option<mpsc::Receiver<u32>>,
    pub tx: mpsc::Sender<u32>,
}

impl GpuQueue {
    pub fn take_rx(&mut self) -> mpsc::Receiver<u32> {
        self.rx.take().unwrap()
    }

    pub fn make_tx(&self) -> mpsc::Sender<u32>{
        self.tx.clone()
    }

    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        Self {
            rx: Some(rx),
            tx
        }
    }
}

unsafe impl Send for GpuQueue {}

#[dynamic]
static GPU_QUEUE: Mutex<GpuQueue> = Mutex::new(GpuQueue::new());

thread_local! {
    static GPU_QUEUE_LOCAL: mpsc::Sender<u32> = GPU_QUEUE.lock().make_tx();
}

pub fn gpu_init(machine: &Arc<Machine>, main_window: MainWindow) {
    let machine = machine.clone();
    let queue = GPU_QUEUE.lock().take_rx();
    std::thread::spawn(move || {
        gpu_thread(queue, machine, main_window);
    });
}

fn gpu_thread(queue: Receiver<u32>, machine: Arc<Machine>, main_window: MainWindow) {
    let mut core = Core::new();
    loop {
        match queue.recv() {
            Ok(commandlist_addr) => {
                match parse_commandlist_header(commandlist_addr, &machine) {
                    Ok(command_list) => {
                        core.add_command_list(command_list);
                    },
                    Err(error) => {
                        println!("GPU: ERROR: Failed command list submission ({:#010X}): {:?}", commandlist_addr, error);
                    },
                }
            },
            Err(_) => {
                return;
            }
        };
        'receive: loop {
            match queue.try_recv() {
                Ok(commandlist_addr) => {
                    match parse_commandlist_header(commandlist_addr, &machine) {
                        Ok(command_list) => {
                            core.add_command_list(command_list);
                        },
                        Err(error) => {
                            println!("GPU: ERROR: Failed command list submission ({:#010X}): {:?}", commandlist_addr, error);
                        },
                    }
                },
                Err(TryRecvError::Empty) => break 'receive,
                Err(TryRecvError::Disconnected) => return,
            }
        }
        core.process(&machine, &main_window);
    }
}

pub fn gpu_write_u32(offset: u32, value: u32) -> WriteResult {
    match offset {
        0 => {
            GPU_QUEUE_LOCAL.with(|queue| {
                queue.send(value).unwrap();
            });
            WriteResult::Ok
        },
        _ => WriteResult::InvalidAddress,
    }
}

pub fn gpu_write_u16(offset: u32, value: u16) -> WriteResult {
    gpu_write_u32(offset, value as u32)
}

pub fn gpu_write_u8(offset: u32, value: u8) -> WriteResult {
    gpu_write_u32(offset, value as u32)
}

pub fn gpu_read_u32(offset: u32) -> ReadResult<u32> {
    match offset {
        0 => ReadResult::Ok(0),
        _ => ReadResult::InvalidAddress
    }
}

pub fn gpu_read_u16(offset: u32) -> ReadResult<u16> {
    gpu_read_u32(offset).map(|x| x as u16)
}

pub fn gpu_read_u8(offset: u32) -> ReadResult<u8> {
    gpu_read_u32(offset).map(|x: u32| x as u8)
}

