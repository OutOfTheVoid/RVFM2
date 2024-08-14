use crate::command_list::*;

pub struct SpuCommands;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpuQueue {
    Queue0 = 0,
    Queue1 = 1,
    Queue2 = 2,
    Queue3 = 3,
}

pub enum SpuCommandBuilderError {
    OutOfSpace
}

pub trait SpuCommandBuilderExt: Sized {
    fn reset_sample_counter(&mut self, reset_value: u32) -> Result<(), SpuCommandBuilderError>;
    fn wait_sample_counter(&mut self, sample_count: u32) -> Result<(), SpuCommandBuilderError>;
    fn write_flag(&mut self, flag_address: u32, value: u32, interrupt: bool) -> Result<(), SpuCommandBuilderError>;
}

impl<C: CommandListCompletion, D: CommandListData<SpuCommands>, Builder: CommandListBuilder<SpuCommands, Data = D, Completion = C>> SpuCommandBuilderExt for Builder {
    fn reset_sample_counter(&mut self, reset_value: u32) -> Result<(), SpuCommandBuilderError> {
        let reset_value_bytes = command_u32_bytes(reset_value);
        let data = &[
            0x00,
            reset_value_bytes[0],
            reset_value_bytes[1],
            reset_value_bytes[2],
            reset_value_bytes[3],
        ];
        if !self.push_command(data) {
			Err(SpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn wait_sample_counter(&mut self, sample_count: u32) -> Result<(), SpuCommandBuilderError> {
        let sample_count_bytes = command_u32_bytes(sample_count);
        let data = &[
            0x01,
            sample_count_bytes[0],
            sample_count_bytes[1],
            sample_count_bytes[2],
            sample_count_bytes[3],
        ];
        if !self.push_command(data) {
			Err(SpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn write_flag(&mut self, flag_address: u32, value: u32, interrupt: bool) -> Result<(), SpuCommandBuilderError> {
        let flag_address_bytes = command_u32_bytes(flag_address);
        let data = &[
            0x02,
            if interrupt { 0x01 } else { 0x00 },
            flag_address_bytes[0],
            flag_address_bytes[1],
            flag_address_bytes[2],
            flag_address_bytes[3],
        ];
        if !self.push_command(data) {
			Err(SpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }
}

impl SpuQueue {
    pub fn submit<Completion: CommandListCompletion, CommandData: CommandListData<SpuCommands>>(&self, command_list: &mut CommandData, completion: Completion) {
        unsafe {
            (completion.raw_ptr()).write_volatile(0);
            command_list.command_list_bytes()[4..8].copy_from_slice(&command_u32_bytes(completion.raw_ptr() as usize as u32));
            ((0x08004_0010 + ((*self as u32) << 2)) as usize as *mut u32).write_volatile(command_list.command_list_bytes().as_ptr() as usize as u32);
        }
    }
}
