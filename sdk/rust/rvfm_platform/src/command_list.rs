use core::{marker::PhantomData, sync::atomic::AtomicU32};

pub trait CommandListBuilder<Commands>: Sized {
    type Data: CommandListData<Commands>;
    type Completion: CommandListCompletion;

    fn push_command(&mut self, command_bytes: &[u8]) -> bool;
    fn finish(self) -> Self::Data;
}

pub trait CommandListData<Command>: Sized {
    fn command_list_bytes(&mut self) -> &mut [u8];
}

pub trait CommandListCompletion: Sized {
    unsafe fn raw_ptr(&self) -> *mut u32;

    fn reset(&self, value: u32) {
        unsafe { self.raw_ptr().write_volatile(value); }
    }

    fn wait_nonzero(&self) {
        while unsafe { self.raw_ptr().read_volatile() } == 0 {}
    }
}

pub struct StaticCommandList<'a, Commands>(pub(crate) &'a mut [u8], PhantomData<Commands>);

impl<'a, Commands> CommandListData<Commands> for StaticCommandList<'a, Commands> {
    fn command_list_bytes(&mut self) -> &mut [u8] {
        self.0
    }
}

pub struct StaticCommandListBuilder<'a, Commands> {
    buffer: &'a mut [u8],
    offset: usize,
    _phantom_command: PhantomData<Commands>,
}

impl<'a, Commands> StaticCommandListBuilder<'a, Commands> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            offset: 8,
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

impl<'a> CommandListCompletion for StaticCommandListCompletion<'a> {
    unsafe fn raw_ptr(&self) -> *mut u32 {
        self.completion.as_ptr()
    }
}

impl<'a, Commands> CommandListBuilder<Commands> for StaticCommandListBuilder<'a, Commands> {
    type Data = StaticCommandList<'a, Commands>;
    type Completion = StaticCommandListCompletion<'a>;

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

pub(crate) fn command_u32_bytes(x: u32) -> [u8; 4] {
    [
        (x >>  0) as u8,
        (x >>  8) as u8,
        (x >> 16) as u8,
        (x >> 24) as u8
    ]
}

pub(crate) fn command_u16_bytes(x: u16) -> [u8; 2] {
    [
        (x >>  0) as u8,
        (x >>  8) as u8,
    ]
}
