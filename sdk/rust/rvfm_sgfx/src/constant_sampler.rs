use alloc::sync::Arc;

use crate::resource_tracker::{ConstantSamplerHandle, ResourceTracker};

pub(crate) struct ConstantSamplerInternal {
    pub handle: ConstantSamplerHandle,
    pub tracker: ResourceTracker
}

#[derive(Clone)]
pub struct ConstantSampler(pub(crate) Arc<ConstantSamplerInternal>);

impl Drop for ConstantSamplerInternal {
    fn drop(&mut self) {
        self.tracker.free_constant_sampler(self.handle);
    }
}

