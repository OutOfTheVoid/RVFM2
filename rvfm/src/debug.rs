use std::{cell::RefCell, sync::Arc};

use crate::machine::{WriteResult, Machine, ReadResult};

struct DebugSPRegs {
    pub message_addr: u32,
    pub length: u32,
    pub status: u32,
}

impl Default for DebugSPRegs {
    fn default() -> Self {
        Self {
            message_addr: 0,
            length: 0,
            status: 0,
        }
    }
}

const DEBUG_STATUS_CODE_OK: u32 = 0;
const DEBUG_STATUS_CODE_ERROR_MEM: u32 = 1;
const DEBUG_STATUS_CODE_ERROR_UTF: u32 = 2;

thread_local! {
    static DEBUG_REGS: RefCell<DebugSPRegs> = RefCell::new(DebugSPRegs::default());
    static DEBUG_BUFFER: RefCell<Vec<u8>> = RefCell::new(Vec::new());
}

pub fn debug_write_u32(machine: &Arc<Machine>, offset: u32, value: u32) -> WriteResult {
    match offset {
        0  => { DEBUG_REGS.with(|r| r.borrow_mut().message_addr = value); WriteResult::Ok },
        4  => { DEBUG_REGS.with(|r| r.borrow_mut().length       = value); WriteResult::Ok },
        8  => { DEBUG_REGS.with(|r| r.borrow_mut().status = value); WriteResult::Ok },
        12 => { 
            let (addr, len) = DEBUG_REGS.with(|r| {
                (r.borrow().message_addr, r.borrow().length)
            });
            debug_push(machine, addr, len);
            WriteResult::Ok
        },
        16 => {
            debug_print();
            WriteResult::Ok
        }
        _ => WriteResult::InvalidAddress
    }
}

fn debug_push(machine: &Arc<Machine>, addr: u32, length: u32) {
    DEBUG_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        for i in 0..length {
            let byte_addr = addr.wrapping_add(i);
            if Machine::ADDRESS_RANGE_DBG.contains(&byte_addr) {
                DEBUG_REGS.with(|r| r.borrow_mut().status = DEBUG_STATUS_CODE_ERROR_MEM);
                return;
            }
            let byte = match machine.read_u8(byte_addr) {
                ReadResult::Ok(byte) => byte,
                _ => {
                    DEBUG_REGS.with(|r| r.borrow_mut().status = DEBUG_STATUS_CODE_ERROR_MEM);
                    return;
                },
            };
            buffer.push(byte);
        }
    });
}

fn debug_print() {
    DEBUG_BUFFER.with(|buffer| {
        let mut buffer = buffer.borrow_mut();
        let mut swapped_buffer = Vec::new();
        std::mem::swap(&mut *buffer, &mut swapped_buffer);
        match String::from_utf8(swapped_buffer) {
            Ok(message) => {
                println!("DBG: {message}");
                DEBUG_REGS.with(|r| r.borrow_mut().status = DEBUG_STATUS_CODE_OK);
            },
            _ => {
                DEBUG_REGS.with(|r| r.borrow_mut().status = DEBUG_STATUS_CODE_ERROR_UTF);
            }
        }
    });
}

pub fn debug_read_u32(_machine: &Arc<Machine>, offset: u32) -> ReadResult<u32> {
    match offset {
        0  => ReadResult::Ok(DEBUG_REGS.with(|r| r.borrow().length)),
        4  => ReadResult::Ok(DEBUG_REGS.with(|r| r.borrow().length)),
        8  => ReadResult::Ok(DEBUG_REGS.with(|r| r.borrow().length)),
        12 => ReadResult::Ok(0),
        16 => ReadResult::Ok(0),
        _  => ReadResult::InvalidAddress
    }
}

pub fn debug_write_u16(machine: &Arc<Machine>, offset: u32, value: u16) -> WriteResult {
    debug_write_u32(machine, offset, value as u32)
}

pub fn debug_read_u16(machine: &Arc<Machine>, offset: u32) -> ReadResult<u16> {
    debug_read_u32(machine, offset).map(|x| x as u16)
}

pub fn debug_write_u8(machine: &Arc<Machine>, offset: u32, value: u8) -> WriteResult {
    debug_write_u32(machine, offset, value as u32)
}

pub fn debug_read_u8(machine: &Arc<Machine>, offset: u32) -> ReadResult<u8> {
    debug_read_u32(machine, offset).map(|x| x as u8)
}
