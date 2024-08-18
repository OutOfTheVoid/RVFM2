use core::pin::Pin;
use core::sync::atomic::{self, AtomicU32};

use alloc::sync::Arc;
use alloc::vec::Vec;
use alloc::vec;
use alloc::boxed::Box;
use rvfm_platform::command_list::{self, CommandListBuilder, command_u32_bytes};
use rvfm_platform::gpu::GpuCommands;

pub(crate) struct CommandListInternal {
    command_bytes: Pin<Box<[u8]>>,
}

impl command_list::CommandListData<'static, GpuCommands> for CommandListInternal {
    fn command_list_bytes(&mut self) -> &mut [u8] {
        &mut self.command_bytes[..]
    }
}

pub(crate) struct CommandListBuilderInternal {
    command_bytes: Vec<u8>,
}

#[derive(Clone)]
pub(crate) struct CompletionInternal {
    completion: Pin<Arc<AtomicU32>>
}

impl command_list::CommandListCompletion<'static> for CompletionInternal {
    unsafe fn raw_ptr(&self) -> *mut u32 {
        self.completion.as_ptr()
    }
}

impl CompletionInternal {
    pub fn new() -> Self {
        Self {
            completion: Arc::pin(AtomicU32::new(0))
        }
    }
}

impl command_list::CommandListBuilder<'static, 'static, GpuCommands> for CommandListBuilderInternal {
    type Data = CommandListInternal;
    type Completion = CompletionInternal;

    fn push_command(&mut self, command_bytes: &[u8]) -> bool {
        self.command_bytes.extend_from_slice(command_bytes);
        true
    }

    fn finish(self) -> Self::Data {
        let mut bytes = self.command_bytes.into_boxed_slice();
        CommandListInternal {
            command_bytes: Box::into_pin(bytes)
        }
    }
}

impl CommandListBuilderInternal {
    pub fn new() -> Self {
        Self {
            command_bytes: vec![0u8; 8],
        }
    }

    pub fn finish_and_reset(&mut self) -> CommandListInternal {
        let mut new_self = Self::new();
        core::mem::swap(self, &mut new_self);
        new_self.finish()
    }
}
