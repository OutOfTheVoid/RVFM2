use alloc::sync::Arc;
use rvfm_platform::gpu::ShaderKind;

use crate::resource_tracker::{ResourceTracker, ShaderHandle};

pub(crate) struct ShaderInternal {
    pub handle: ShaderHandle,
    pub tracker: ResourceTracker,
    pub kind: ShaderKind,
    pub sid: usize,
}

pub struct Shader(pub(crate) Arc<ShaderInternal>);

impl Shader {
    pub fn kind(&self) -> ShaderKind {
        self.0.kind
    }
}

impl Drop for ShaderInternal {
    fn drop(&mut self) {
        self.tracker.free_shader(self.handle);
    }
}
