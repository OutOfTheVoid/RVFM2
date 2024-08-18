use core::sync::atomic::AtomicU32;

use alloc::sync::Arc;
use rvfm_platform::multihart::spinlock::SpinLock;

use crate::{instance::Instance, resource_tracker::ResourceTracker};

use super::resource_tracker::BufferHandle;

pub(crate) struct BufferState {
    pub size: usize,
    pub sid: usize,
}

pub(crate) struct BufferInternal {
    pub handle: BufferHandle,
    pub state: SpinLock<BufferState>,
    pub tracker: ResourceTracker,
}

impl Drop for BufferInternal {
    fn drop(&mut self) {
        self.tracker.free_buffer(self.handle);
    }
}

pub struct Buffer(pub(crate) Arc<BufferInternal>);


impl Buffer {
    pub(crate) fn new(handle: BufferHandle, tracker: ResourceTracker, size: usize, creation_sid: usize) -> Self {
        let state = BufferState {
            size,
            sid: creation_sid,
        };
        Self(Arc::new(BufferInternal {
            handle,
            state: SpinLock::new(state),
            tracker
        }))
    }
}
