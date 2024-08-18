use rvfm_platform::multihart::spinlock::SpinLock;
use alloc::sync::Arc;
use core::sync::atomic::{self, AtomicU32};
use alloc::vec::Vec;

pub(crate) struct TrackedResources {
    pub constant_sampler_allocs: [u32; 2],
    pub texture_allocs: [u32; 2],
    pub buffer_allocs: [u32; 8],
    pub shader_allocs: [u32; 4],
    pub pipeline_state_allocs: [u32; 2],
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct ConstantSamplerHandle(pub u8);
#[derive(Clone, Copy, Debug)]
pub(crate) struct TextureHandle(pub u8);
#[derive(Clone, Copy, Debug)]
pub(crate) struct BufferHandle(pub u8);
#[derive(Clone, Copy, Debug)]
pub(crate) struct ShaderHandle(pub u8);
#[derive(Clone, Copy, Debug)]
pub(crate) struct PipelineStateHandle(pub u8);

impl TrackedResources {
    pub fn alloc_texture(&mut self) -> Option<TextureHandle> {
        Self::alloc(&mut self.texture_allocs[..])
            .map(|x| TextureHandle(x))
    }

    pub fn free_texture(&mut self, texture: TextureHandle) {
        Self::free(&mut self.texture_allocs[..], texture.0)
    }

    pub fn alloc_buffer(&mut self) -> Option<BufferHandle> {
        Self::alloc(&mut self.buffer_allocs[..])
            .map(|x| BufferHandle(x))
    }

    pub fn free_buffer(&mut self, buffer: BufferHandle) {
        Self::free(&mut self.buffer_allocs[..], buffer.0)
    }

    pub fn alloc_shader(&mut self) -> Option<ShaderHandle> {
        Self::alloc(&mut self.shader_allocs[..])
            .map(|x| ShaderHandle(x))
    }

    pub fn free_shader(&mut self, shader: ShaderHandle) {
        Self::free(&mut self.shader_allocs[..], shader.0)
    }

    pub fn alloc_pipeline_state(&mut self) -> Option<PipelineStateHandle> {
        Self::alloc(&mut self.pipeline_state_allocs[..])
            .map(|x| PipelineStateHandle(x))
    }

    pub fn free_pipeline_state(&mut self, state: PipelineStateHandle) {
        Self::free(&mut self.pipeline_state_allocs[..], state.0)
    }

    pub fn alloc_constant_sampler(&mut self) -> Option<ConstantSamplerHandle> {
        Self::alloc(&mut self.constant_sampler_allocs[..])
            .map(|x| ConstantSamplerHandle(x))
    }

    pub fn free_constant_sampler(&mut self, sampler: ConstantSamplerHandle) {
        Self::free(&mut self.constant_sampler_allocs[..], sampler.0)
    }

    fn alloc(alloc_flags: &mut [u32]) -> Option<u8> {
        for w in 0..alloc_flags.len() {
            if alloc_flags[w] != 0xFFFF_FFFF {
                match alloc_flags[w].leading_ones() {
                    32 => continue,
                    x => {
                        alloc_flags[w] |= (1 << x);
                        return Some(x as u8 + (w * 32) as u8);
                    }
                }
            }
        }
        None
    }

    fn free(alloc_flags: &mut [u32], x: u8) {
        alloc_flags[x as usize >> 5] &= !(1 << (x & 31));
    }
}

#[derive(Clone)]
pub struct ResourceTracker {
    resources: Arc<SpinLock<TrackedResources>>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(SpinLock::new (
                TrackedResources {
                    constant_sampler_allocs: [0; 2],
                    texture_allocs: [0; 2],
                    buffer_allocs: [0; 8],
                    shader_allocs: [0; 4],
                    pipeline_state_allocs: [0; 2],
                }
            )),
        }
    }

    pub fn alloc_buffer(&self) -> Option<BufferHandle> {
        let mut resources = self.resources.lock();
        resources.alloc_buffer()
    }

    pub fn free_buffer(&self, handle: BufferHandle) {
        let mut resources = self.resources.lock();
        resources.free_buffer(handle);
    }

    pub fn alloc_texture(&self) -> Option<TextureHandle> {
        let mut resources = self.resources.lock();
        resources.alloc_texture()
    }

    pub fn free_texture(&self, handle: TextureHandle) {
        let mut resources = self.resources.lock();
        resources.free_texture(handle);
    }

    pub fn alloc_shader(&self) -> Option<ShaderHandle> {
        let mut resources = self.resources.lock();
        resources.alloc_shader()
    }

    pub fn free_shader(&self, handle: ShaderHandle) {
        let mut resources = self.resources.lock();
        resources.free_shader(handle);
    }

    pub fn alloc_constant_sampler(&self) -> Option<ConstantSamplerHandle> {
        let mut resources = self.resources.lock();
        resources.alloc_constant_sampler()
    }

    pub fn free_constant_sampler(&self, handle: ConstantSamplerHandle) {
        let mut resources = self.resources.lock();
        resources.free_constant_sampler(handle);
    }
}
