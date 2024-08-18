use alloc::{sync::Arc, vec::Vec};
use rvfm_platform::{command_list::CommandListCompletion, gpu::{gpu_submit, GpuCommandBuilderExt, ShaderKind, VideoResolution}, multihart::spinlock::SpinLock};
use core::sync::atomic::{self, AtomicUsize};
pub use rvfm_platform::gpu::TextureConfig;

use crate::{buffer::Buffer, command_list_internal::{CommandListBuilderInternal, CompletionInternal}, constant_sampler::{ConstantSampler, ConstantSamplerInternal}, fence::{Fence, FenceWait}, resource_tracker::{BufferHandle, ResourceTracker, TextureHandle}, shader, texture::Texture, transfer_queue::{self, TransferQueue}, CommandBuffer, CommandBuilder, RetainedData, Shader};

struct RawInstance {
    resource_tracker: ResourceTracker,
    sequence_counter: Arc<AtomicUsize>,
    transfer_queue: SpinLock<TransferQueue>,
}

#[derive(Clone)]
pub struct Instance(Arc<RawInstance>);

unsafe impl Send for Instance {}
unsafe impl Sync for Instance {}

#[derive(Copy, Clone, Debug)]
pub enum ResourceCreateError {
    NoneFree,
}

impl Instance {
    pub fn new() -> Self {
        let sequence_counter = Arc::new(AtomicUsize::new(0));
        let raw_instance = RawInstance {
            resource_tracker: ResourceTracker::new(),
            transfer_queue: SpinLock::new(TransferQueue::new(sequence_counter.clone())),
            sequence_counter,
        };
        Self(Arc::new(raw_instance))
    }

    pub fn set_video_mode(&mut self, resolution: VideoResolution) -> FenceWait {
        let mut queue = self.0.transfer_queue.lock();
        queue.append_set_video_mode(resolution);
        let sid = queue.barrier();
        queue.submit().wait();
        FenceWait {
            fence: Fence(queue.transfer_completion()),
            value: sid
        }
    }

    pub fn create_constant_sampler(&self) -> Result<ConstantSampler, ResourceCreateError> {
        match self.0.resource_tracker.alloc_constant_sampler() {
            Some(sampler_handle) => Ok(ConstantSampler(
                Arc::new(ConstantSamplerInternal {
                    handle: sampler_handle,
                    tracker: self.0.resource_tracker.clone()
                })
            )),
            None => Err(ResourceCreateError::NoneFree)
        }
    }

    pub fn create_buffer(&self, size: usize) -> Result<Buffer, ResourceCreateError> {
        match self.0.resource_tracker.alloc_buffer() {
            Some(buffer_handle) => {
                let (buffer, _) = self.0.transfer_queue.lock().append_buffer_creation(buffer_handle, self.0.resource_tracker.clone(), size, false);
                Ok(buffer)
            },
            None => Err(ResourceCreateError::NoneFree),
        }
    }

    pub fn write_buffer_now(&self, buffer: &Buffer, offset: usize, data: &[u8]) {
        let (fence, sid) = {
            let mut transfer_queue = self.0.transfer_queue.lock();
            let sid = transfer_queue.append_buffer_write(buffer, data.as_ptr(), data.len(), offset, true);
            transfer_queue.submit();
            (Fence(transfer_queue.transfer_completion()), sid)
        };
        fence.wait_greater_or_equal(sid);
    }

    pub fn write_buffer(&self, buffer: &Buffer, offset: usize, data: impl RetainedData, fence: bool) -> Option<FenceWait> {
        let mut transfer_queue = self.0.transfer_queue.lock();
        let sid = transfer_queue.append_buffer_write_retained(buffer, data, offset, fence);
        if fence {
            Some(FenceWait { fence: Fence(transfer_queue.transfer_completion()), value: sid })
        } else {
            None
        }
    }

    pub(crate) fn free_buffer(&self, buffer: BufferHandle) {
        self.0.resource_tracker.free_buffer(buffer);
    }

    pub fn create_texture(&mut self, config: &TextureConfig) -> Result<Texture, ResourceCreateError> {
        match self.0.resource_tracker.alloc_texture() {
            Some(texture_handle) => {
                let (texture, _) = self.0.transfer_queue.lock().append_texture_creation(texture_handle, self.0.resource_tracker.clone(), config, false);
                Ok(texture)
            }
            None => Err(ResourceCreateError::NoneFree),
        }
    }

    pub(crate) fn free_texture(&self, texture: TextureHandle) {
        self.0.resource_tracker.free_texture(texture);
    }

    pub fn create_fence(&self) -> Fence {
        Fence(CompletionInternal::new())
    }

    pub fn create_command_builder(&self) -> CommandBuilder {
        CommandBuilder {
            sequence_counter: self.0.sequence_counter.clone(),
            builder_internal: CommandListBuilderInternal::new(),
            resources: Vec::new()
        }
    }

    pub fn flush_transfer_queue(&self, operation_fence: bool) -> Option<FenceWait> {
        let mut queue = self.0.transfer_queue.lock();
        let finished_sid = queue.barrier();
        let mut transfer_completion = queue.submit();
        transfer_completion.wait();
        if operation_fence {
            Some (
                FenceWait { fence: Fence(queue.transfer_completion().clone()), value: finished_sid }
            )
        } else {
            None
        }
    }

    pub fn create_shader(&self, kind: ShaderKind, shader_code: impl RetainedData, fence: bool) -> Result<(Shader, Option<FenceWait>), ResourceCreateError> {
        match self.0.resource_tracker.alloc_shader() {
            Some(shader_handle) => {
                let mut queue =  self.0.transfer_queue.lock();
                let (shader, sid) = queue.append_shader_write_retained(shader_handle, kind, self.0.resource_tracker.clone(), shader_code, true);
                let fence_wait = if fence {
                    Some(FenceWait {
                        fence: Fence(queue.transfer_completion()),
                        value: sid
                    })
                } else {
                    None
                };
                Ok((shader, fence_wait))
            },
            None => Err(ResourceCreateError::NoneFree)
        }
    }

    pub fn create_shader_now(&self, kind: ShaderKind, shader_code: &[u8]) -> Result<Shader, ResourceCreateError> {
        let (shader, fence, sid) = match self.0.resource_tracker.alloc_shader() {
            Some(shader_handle) => {
                let mut transfer_queue = self.0.transfer_queue.lock();
                let (shader, sid) = transfer_queue.append_shader_write(shader_handle, kind, self.0.resource_tracker.clone(), shader_code.as_ptr(), shader_code.len() as u32, true);
                transfer_queue.submit();
                (shader, Fence(transfer_queue.transfer_completion()), sid)
            },
            None => return Err(ResourceCreateError::NoneFree)
        };
        fence.wait_greater_or_equal(sid);
        Ok(shader)
    }

    pub fn submit_command_buffer(&self, command_buffer: &mut CommandBuffer) {
        let dependency_sid = command_buffer.transfer_dependency_sid_max();
        let transfer_sid = {
            let transfer_queue = self.0.transfer_queue.lock();
            transfer_queue.transfer_completion().read()
        };
        if dependency_sid >= transfer_sid as usize {
            self.flush_transfer_queue(false);
        }
        let mut submission_completion = command_buffer.submission_completion.take().map(|completion| {
            completion.wait_nonzero();
            completion
        }).unwrap_or_else(|| CompletionInternal::new());
        gpu_submit(&mut command_buffer.list_internal, &mut submission_completion);
        command_buffer.submission_completion = Some(submission_completion);
        self.0.transfer_queue.lock().resolve_retained_writes();
    }
}