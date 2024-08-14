use alloc::sync::Arc;
use core::sync::atomic::AtomicUsize;

use crate::{buffer::Buffer, resource_tracker::{BufferHandle, ResourceTracker}};

struct RawInstance {
    resource_tracker: ResourceTracker,
    sequence_counter: AtomicUsize,
}

#[derive(Clone)]
pub struct Instance(Arc<RawInstance>);

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

pub enum ResourceCreateError {
    NoneFree,
}

impl Instance {
    pub fn new() -> Self {
        let sequence_counter = AtomicUsize::new(0);
        let raw_instance = RawInstance {
            resource_tracker: ResourceTracker::new(),
            sequence_counter,
        };
        Self(Arc::new(raw_instance))
    }

    pub fn create_buffer(&self, size: usize) -> Result<Buffer, ResourceCreateError> {
        match self.0.resource_tracker.alloc_buffer() {
            Some(buffer_handle) => {
                Ok(Buffer::new(buffer_handle, self, size))
            },
            None => Err(ResourceCreateError::NoneFree),
        }
    }

    pub(crate) fn free_buffer(&self, buffer: BufferHandle) {
        self.0.resource_tracker.free_buffer(buffer);
    }
}