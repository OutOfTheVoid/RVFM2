pub mod decoder;
pub mod csrs;
mod test;

use std::sync::Arc;

use csrs::*;
use decoder::*;

use crate::machine::{Machine, ReadResult, WriteResult};

#[derive(Copy, Clone, Debug)]
pub enum InterruptCause {
    Software,
    Timer,
    External,
    InstructionAlignment(u32),
    InstructionAccess(u32),
    IllegalInstruction(u32),
    Breakpoint(u32),
    LoadAddressAlignment(u32),
    LoadAccess(u32),
    AtomicAlignment(u32),
    AtlimcAccess(u32),
    ECall,
}

impl InterruptCause {
    pub fn to_vector(&self) -> u32 {
        match *self {
            Self::External => 11,
            Self::Software => 3,
            Self::Timer => 7,
            _ => 0
        }
    }

    pub fn to_mcause(&self) -> u32 {
        match *self {
            Self::Software                   => 0x8000_0003,
            Self::Timer                      => 0x8000_0007,
            Self::External                   => 0x8000_000B,
            Self::InstructionAlignment(_)    => 0,
            Self::InstructionAccess(_)       => 1,
            Self::IllegalInstruction(_)      => 2,
            Self::Breakpoint(_)              => 3,
            Self::LoadAddressAlignment(_)    => 4,
            Self::LoadAccess(_)              => 5,
            Self::AtomicAlignment(_)         => 6,
            Self::AtlimcAccess(_)            => 7,
            Self::ECall                      => 11,
        }
    }

    pub fn is_exception(&self) -> bool {
        (self.to_mcause() & 0x8000_0000) == 0
    }

    pub fn to_mtval(&self) -> Option<u32> {
        match *self {
            Self::InstructionAlignment(x) |
            Self::InstructionAccess(x)    |
            Self::IllegalInstruction(x)   |
            Self::Breakpoint(x)           |
            Self::LoadAddressAlignment(x) |
            Self::LoadAccess(x)           |
            Self::AtomicAlignment(x)      |
            Self::AtlimcAccess(x)         => Some(x),
            _                                  => None
        }
    }
}

pub struct Hart {
    pub pc: u32,
    pub gprs: [u32; 32],
    pub csrs: CSRs,
    pub machine: Arc<Machine>,
}

#[derive(Debug, Copy, Clone)]
pub enum StepState {
    BusError,
    InstructionError,
    Run,
    WaitForInterrupt,
}

#[derive(Copy, Clone, Debug)]
enum BusAccessSize {
    S8,
    S16,
    S32
}

impl BusAccessSize {
    pub fn to_str(&self) -> &'static str {
        match self {
            BusAccessSize::S8  => "8",
            BusAccessSize::S16 => "16",
            BusAccessSize::S32 => "32",
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum BusErrorType {
    Fetch,
    Read(BusAccessSize),
    Write(BusAccessSize),
}

#[allow(unused)]
impl Hart {
    pub fn new(reset_addr: u32, hart_id: u32, shared_csrs: &SharedCSRs, machine: &Arc<Machine>) -> Self {
        Hart {
            pc: reset_addr,
            gprs: [0u32; 32],
            csrs: CSRs::new(hart_id, shared_csrs),
            machine: machine.clone()
        }
    }

    pub fn reset(&mut self, reset_addr: u32) {
        self.pc = reset_addr;
        self.gprs = [0u32; 32];
        self.csrs.reset();
    }

    fn take_interrupt(&mut self, cause: InterruptCause) {
        println!("take_interrupt({:?})", cause);
        self.csrs.mepc = self.pc;
        self.csrs.mcause = cause.to_mcause();
        if let Some(mtval) = cause.to_mtval() {
            self.csrs.mtval = mtval;
        }
        self.csrs.mstatus.enter_interrupt();
        self.pc = self.csrs.mtvec.get_vector_address(cause.to_vector());
        println!("interrupt vector address: {:08X}", self.pc);
    }

    pub fn set_gpr(&mut self, gpr: u8, value: u32) {
        match gpr {
            0 => {},
            _ => self.gprs[gpr as usize] = value,
        }
    }

    pub fn trace_regs(&self) -> String {
        let pc = self.pc;
        let opcode = match self.machine.read_u32(pc) {
            ReadResult::Ok(opcode) => opcode,
            ReadResult::InvalidAddress => return "error".to_string(),
        };
        let op = Rv32Op::decode(opcode);
        format!(
            r#"
            
            x1:  {:08X}, x2:  {:08X}, x3:  {:08X}, x4:  {:08X}
            x5:  {:08X}, x6:  {:08X}, x7:  {:08X}, x8:  {:08X}
            x9:  {:08X}, x10: {:08X}, x11: {:08X}, x12: {:08X}
            x13: {:08X}, x14: {:08X}, x15: {:08X}, x16: {:08X}
            
            pc: {:#010X} - {:08X}: {:?}"#, 
            self.gprs[1], self.gprs[2], self.gprs[3], self.gprs[4],
            self.gprs[5], self.gprs[6], self.gprs[7], self.gprs[8],
            self.gprs[9], self.gprs[10], self.gprs[11], self.gprs[12],
            self.gprs[13], self.gprs[14], self.gprs[15], self.gprs[16],
            pc, opcode, op,
        )
    }

    fn interrupt_check(&mut self) {
        let active_interrupts = self.csrs.mie.value() & self.csrs.mip.value();
        if active_interrupts & InterruptBits::MEI != 0 {
            self.take_interrupt(InterruptCause::External);
        } else if active_interrupts & InterruptBits::MSI != 0 {
            self.take_interrupt(InterruptCause::Software);
        } else if active_interrupts & InterruptBits::MTI != 0 {
            self.take_interrupt(InterruptCause::Timer);
        }
    }

    pub fn single_step<const TRACE: bool>(&mut self) -> StepState {
        let result = self.single_step_internal::<TRACE>();
        self.csrs.mcycle += 1;
        result
    }

    pub fn single_step_internal<const TRACE: bool>(&mut self) -> StepState {
        if self.csrs.mstatus.interrupts_enabled() {
            self.interrupt_check();
        }
        let instruction_addr = self.pc;
        let opcode_value = match self.machine.read_u32(instruction_addr) {
            ReadResult::Ok(value) => value,
            ReadResult::InvalidAddress => return self.bus_error(instruction_addr, BusErrorType::Fetch),
        };
        let op = Rv32Op::decode(opcode_value);
        let pc = match op {
            Rv32Op::Lui { immediate, rd } => {
                self.set_gpr(rd, immediate);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Auipc { immediate, rd } => {
                self.set_gpr(rd, instruction_addr.wrapping_add(immediate));
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Jal { immediate, rd } => {
                let ret_addr = instruction_addr.wrapping_add(4);
                let jump_addr = instruction_addr.wrapping_add(immediate);
                self.set_gpr(rd, ret_addr);
                jump_addr
            },
            Rv32Op::Jalr { immediate, rd, rs1 } => {
                let ret_addr = instruction_addr.wrapping_add(4);
                let jump_addr = (immediate as u32).wrapping_add(self.gprs[rs1 as usize]);
                self.set_gpr(rd, ret_addr);
                jump_addr
            },
            Rv32Op::Beq { immediate, rs2, rs1 } => {
                match self.gprs[rs1 as usize] == self.gprs[rs2 as usize] {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Bne { immediate, rs2, rs1 } => {
                match self.gprs[rs1 as usize] != self.gprs[rs2 as usize] {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Blt { immediate, rs2, rs1 } => {
                match (self.gprs[rs1 as usize] as i32) < (self.gprs[rs2 as usize] as i32) {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Bge { immediate, rs2, rs1 } => {
                match (self.gprs[rs1 as usize] as i32) >= (self.gprs[rs2 as usize] as i32) {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Bltu { immediate, rs2, rs1 } => {
                match self.gprs[rs1 as usize] < self.gprs[rs2 as usize] {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Bgeu { immediate, rs2, rs1 } => {
                match self.gprs[rs1 as usize] >= self.gprs[rs2 as usize] {
                    true  => instruction_addr.wrapping_add(immediate as u32),
                    false => instruction_addr.wrapping_add(4), 
                }
            },
            Rv32Op::Lb { immediate, rd, rs1 } => {
                let load_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let value = match self.machine.read_u8(load_addr) {
                    ReadResult::Ok(value)  => value as i8 as i32 as u32,
                    ReadResult::InvalidAddress => return self.bus_error(load_addr, BusErrorType::Read(BusAccessSize::S8))
                };
                self.set_gpr(rd, value);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Lh { immediate, rd, rs1 } => {
                let load_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let value = match self.machine.read_u16(load_addr) {
                    ReadResult::Ok(value)  => value as i16 as i32 as u32,
                    ReadResult::InvalidAddress => return self.bus_error(load_addr, BusErrorType::Read(BusAccessSize::S16))
                };
                self.set_gpr(rd, value);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Lw { immediate, rd, rs1 } => {
                let load_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let value = match self.machine.read_u32(load_addr) {
                    ReadResult::Ok(value)  => value,
                    ReadResult::InvalidAddress => return self.bus_error(load_addr, BusErrorType::Read(BusAccessSize::S32))
                };
                self.set_gpr(rd, value);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Lbu { immediate, rd, rs1 } => {
                let load_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let value = match self.machine.read_u8(load_addr) {
                    ReadResult::Ok(value)  => value as u32,
                    ReadResult::InvalidAddress => return self.bus_error(load_addr, BusErrorType::Read(BusAccessSize::S8))
                };
                self.set_gpr(rd, value);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Lhu { immediate, rd, rs1 } => {
                let load_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let value = match self.machine.read_u16(load_addr) {
                    ReadResult::Ok(value)  => value as u32,
                    ReadResult::InvalidAddress => return self.bus_error(load_addr, BusErrorType::Read(BusAccessSize::S16))
                };
                self.set_gpr(rd, value);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sb { immediate, rs1, rs2 } => {
                let store_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let store_value = self.gprs[rs2 as usize] as u8;
                match self.machine.write_u8(store_addr, store_value) {
                    WriteResult::InvalidAddress |
                    WriteResult::ReadOnly => return self.bus_error(store_addr, BusErrorType::Write(BusAccessSize::S8)),
                    _ => {}
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sh { immediate, rs1, rs2 } => {
                let store_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let store_value = self.gprs[rs2 as usize] as u16;
                match self.machine.write_u16(store_addr, store_value) {
                    WriteResult::InvalidAddress |
                    WriteResult::ReadOnly => return self.bus_error(store_addr, BusErrorType::Write(BusAccessSize::S16)),
                    _ => {}
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sw { immediate, rs1, rs2 } => {
                let store_addr = self.gprs[rs1 as usize].wrapping_add(immediate as u32);
                let store_value = self.gprs[rs2 as usize];
                match self.machine.write_u32(store_addr, store_value) {
                    WriteResult::InvalidAddress |
                    WriteResult::ReadOnly => return self.bus_error(store_addr, BusErrorType::Write(BusAccessSize::S32)),
                    _ => {}
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Addi{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = (a as i32).wrapping_add(b) as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Slti{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = (a as i32) < b;
                self.set_gpr(rd, if result {1} else {0});
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sltiu{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = a < b;
                self.set_gpr(rd, if result {1} else {0});
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Xori{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = a ^ b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Ori{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = a | b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Andi{immediate, rd, rs1} => {
                let a = self.gprs[rs1 as usize];
                let b = immediate;
                let result = a & b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Slli { shamt, rd, rs1 } => {
                let a = self.gprs[rs1 as usize];
                let b = shamt as u32;
                let result = a << b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Srli { shamt, rd, rs1 } => {
                let a = self.gprs[rs1 as usize];
                let b = shamt as u32;
                let result = a >> b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Srai { shamt, rd, rs1 } => {
                let a = self.gprs[rs1 as usize] as i32;
                let b = shamt as u32;
                let result = a >> b;
                self.set_gpr(rd, result as u32);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Add { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a.wrapping_add(b);
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sub { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a.wrapping_sub(b);
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sll { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a << b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Srl { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a >> b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sra { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as i32;
                let b = self.gprs[rs2 as usize];
                let result = a << b;
                self.set_gpr(rd, result as u32);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Slt { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as i32;
                let b = self.gprs[rs2 as usize] as i32;
                let result = a < b;
                self.set_gpr(rd, if result {1} else {0});
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Sltu { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a < b;
                self.set_gpr(rd, if result {1} else {0});
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Xor { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a ^ b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::And { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a & b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Or { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a | b;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Fence { predecessor, successor } => {
                let (p_r, p_w) = (predecessor.read, predecessor.write);
                let (s_r, s_w) = (successor.read, successor.write);
                match (p_r, p_w, s_r, s_w) {
                    (false, false, _, _) |
                    (_, _, false, false) => (),
                    (true, true, _, _) |
                    (_, _, true, true) |
                    (true, _, true, _) |
                    (_, true, _, true) => std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst),
                    (true, _, _, true) => std::sync::atomic::fence(std::sync::atomic::Ordering::Release),
                    (_, true, true, _) => std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire),
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Fencei => {
                self.ifence();
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrw { csr, rd, rs1 } => {
                match rd {
                    0 => {
                        let val = self.gprs[rs1 as usize];
                        if self.csrs.w(csr, val).is_err() {
                            return self.instruction_error(instruction_addr)
                        }
                    },
                    rd => {
                        let val = self.gprs[rs1 as usize];
                        match self.csrs.rw(csr, val) {
                            Ok(val) => self.set_gpr(rd, val),
                            Err(()) => return self.instruction_error(instruction_addr),
                        }
                    }
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrs { csr, rd, rs1 } => {
                let result = match rs1 {
                    0 => {
                        self.csrs.r(csr)
                    },
                    rs1 => {
                        let value = self.gprs[rs1 as usize];
                        self.csrs.rs(csr, value)
                    }
                };
                match result {
                    Ok(val) => self.set_gpr(rd, val),
                    Err(()) => return self.instruction_error(instruction_addr)
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrc { csr, rd, rs1 } => {
                let result = match rs1 {
                    0 => {
                        self.csrs.r(csr)
                    },
                    rs1 => {
                        let value = self.gprs[rs1 as usize];
                        self.csrs.rc(csr, value)
                    }
                };
                match result {
                    Ok(val) => self.set_gpr(rd, val),
                    Err(()) => return self.instruction_error(instruction_addr),
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrwi { csr, rd, immediate } => {
                let val = immediate as u32;
                match rd {
                    0 => {
                        if self.csrs.w(csr, val).is_err() {
                            return self.instruction_error(instruction_addr)
                        }
                    }
                    rd => {
                        match self.csrs.rw(csr, val) {
                            Ok(val) => self.set_gpr(rd, val),
                            Err(()) => return self.instruction_error(instruction_addr),
                        }
                    }
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrsi { csr, rd, immediate } => {
                let value = immediate as u32;
                let result = match value {
                    0 => self.csrs.r(csr),
                    val => self.csrs.rs(csr, val),
                };
                match result {
                    Ok(val) => self.set_gpr(rd, val),
                    Err(()) => return self.instruction_error(instruction_addr)
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Csrrci { csr, rd, immediate } => {
                let value = immediate as u32;
                let result = match value {
                    0 => self.csrs.r(csr),
                    val => self.csrs.rc(csr, val),
                };
                match result {
                    Ok(val) => self.set_gpr(rd, val),
                    Err(()) => return self.instruction_error(instruction_addr)
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Mul { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = a.wrapping_mul(b);
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Mulh { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as i32 as i64;
                let b = self.gprs[rs2 as usize] as i32 as i64;
                let result = ((a * b) as i64 >> 32) as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Mulhu { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as u64;
                let b = self.gprs[rs2 as usize] as u64;
                let result = ((a * b) >> 32) as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Mulhsu { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as u64 as i64;
                let b = self.gprs[rs2 as usize] as i32 as i64;
                let result = ((a * b) >> 32) as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Div { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as i32;
                let b = self.gprs[rs2 as usize] as i32;
                let result = match (a, b) {
                    (std::i32::MIN, -1) => std::i32::MIN,
                    (_, 0) => -1,
                    (_, _) => a / b
                } as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Divu { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = match b {
                    0 => 0xFFFF_FFFF,
                    _ => a / b
                };
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Rem { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize] as i32;
                let b = self.gprs[rs2 as usize] as i32;
                let result = match (a, b) {
                    (std::i32::MIN, -1) => std::i32::MIN,
                    (_, 0) => a,
                    (_, _) => a.wrapping_rem(b)
                } as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Remu { rd, rs1, rs2 } => {
                let a = self.gprs[rs1 as usize];
                let b = self.gprs[rs2 as usize];
                let result = match b {
                    0 => a,
                    _ => a.wrapping_rem(b)
                } as u32;
                self.set_gpr(rd, result);
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Fence { predecessor, successor } => {
                let FenceOps {
                    read: read_p, write: write_p, ..
                } = predecessor;
                let FenceOps {
                    read: read_s, write: write_s, ..
                } = successor;
                let acquire = read_s | write_p;
                let release = write_s | read_p;
                match (acquire, release) {
                    (true, false)  => std::sync::atomic::fence(std::sync::atomic::Ordering::Acquire),
                    (false, true)  => std::sync::atomic::fence(std::sync::atomic::Ordering::Release),
                    (true, true)   => std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel),
                    (false, false) => {},
                }
                instruction_addr.wrapping_add(4)
            },
            Rv32Op::Wfi => {
                if TRACE {
                    println!("pc: {:08X} - {:08X}: Wfi => WAIT FOR INTERRUPT...", instruction_addr, opcode_value);
                }
                self.csrs.minstret += 1;
                return self.wfi()
            },
            Rv32Op::Mret => {
                self.csrs.mstatus.exit_interrupt();
                self.csrs.mepc
            },
            _ => return self.unimplemend_instruction(instruction_addr, opcode_value, op),
        };
        self.pc = pc;
        if TRACE {
            println!(
r#"@ {:#010X}: {:?}

x1:  {:08X}, x2:  {:08X}, x3:  {:08X}, x4:  {:08X}
x5:  {:08X}, x6:  {:08X}, x7:  {:08X}, x8:  {:08X}
x9:  {:08X}, x10: {:08X}, x11: {:08X}, x12: {:08X}
x13: {:08X}, x14: {:08X}, x15: {:08X}, x16: {:08X}

pc:  {:08X}
"#,
instruction_addr,
op,
self.gprs[1],  self.gprs[2],  self.gprs[3],  self.gprs[4],
self.gprs[5],  self.gprs[6],  self.gprs[7],  self.gprs[8],
self.gprs[9],  self.gprs[10], self.gprs[11], self.gprs[12],
self.gprs[13], self.gprs[14], self.gprs[15], self.gprs[16],
pc,
            );
        }
        self.csrs.minstret += 1;
        StepState::Run
    }

    fn bus_error(&mut self, addr: u32, kind: BusErrorType) -> StepState {
        match kind {
            BusErrorType::Fetch => println!("BUS ERROR (FETCH) AT {:#010X}", addr),
            BusErrorType::Read(size) => println!("BUS ERROR (READ {}) AT {:#010X}", size.to_str(), addr),
            BusErrorType::Write(size) => println!("BUS ERROR (WRITE {}) AT {:#010X}", size.to_str(), addr),
        }
        StepState::BusError
    }

    fn instruction_error(&mut self, addr: u32) -> StepState {
        StepState::InstructionError
    }

    fn unimplemend_instruction(&mut self, addr: u32, opcode: u32, op: Rv32Op) -> StepState {
        println!("UNIMPLEMENTED INSTRUCTION AT {:#010X}: {:#010X} - {:?}", addr, opcode, op);
        StepState::InstructionError
    }

    fn wfi(&mut self) -> StepState {
        StepState::WaitForInterrupt
    }

    fn ifence(&mut self) {
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst)
    }
}   
