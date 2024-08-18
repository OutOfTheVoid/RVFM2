use rvfm_platform::command_list::CommandListCompletion;

use super::command_list_internal::CompletionInternal;

#[derive(Clone)]
pub struct Fence(pub(crate) CompletionInternal);

impl Fence {
    pub fn new(initial_value: u32) -> Self {
        Self(CompletionInternal::new())
    }

    pub fn read(&self) -> u32 {
        self.0.read()
    }

    pub fn reset(&self, value: u32) {
        self.0.reset(value);
    }

    pub fn wait_nonzero(&self) {
        while self.read() == 0 {}
    }

    pub fn wait_greater_or_equal(&self, value: u32) {
        while self.read() < value {}
    }
}

pub struct FenceWait {
    pub fence: Fence,
    pub value: u32,
}

impl FenceWait {
    pub fn wait(&self) {
        self.fence.wait_greater_or_equal(self.value);
    }
}

