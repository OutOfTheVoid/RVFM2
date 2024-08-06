use std::sync::Arc;

use bytemuck::{cast_slice, cast_slice_mut};

use crate::interrupt_controller::INTERRUPT_CONTROLLER;

trait ControlStatusReg {
    fn reset(&mut self);
    fn r(&self) -> CsrReadResult;
    fn w(&mut self, val: u32) -> CsrWriteResult;
    fn rw(&mut self, val: u32) -> CsrReadWriteResult;
    fn rs(&mut self, val: u32) -> CsrReadWriteResult;
    fn rc(&mut self, val: u32) -> CsrReadWriteResult;
}

#[allow(unused)]
enum CsrRefMut<'a> {
    None,
    Constant(u32),
    ReadOnly(&'a u32),
    ReadWrite(&'a mut u32),
    Dynamic(&'a mut dyn ControlStatusReg),
    Shared,
}

#[allow(unused)]
enum CsrRef<'a> {
    None,
    Constant(u32),
    Ref(&'a u32),
    Dynamic(&'a dyn ControlStatusReg),
    Shared
}

pub type CsrReadResult = Result<u32, ()>;
pub type CsrWriteResult = Result<(), ()>;
pub type CsrReadWriteResult = Result<u32, ()>;

#[allow(unused)]
#[derive(Clone)]
pub struct SharedCSRs {
    regs: Arc<SharedCSRRegs>,
}

impl SharedCSRs {
    pub fn new() -> Self {
        Self {
            regs: Arc::new(SharedCSRRegs::new())
        }
    }

    pub fn r(&self, _csr: u16) -> CsrReadResult {
        Err(())
    }

    pub fn w(&self, _csr: u16, _val: u32) -> CsrWriteResult {
        Err(())
    }

    pub fn rw(&self, _csr: u16, _val: u32) -> CsrReadWriteResult {
        Err(())
    }

    pub fn rs(&self, _csr: u16, _val: u32) -> CsrReadWriteResult {
        Err(())
    }

    pub fn rc(&self, _csr: u16, _val: u32) -> CsrReadWriteResult {
        Err(())
    }
}

struct SharedCSRRegs {
}

impl SharedCSRRegs {
    pub fn new() -> Self {
        SharedCSRRegs {
        }
    }
}


pub struct CSRs {
    shared: SharedCSRs,
    mhartid: u32,
    pub mstatus: CsrMStatus,
    pub mtvec: CsrMTVec,
    pub mscratch: u32,
    pub mepc: u32,
    pub mcause: u32,
    pub mtval: u32,
    pub mie: CsrMIE,
    pub mip: CsrMIP,
    pub mcycle: u64,
    pub minstret: u64,
}

impl CSRs {
    pub fn new(hart_id: u32, shared_csrs: &SharedCSRs) -> Self {
        CSRs {
            shared: shared_csrs.clone(),
            mhartid: hart_id,
            mstatus: CsrMStatus::new(),
            mtvec: CsrMTVec::new(),
            mcause: 0,
            mscratch: 0,
            mepc: 0,
            mtval: 0,
            mie: CsrMIE::new(),
            mip: CsrMIP::new(hart_id),
            mcycle: 0,
            minstret: 0,
        }
    }

    pub fn hart_id(&self) -> u32 {
        self.mhartid
    }

    pub fn reset(&mut self) {
        self.mstatus.reset();
        self.mtvec.reset();
        self.mcause = 0;
        self.mscratch = 0;
        self.mepc = 0;
        self.mtval = 0;
        self.mie.reset();
        self.mip.reset();
        self.mcycle = 0;
        self.minstret = 0;
    }

    const MVENDORID: u32 = 0;
    const MARCHID: u32 = 0;
    const MIMPID: u32 = 0;
    const MISA: u32 = 
        1 << 31 |  // RV32
        1 << 5  |  // F
        1 << 8  |  // I
        1 << 12 |  // M
        0;

    fn get_csr_mut(&mut self, csr: u16) -> CsrRefMut<'_> {
        match csr {
            0x300 => CsrRefMut::Dynamic(&mut self.mstatus),
            0x301 => CsrRefMut::Constant(Self::MISA),
            0x304 => CsrRefMut::Dynamic(&mut self.mie),
            0x305 => CsrRefMut::Dynamic(&mut self.mtvec),

            0x340 => CsrRefMut::ReadWrite(&mut self.mscratch),
            0x341 => CsrRefMut::ReadWrite(&mut self.mepc),
            0x342 => CsrRefMut::ReadWrite(&mut self.mcause),
            0x343 => CsrRefMut::ReadWrite(&mut self.mtval),
            0x344 => CsrRefMut::Dynamic(&mut self.mip),

            0xB00 => CsrRefMut::ReadWrite(&mut cast_slice_mut::<_, u32>(std::slice::from_mut(&mut self.mcycle))[0]),
            0xB02 => CsrRefMut::ReadWrite(&mut cast_slice_mut::<_, u32>(std::slice::from_mut(&mut self.minstret))[0]),
            0xB80 => CsrRefMut::ReadWrite(&mut cast_slice_mut::<_, u32>(std::slice::from_mut(&mut self.mcycle))[1]),
            0xB82 => CsrRefMut::ReadWrite(&mut cast_slice_mut::<_, u32>(std::slice::from_mut(&mut self.minstret))[1]),

            0xF11 => CsrRefMut::Constant(Self::MVENDORID),
            0xF12 => CsrRefMut::Constant(Self::MARCHID),
            0xF13 => CsrRefMut::Constant(Self::MIMPID),
            0xF14 => CsrRefMut::ReadOnly(&self.mhartid),

            _ => CsrRefMut::None,
        }
    }

    fn get_csr(&self, csr: u16) -> CsrRef<'_> {
        match csr {
            0x300 => CsrRef::Dynamic(&self.mstatus),
            0x301 => CsrRef::Constant(Self::MISA),
            0x304 => CsrRef::Dynamic(&self.mie),
            0x305 => CsrRef::Dynamic(&self.mtvec),

            0x340 => CsrRef::Ref(&self.mscratch),
            0x341 => CsrRef::Ref(&self.mepc),
            0x342 => CsrRef::Ref(&self.mcause),
            0x343 => CsrRef::Ref(&self.mtval),
            0x344 => CsrRef::Dynamic(&self.mip),

            0xB00 => CsrRef::Ref(&cast_slice::<_, u32>(std::slice::from_ref(&self.mcycle))[0]),
            0xB02 => CsrRef::Ref(&cast_slice::<_, u32>(std::slice::from_ref(&self.minstret))[0]),
            0xB80 => CsrRef::Ref(&cast_slice::<_, u32>(std::slice::from_ref(&self.mcycle))[1]),
            0xB82 => CsrRef::Ref(&cast_slice::<_, u32>(std::slice::from_ref(&self.minstret))[1]),

            0xF11 => CsrRef::Constant(Self::MVENDORID),
            0xF12 => CsrRef::Constant(Self::MARCHID),
            0xF13 => CsrRef::Constant(Self::MIMPID),
            0xF14 => CsrRef::Ref(&self.mhartid),

            _ => CsrRef::None,
        }
    }

    pub fn r(&self, csr: u16) -> CsrReadResult {
        match self.get_csr(csr) {
            CsrRef::None => Err(()),
            CsrRef::Constant(val) => Ok(val),
            CsrRef::Dynamic(csr) => csr.r(),
            CsrRef::Shared => self.shared.r(csr),
            CsrRef::Ref(csr) => Ok(*csr),
        }
    }

    pub fn w(&mut self, csr: u16, val: u32) -> CsrWriteResult {
        match self.get_csr_mut(csr) {
            CsrRefMut::None        |
            CsrRefMut::ReadOnly(_) |
            CsrRefMut::Constant(_) => Err(()),
            CsrRefMut::Dynamic(csr) => csr.w(val),
            CsrRefMut::ReadWrite(csr) => {
                *csr = val;
                Ok(())
            },
            CsrRefMut::Shared => self.shared.w(csr, val)
        }
    }

    pub fn rw(&mut self, csr: u16, val: u32) -> CsrReadWriteResult {
        match self.get_csr_mut(csr) {
            CsrRefMut::None        |
            CsrRefMut::ReadOnly(_) |
            CsrRefMut::Constant(_) => Err(()),
            CsrRefMut::Dynamic(csr) => csr.rw(val),
            CsrRefMut::ReadWrite(csr) => {
                let old_val = *csr;
                *csr = val;
                Ok(old_val)
            },
            CsrRefMut::Shared => self.shared.rw(csr, val)
        }
    }

    pub fn rs(&mut self, csr: u16, val: u32) -> CsrReadWriteResult {
        match self.get_csr_mut(csr) {
            CsrRefMut::None        |
            CsrRefMut::ReadOnly(_) |
            CsrRefMut::Constant(_) => Err(()),
            CsrRefMut::Dynamic(csr) => csr.rs(val),
            CsrRefMut::ReadWrite(csr) => {
                let old_val = *csr;
                *csr |= val;
                Ok(old_val)
            },
            CsrRefMut::Shared => self.shared.rs(csr, val)
        }
    }
    
    pub fn rc(&mut self, csr: u16, val: u32) -> CsrReadWriteResult {
        match self.get_csr_mut(csr) {
            CsrRefMut::None        |
            CsrRefMut::ReadOnly(_) |
            CsrRefMut::Constant(_) => Err(()),
            CsrRefMut::Dynamic(csr) => csr.rc(val),
            CsrRefMut::ReadWrite(csr) => {
                let old_val = *csr;
                *csr &= !val;
                Ok(old_val)
            },
            CsrRefMut::Shared => self.shared.rc(csr, val)
        }
    }

    pub fn interrupts_enabled(&self) -> bool {
        self.mstatus.interrupts_enabled()
    }
}

pub struct CsrMStatus(u32);

impl CsrMStatus {
    const MIE: u32 = 1 << 3;
    const MPIE: u32 = 1 << 7;
    const WRITE_MASK: u32 = Self::MIE | Self::MPIE;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn interrupts_enabled(&self) -> bool {
        (self.0 & Self::MIE) != 0
    }

    pub fn enter_interrupt(&mut self) {
        self.0 = Self::MPIE;
    }

    pub fn exit_interrupt(&mut self) {
        let mpie = (self.0 & Self::MPIE) != 0;
        self.0 = (if mpie { Self::MIE } else { 0 } ) | Self::MPIE;
    }
}

impl ControlStatusReg for CsrMStatus {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn r(&self) -> CsrReadResult {
        Ok(self.0)
    }

    fn w(&mut self, val: u32) -> CsrWriteResult {
        self.0 = val & Self::WRITE_MASK;
        Ok(())
    }

    fn rw(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 = val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rs(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 |= val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rc(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 &= !(val & Self::WRITE_MASK);
        Ok(old_value)
    }
}

pub struct CsrMTVec(u32);

impl CsrMTVec {
    const VECTOR_BASE_BITS: u32 = 0xFFFF_FFFC;
    const MODE_BIT: u32 = 1;
    const WRITE_MASK: u32 = Self::VECTOR_BASE_BITS | Self::MODE_BIT;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn get_vector_address(&self, vector: u32) -> u32 {
        let vectored_mode = self.0 & Self::MODE_BIT != 0;
        let base_address = self.0 & Self::VECTOR_BASE_BITS;
        if vectored_mode {
            base_address + vector * 4
        } else {
            base_address
        }
    }
}

impl ControlStatusReg for CsrMTVec {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn r(&self) -> CsrReadResult {
        Ok(self.0)
    }

    fn w(&mut self, val: u32) -> CsrWriteResult {
        println!("mtvec::w({:08X})", val);
        self.0 = val & Self::WRITE_MASK;
        Ok(())
    }

    fn rw(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 = val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rs(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 |= val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rc(&mut self, val: u32) -> CsrReadWriteResult {
        let old_value = self.0;
        self.0 &= !(val & Self::WRITE_MASK);
        Ok(old_value)
    }
}

pub struct InterruptBits;

impl InterruptBits {
    pub const MSI: u32 = 1 << 3;
    pub const MTI: u32 = 1 << 7;
    pub const MEI: u32 = 1 << 11;
}

pub struct CsrMIE(u32);

impl CsrMIE {
    const WRITE_MASK: u32 = InterruptBits::MSI | InterruptBits::MTI | InterruptBits::MEI;

    pub fn new() -> Self {
        Self(0)
    }

    pub fn value(&self) -> u32 {
        self.0
    }
}

impl ControlStatusReg for CsrMIE {
    fn reset(&mut self) {
        self.0 = 0;
    }

    fn r(&self) -> CsrReadResult {
        Ok(self.0)
    }

    fn w(&mut self, val: u32) -> CsrWriteResult {
        println!("csrmie: w({:X})", val);
        self.0 = val & Self::WRITE_MASK;
        Ok(())
    }

    fn rw(&mut self, val: u32) -> CsrReadWriteResult {
        println!("csrmie: rw({:X}) -> {:X}", val, self.0);
        let old_value = self.0;
        self.0 = val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rs(&mut self, val: u32) -> CsrReadWriteResult {
        println!("csrmie: rs({:X}) -> {:X}", val, self.0);
        let old_value = self.0;
        self.0 |= val & Self::WRITE_MASK;
        Ok(old_value)
    }

    fn rc(&mut self, val: u32) -> CsrReadWriteResult {
        println!("csrmie: rc({:X}) -> {:X}", val, self.0);
        let old_value = self.0;
        self.0 &= !(val & Self::WRITE_MASK);
        Ok(old_value)
    }
}


pub struct CsrMIP(u32);

impl CsrMIP {
    pub fn new(hart_id: u32) -> Self {
        Self(hart_id)
    }

    pub fn value(&self) -> u32 {
        INTERRUPT_CONTROLLER.mip(self.0)
    }
}

impl ControlStatusReg for CsrMIP {
    fn reset(&mut self) {
    }

    fn r(&self) -> CsrReadResult {
        Ok(self.value())
    }

    fn w(&mut self, _val: u32) -> CsrWriteResult {
        Ok(())
    }

    fn rw(&mut self, _val: u32) -> CsrReadWriteResult {
        Ok(self.value())
    }

    fn rs(&mut self, _val: u32) -> CsrReadWriteResult {
        Ok(self.value())
    }

    fn rc(&mut self, _val: u32) -> CsrReadWriteResult {
        Ok(self.value())
    }
}
