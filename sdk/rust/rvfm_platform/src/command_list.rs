use core::{marker::PhantomData, sync::atomic::{self, AtomicU32}};

pub const COMMANDLIST_HEADER_LENGTH: usize = 8;

pub trait CommandListBuilder<'c, 'd, Commands>: Sized {
    type Data: CommandListData<'d, Commands>;
    type Completion: CommandListCompletion<'c>;

    fn push_command(&mut self, command_bytes: &[u8]) -> bool;
    fn finish(self) -> Self::Data;
}

pub trait CommandListData<'d, Command>: Sized {
    fn command_list_bytes(&mut self) -> &mut [u8];
}

pub trait CommandListCompletion<'c>: Sized {
    unsafe fn raw_ptr(&self) -> *mut u32;

    fn reset(&self, value: u32) {
        unsafe { AtomicU32::from_ptr(self.raw_ptr()).store(value, atomic::Ordering::Release); }
    }

    fn wait_nonzero(&self) {
        loop { 
            let value = unsafe {
                AtomicU32::from_ptr(self.raw_ptr()).load(atomic::Ordering::Acquire)
            };
            if value != 0 {
                break;
            }
        }
    }

    fn wait_greater_or_equal(&self, target: u32) {
        loop { 
            let value = unsafe {
                AtomicU32::from_ptr(self.raw_ptr()).load(atomic::Ordering::Acquire)
            };
            if value >= target {
                break;
            }
        }
    }

    fn read(&self) -> u32 {
        unsafe { AtomicU32::from_ptr(self.raw_ptr()).load(atomic::Ordering::Acquire) }
    }
}

pub struct StaticCommandList<'a, Commands>(pub(crate) &'a mut [u8], PhantomData<Commands>);

impl<'d, Commands> CommandListData<'d, Commands> for StaticCommandList<'d, Commands> {
    fn command_list_bytes(&mut self) -> &mut [u8] {
        self.0
    }
}

pub struct StaticCommandListBuilder<'c, 'd, Commands> {
    buffer: &'d mut [u8],
    offset: usize,
    _phantom_command: PhantomData<(Commands, &'c())>,
}

impl<'c, 'd, Commands> StaticCommandListBuilder<'c, 'd, Commands> {
    pub fn new(buffer: &'d mut [u8]) -> Self {
        Self {
            buffer,
            offset: COMMANDLIST_HEADER_LENGTH,
            _phantom_command: PhantomData
        }
    }
}

pub struct StaticCommandListCompletion<'a> {
    completion: &'a AtomicU32,
}

impl<'a> StaticCommandListCompletion<'a> {
    pub fn new(completion: &'a AtomicU32) -> Self {
        Self {
            completion
        }
    }
}

impl<'a> CommandListCompletion<'a> for StaticCommandListCompletion<'a> {
    unsafe fn raw_ptr(&self) -> *mut u32 {
        self.completion.as_ptr()
    }
}

impl<'c, 'd, Commands> CommandListBuilder<'c, 'd, Commands> for StaticCommandListBuilder<'c, 'd, Commands> {
    type Data = StaticCommandList<'d, Commands>;
    type Completion = StaticCommandListCompletion<'c>;

    fn push_command(&mut self, data: &[u8]) -> bool {
        let data_len = data.len();
        if data_len < self.buffer.len() - self.offset as usize {
            self.buffer[self.offset as usize..(self.offset + data_len) as usize].copy_from_slice(data);
            self.offset += data_len;
            true
        } else {
            false
        }
    }

    fn finish(self) -> Self::Data {
        self.buffer[0..4].copy_from_slice(&command_u32_bytes((self.offset - 8) as u32));
        StaticCommandList(self.buffer, PhantomData)
    }
}

pub fn command_u32_bytes(x: u32) -> [u8; 4] {
    [
        (x >>  0) as u8,
        (x >>  8) as u8,
        (x >> 16) as u8,
        (x >> 24) as u8
    ]
}

pub fn command_u16_bytes(x: u16) -> [u8; 2] {
    [
        (x >>  0) as u8,
        (x >>  8) as u8,
    ]
}
