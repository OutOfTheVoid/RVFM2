use crate::command_list::*;

pub struct SpuCommands;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum SpuQueue {
    Queue0 = 0,
    Queue1 = 1,
    Queue2 = 2,
    Queue3 = 3,
}

pub trait SpuCommandBuilderExt: Sized {
    fn reset_sample_counter(self, reset_value: u32) -> Result<Self, ()>;
    fn wait_sample_counter(self, sample_count: u32) -> Result<Self, ()>;
    fn write_flag(self, flag_address: u32, value: u32, interrupt: bool) -> Result<Self, ()>;
}

impl SpuCommandBuilderExt for CommandListBuilder<'_, SpuCommands> {
    fn reset_sample_counter(self, reset_value: u32) -> Result<Self, ()> {
        let reset_value_bytes = command_u32_bytes(reset_value);
        let data = &[
            0x00,
            reset_value_bytes[0],
            reset_value_bytes[1],
            reset_value_bytes[2],
            reset_value_bytes[3],
        ];
        self.push_command(data)
    }

    fn wait_sample_counter(self, sample_count: u32) -> Result<Self, ()> {
        let sample_count_bytes = command_u32_bytes(sample_count);
        let data = &[
            0x01,
            sample_count_bytes[0],
            sample_count_bytes[1],
            sample_count_bytes[2],
            sample_count_bytes[3],
        ];
        self.push_command(data)
    }

    fn write_flag(self, flag_address: u32, value: u32, interrupt: bool) -> Result<Self, ()> {
        let flag_address_bytes = command_u32_bytes(flag_address);
        let value_bytes = command_u32_bytes(value);
        let data = &[
            0x02,
            if interrupt { 0x01 } else { 0x00 },
            flag_address_bytes[0],
            flag_address_bytes[1],
            flag_address_bytes[2],
            flag_address_bytes[3],
        ];
        self.push_command(data)
    }
}

impl SpuQueue {
    pub fn submit<'l, 'c: 'l>(&self, command_list: CommandList<'l, SpuCommands>, completion: &'c mut u32) -> CommandListCompletion<'c> {
        unsafe { (completion as *mut u32).write_volatile(0); }
        command_list.0[4..8].copy_from_slice(&command_u32_bytes(completion as *mut u32 as usize as u32));
        unsafe { ((0x08004_0010 + ((*self as u32) << 2)) as usize as *mut u32).write_volatile(command_list.0.as_ptr() as usize as u32); }
        CommandListCompletion(completion)
    }
}
