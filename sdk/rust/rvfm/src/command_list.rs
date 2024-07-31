use core::marker::PhantomData;

pub struct CommandList<'a, Commands>(pub(crate) &'a mut [u8], PhantomData<Commands>);

pub struct CommandListBuilder<'a, Commands> {
    buffer: &'a mut [u8],
    offset: usize,
    _phantom_command: PhantomData<Commands>,
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

impl<'a, Commands> CommandListBuilder<'a, Commands> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self {
            buffer,
            offset: 8,
            _phantom_command: PhantomData
        }
    }

    pub(crate) fn push_command(self, data: &[u8]) -> Result<Self, ()> {
        let CommandListBuilder {
            buffer, 
            offset,
            _phantom_command
        } = self;
        let data_len = data.len() as usize;
        if data_len < (buffer.len() - offset as usize) {
            buffer[offset as usize..(offset + data_len) as usize].copy_from_slice(data);
            Ok(
                CommandListBuilder {
                    buffer,
                    offset: offset + data_len,
                    _phantom_command,
                }
            )
        } else {
            Err(())
        }
    }

    pub fn finish(self) -> CommandList<'a, Commands> {
        self.buffer[0..4].copy_from_slice(&command_u32_bytes((self.offset - 8) as u32));
        CommandList(self.buffer, PhantomData)
    }
}

pub struct CommandListCompletion<'c>(pub(crate) &'c mut u32);

impl<'c> CommandListCompletion<'c> {
    fn wait(&mut self) {
        while !self.poll() {}
    }

    fn poll(&mut self) -> bool {
        unsafe { (self.0 as *mut u32).read_volatile() != 0 }
    }
}

impl Drop for CommandListCompletion<'_> {
    fn drop(&mut self) {
        self.wait()
    }
}
