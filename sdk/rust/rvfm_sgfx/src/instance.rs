use alloc::sync::Arc;
use rvfm_platform::multihart::spinlock::*;

use crate::{buffer::Buffer, resource_tracker::ResourceTracker};

struct RawInstance {
    resource_tracker: ResourceTracker,

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
        let raw_instance = RawInstance {
            resource_tracker: ResourceTracker::new()
        };
        Self(Arc::new(raw_instance))
    }

    pub fn create_buffer(&self, size: usize) -> Result<Buffer, ResourceCreateError> {
        match self.0.resource_tracker.alloc_buffer() {
            Some(buffer_handle) => Ok(Buffer::new(buffer_handle, self, size)),
            None => Err(ResourceCreateError::NoneFree),
        }
    }
}