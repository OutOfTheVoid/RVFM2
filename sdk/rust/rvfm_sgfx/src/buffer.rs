use core::sync::atomic::AtomicU32;

use alloc::sync::Arc;
use rvfm_platform::multihart::spinlock::SpinLock;

use crate::instance::Instance;

use super::resource_tracker::BufferHandle;

struct BufferState {
    size: usize,
    dirty: bool,
}

struct BufferInternal {
    handle: BufferHandle,
    instance: Instance,
    state: SpinLock<BufferState>,

}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        self.instance.free_buffer(self.handle);
    }
}

pub struct Buffer(Arc<BufferInternal>);


impl Buffer {
    pub(crate) fn new(handle: BufferHandle, instance: &Instance, size: usize) -> Self {
        let state = BufferState {
            size,
            dirty: true
        };
        Self(Arc::new(BufferInternal {
            handle,
            instance: instance.clone(),
            state: SpinLock::new(state)
        }))
    }
}
