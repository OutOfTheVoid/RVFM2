use std::{sync::Arc, ops::{RangeInclusive}};

use crate::{debug::*, gpu::*, ui::main_window::{self, MainWindow}, hart_clock::{clock_read_u8, clock_read_u16, clock_read_u32, clock_write_u8, clock_write_u16, clock_write_u32}, interrupt_controller::{interrupt_controller_read_u8, interrupt_controller_read_u32, interrupt_controller_read_u16, interrupt_controller_write_u8, interrupt_controller_write_u16, interrupt_controller_write_u32}, spu::spu_init};

pub enum WriteResult {
    Ok,
    InvalidAddress,
    ReadOnly,
}

impl WriteResult {
    pub fn is_ok(&self) -> bool {
        match self {
            Self::Ok => true,
            _ => false,
        }
    }
}

pub enum ReadResult<T> {
    Ok(T),
    InvalidAddress
}

impl<T> ReadResult<T> {
    pub fn map<U>(self, map_fn: impl Fn(T) -> U) -> ReadResult<U> {
        match self {
            Self::Ok(x)       => ReadResult::Ok((map_fn)(x)),
            Self::InvalidAddress => ReadResult::InvalidAddress
        }
    }

    pub fn unwrap(self) -> T {
        match self {
            Self::Ok(x)       => x,
            Self::InvalidAddress => panic!("unwrap() called on ReadResult::InvalidAddress!"),
        }
    }
}

pub struct Machine {
    pub ram: *mut u8,
    pub rom: *mut u8,
}

/*
Memory map
==========
0x0000_0000 .. 0x07FF_FFFF = RAM
...
0x8000_0000 .. 0x8000_0010 = Debug Serial Port
0x8001_0000 .. 0x8001_0001 = GPU
...
0xF800_0000 .. 0xFFFF_FFFF = ROM
 */

unsafe impl Send for Machine {}
unsafe impl Sync for Machine {}

#[allow(unused)]
impl Machine {
    pub const ADDRESS_RANGE_RAM: RangeInclusive<u32> = 0x0000_0000 ..= 0x07FF_FFFF;
    pub const ADDRESS_RANGE_DBG: RangeInclusive<u32> = 0x8000_0000 ..= 0x8000_0013;
    pub const ADDRESS_RANGE_GPU: RangeInclusive<u32> = 0x8001_0000 ..= 0x8001_0003;
    pub const ADDRESS_RANGE_CLK: RangeInclusive<u32> = 0x8002_0000 ..= 0x8002_000F;
    pub const ADDRESS_RANGE_INT: RangeInclusive<u32> = 0x8003_0000 ..= 0x8003_0FFF;
    pub const ADDRESS_RANGE_ROM: RangeInclusive<u32> = 0xF800_0000 ..= 0xFFFF_FFFF;

    pub fn new(rom_data: &[u8], main_window: MainWindow) -> Arc<Self> {
        let ram = Box::leak(vec![0u8; 0x800_0000].into_boxed_slice()).as_mut_ptr();
        let rom = Box::leak(vec![0u8; 0x800_0000].into_boxed_slice()).as_mut_ptr();
        unsafe { std::slice::from_raw_parts_mut(rom, rom_data.len().min(0x800_000)).copy_from_slice(rom_data) };
        let machine = Arc::new(Self {
            ram,
            rom,
        });
        gpu_init(&machine, main_window);
        spu_init(&machine);
        machine
    }

    pub fn read_u8(self: &Arc<Self>, addr: u32) -> ReadResult<u8> {
        unsafe {
            match addr {
                0x0000_0000 ..= 0x07FF_FFFF => ReadResult::Ok(self.ram_read(addr)),
                0x8000_0000 ..= 0x8000_0013 => debug_read_u8(self, (addr & 0xF)),
                0x8001_0000                 => gpu_read_u8(0),
                0x8002_0000 ..= 0x8002_000F => clock_read_u8(addr & 0x1F),
                0x8003_0000 ..= 0x8003_0FFF => interrupt_controller_read_u8(addr & 0xFFF),
                0xF800_0000 ..= 0xFFFF_FFFF => ReadResult::Ok(self.rom_read(addr & 0x07FF_FFFF)),
                _ => ReadResult::InvalidAddress
            }
        }
    }

    pub fn read_u16(self: &Arc<Self>, addr: u32) -> ReadResult<u16> {
        unsafe {
            match addr {
                0x0000_0000 ..= 0x07FF_FFFE => ReadResult::Ok(self.ram_read(addr)),
                0x8000_0000 ..= 0x8000_0012 => debug_read_u16(self, (addr & 0xF)),
                0x8001_0000                 => gpu_read_u16(0),
                0x8002_0000 ..= 0x8002_000E => clock_read_u16(addr & 0x1F),
                0x8003_0000 ..= 0x8003_0FFE => interrupt_controller_read_u16(addr & 0xFFF),
                0xF800_0000 ..= 0xFFFF_FFFE => ReadResult::Ok(self.rom_read(addr & 0x07FF_FFFF)),
                _ => ReadResult::InvalidAddress
            }
        }
    }

    pub fn read_u32(self: &Arc<Self>, addr: u32) -> ReadResult<u32> {
        unsafe {
            match addr {
                0x0000_0000 ..= 0x07FF_FFFC => ReadResult::Ok(self.ram_read(addr)),
                0x8000_0000 ..= 0x8000_0010 => debug_read_u32(self, (addr & 0xF)),
                0x8001_0000                 => gpu_read_u32(0),
                0x8002_0000 ..= 0x8002_000C => clock_read_u32(addr & 0x1F),
                0x8003_0000 ..= 0x8003_0FFC => interrupt_controller_read_u32(addr & 0xFFF),
                0xF800_0000 ..= 0xFFFF_FFFC => ReadResult::Ok(self.rom_read(addr & 0x07FF_FFFF)),
                _ => ReadResult::InvalidAddress
            }
        }
    }

    pub fn read_block(self: &Arc<Self>, addr: u32, data: &mut [u8]) -> ReadResult<()> {
        match addr {
            0x0000_0000 ..= 0x07FF_FFFF => {
                if addr.wrapping_add(data.len() as u32) >= 0x0800_0000 {
                    ReadResult::InvalidAddress
                } else {
                    unsafe { data.copy_from_slice(std::slice::from_raw_parts(self.ram.add(addr as usize) as *const u8, data.len())); }
                    ReadResult::Ok(())
                }
            },
            0xF800_0000 ..= 0xFFFF_FFFF => {
                if addr.wrapping_add(data.len() as u32) <= 0xF800_0000 {
                    ReadResult::InvalidAddress
                } else {
                    unsafe { data.copy_from_slice(std::slice::from_raw_parts(self.rom.add((addr & 0x07FF_FFFF) as usize) as *const u8, data.len())); }
                    ReadResult::Ok(())
                }
            },
            _ => ReadResult::InvalidAddress
        }
    }

    pub fn write_u8(self: &Arc<Self>, addr: u32,  value: u8 ) -> WriteResult {
        unsafe {
            match addr {
                0x0000_0000 ..= 0x07FF_FFFF => self.ram_write(addr, value),
                0x8000_0000 ..= 0x8000_0013 => return debug_write_u8(self, addr & 0xF, value),
                0x8001_0000                 => return gpu_write_u8(0, value),
                0x8002_0000 ..= 0x8002_000F => return clock_write_u8(addr & 0x1F, value),
                0x8003_0000 ..= 0x8003_0FFF => return interrupt_controller_write_u8(addr & 0xFFF, value),
                0xF800_0000 ..= 0xFFFF_FFFF => return WriteResult::ReadOnly,
                _ => return WriteResult::InvalidAddress,
            }
            WriteResult::Ok
        }
    }

    pub fn write_u16(self: &Arc<Self>, addr: u32, value: u16) -> WriteResult {
        unsafe { 
            match addr {
                0x0000_0000 ..= 0x07FF_FFFE => self.ram_write(addr, value),
                0x8000_0000 ..= 0x8000_0012 => return debug_write_u16(self, addr & 0xF, value),
                0x8001_0000                 => return gpu_write_u16(0, value),
                0x8002_0000 ..= 0x8002_000E => return clock_write_u16(addr & 0x1F, value),
                0x8003_0000 ..= 0x8003_0FFE => return interrupt_controller_write_u16(addr & 0xFFF, value),
                0xF800_0000 ..= 0xFFFF_FFFE => return WriteResult::ReadOnly,
                _ => return WriteResult::InvalidAddress
            }
        }
        WriteResult::Ok
    }

    pub fn write_u32(self: &Arc<Self>, addr: u32, value: u32) -> WriteResult {
        unsafe {
            match addr {
                0x0000_0000 ..= 0x07FF_FFFC => self.ram_write(addr, value),
                0x8000_0000 ..= 0x8000_0013 => return debug_write_u32(self, addr & 0xF, value),
                0x8001_0000                 => return gpu_write_u32(0, value),
                0x8002_0000 ..= 0x8002_000C => return clock_write_u32(addr & 0x1F, value),
                0x8003_0000 ..= 0x8003_0FFC => return interrupt_controller_write_u32(addr & 0xFFF, value),
                0xF800_0000 ..= 0xFFFF_FFFC => return WriteResult::ReadOnly,
                _ => return WriteResult::InvalidAddress
            }
        }
        WriteResult::Ok
    }

    fn ram_write<T>(self: &Arc<Self>, offset: u32, value: T) {
        unsafe { *(self.ram.add(offset as usize) as *mut T) = value; }
    }

    fn ram_read<T: Copy>(self: &Arc<Self>, offset: u32) -> T {
        unsafe { *(self.ram.add(offset as usize) as *mut T) }
    }

    fn rom_read<T: Copy>(self: &Arc<Self>, offset: u32) -> T {
        unsafe { *(self.rom.add(offset as usize) as *mut T) }
    }
}
