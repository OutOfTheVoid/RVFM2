use bytemuck::checked::cast_slice;

use super::super::Machine;
use std::{sync::Arc, cell::RefCell};

pub struct CommandList {
    pub data: Vec<u8>,
    pub offset: usize,
}

impl CommandList {
    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn read_u8(&self, offset: u32) -> Option<u8> {
        if (offset as usize) < self.data.len() {
            Some(self.data[offset as usize])
        } else {
            None
        }
    }
    
    pub fn read_u16(&self, offset: u32) -> Option<u16> {
        if ((offset + 1) as usize) < self.data.len() {
            Some(cast_slice::<_, u16>(&self.data[offset as usize..(offset + 2) as usize])[0])
        } else {
            None
        }
    }

    pub fn read_u32(&self, offset: u32) -> Option<u32> {
        if ((offset + 3) as usize) < self.data.len() {
            Some(cast_slice::<_, u32>(&self.data[offset as usize..(offset + 4) as usize])[0])
        } else {
            None
        }
    }
}

thread_local! {
    static LIST_POOL: RefCell<Vec<CommandList>> = RefCell::new(Vec::new());
}

const MAX_COMMANDLIST_LEN: u32 = 1024 * 1024;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum CommandListHeaderError {
    HeaderNotInRam,
    ListNotInRam,
    ListTooLong,
}

pub fn parse_commandlist_header(commandlist_addr: u32, machine: &Arc<Machine>) -> Result<CommandList, CommandListHeaderError> {
    if !Machine::ADDRESS_RANGE_RAM.contains(&commandlist_addr) || !Machine::ADDRESS_RANGE_RAM.contains(&(commandlist_addr + 8)) {
        return Err(CommandListHeaderError::HeaderNotInRam);
    }
    std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
    let list_len = machine.read_u32(commandlist_addr).unwrap();
    let transfer_completion_flag = machine.read_u32(commandlist_addr + 4).unwrap();

    if list_len > MAX_COMMANDLIST_LEN {
        return Err(CommandListHeaderError::ListTooLong);
    }
    let list_start = commandlist_addr + 8;
    let list_end = commandlist_addr + 8 + list_len;

    if !Machine::ADDRESS_RANGE_RAM.contains(&list_start) || !Machine::ADDRESS_RANGE_RAM.contains(&list_end) {
        return Err(CommandListHeaderError::ListNotInRam);
    }

    let mut header = LIST_POOL.with(|r| {
        r.borrow_mut().pop()
    }).unwrap_or_else(|| CommandList {
        data: vec![0u8; list_len as usize],
        offset: 0,
    });

    if header.data.len() < list_len as usize {
        header.data.resize(list_len as usize, 0);
    }

    header.offset = 0;
    machine.read_block(list_start, &mut header.data[0..list_len as usize]).unwrap();
    if transfer_completion_flag != 0 {
        println!("writing submission completion!");
        machine.write_u32(transfer_completion_flag, 1);
        std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
    }

    Ok(header)
}

pub fn retire_commandlist(cl: CommandList) {
    LIST_POOL.with(|pool| pool.borrow_mut().push(cl));
}
