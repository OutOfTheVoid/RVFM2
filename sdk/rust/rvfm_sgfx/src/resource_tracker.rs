use rvfm_platform::multihart::spinlock::SpinLock;
use alloc::sync::Arc;

pub(crate) struct TrackedResources {
    pub texture_allocs: [u32; 2],
    pub buffer_allocs: [u32; 8],
    pub shader_allocs: [u32; 4],
    pub pipeline_state_allocs: [u32; 2],
}

pub(crate) struct TextureHandle(u8);
pub(crate) struct BufferHandle(u8);
pub(crate) struct ShaderHandle(u8);
pub(crate) struct PipelineStateHandle(u8);

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
        Self::free(&mut self.buffer_allocs[..], shader.0)
    }

    pub fn alloc_pipeline_state(&mut self) -> Option<PipelineStateHandle> {
        Self::alloc(&mut self.pipeline_state_allocs[..])
            .map(|x| PipelineStateHandle(x))
    }

    pub fn free_pipeline_state(&mut self, state: PipelineStateHandle) {
        Self::free(&mut self.pipeline_state_allocs[..], state.0)
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

pub struct ResourceTracker {
    resources: Arc<SpinLock<TrackedResources>>,
}

impl ResourceTracker {
    pub fn new() -> Self {
        Self {
            resources: Arc::new(SpinLock::new (
                TrackedResources {
                    texture_allocs: [0; 2],
                    buffer_allocs: [0; 8],
                    shader_allocs: [0; 4],
                    pipeline_state_allocs: [0; 2],
                }
            ))
        }
    }

    pub fn alloc_buffer(&self) -> Option<BufferHandle> {
        let mut gaurd = self.resources.lock();
        gaurd.alloc_buffer()
    }
}
