use core::sync::atomic::AtomicUsize;
use alloc::{boxed::Box, sync::Arc, vec::Vec};
use rvfm_platform::{command_list::{CommandListBuilder, CommandListCompletion}, gpu::GpuCommandBuilderExt};

use crate::{command_list_internal::{CommandListBuilderInternal, CommandListInternal, CompletionInternal}, constant_sampler::ConstantSampler, fence::Fence, texture, Buffer, Shader, Texture};

pub enum CommandResource {
    Fence(Fence),
    Buffer(Buffer),
    Texture(Texture),
    Shader(Shader),
    ConstantSampler(ConstantSampler),
}

impl CommandResource {
    fn transfer_sid(&self) -> Option<usize> {
        match self {
            Self::Buffer(buffer) => Some(buffer.0.state.lock().sid),
            Self::Texture(texture) => Some(texture.0.state.lock().sid),
            Self::Shader(shader) => Some(shader.0.sid),

            Self::Fence(_) => None,
            Self::ConstantSampler(_) => None,
        }
    }
}

impl PartialEq for CommandResource {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Fence(l0),   Self::Fence(r0))   => unsafe { (l0.0.raw_ptr() == r0.0.raw_ptr()) },
            (Self::Buffer(l0),  Self::Buffer(r0))  => l0.0.handle.0 == r0.0.handle.0,
            (Self::Texture(l0), Self::Texture(r0)) => l0.0.handle.0 == r0.0.handle.0,
            (Self::Shader(l0),  Self::Shader(r0))  => l0.0.handle.0 == r0.0.handle.0,
            (Self::ConstantSampler(l0), Self::ConstantSampler(r0)) => l0.0.handle.0 == r0.0.handle.0,
            _ => false,
        }
    }
}

pub struct CommandBuilder {
    pub(crate) sequence_counter: Arc<AtomicUsize>,
    pub(crate) builder_internal: CommandListBuilderInternal,
    pub(crate) resources: Vec<CommandResource>,
}

impl CommandBuilder {
    fn resource_dependency(&mut self, resource: CommandResource) {
        if !self.resources.contains(&resource) {
            self.resources.push(resource);
        }
    }

    fn write_fence(&mut self, fence: &Fence, value: u32) {
        self.builder_internal.write_flag(unsafe { fence.0.raw_ptr() as usize as u32 }, value, false).unwrap();
        self.resource_dependency(CommandResource::Fence(fence.clone()));
    }

    pub fn set_constant_sampler_f32(&mut self, constant_sampler: &ConstantSampler, color: [f32; 4]) {
        self.builder_internal.set_constant_sampler_f32(constant_sampler.0.handle.0, color).unwrap();
        self.resource_dependency(CommandResource::ConstantSampler(constant_sampler.clone()));
    }

    pub fn set_constant_sampler_unorm8(&mut self, constant_sampler: &ConstantSampler, color: [u8; 4]) {
        self.builder_internal.set_constant_sampler_unorm8(constant_sampler.0.handle.0, color).unwrap();
        self.resource_dependency(CommandResource::ConstantSampler(constant_sampler.clone()));
    }

    pub fn clear_texture(&mut self, texture: &Texture, constant_sampler: &ConstantSampler) {
        self.builder_internal.clear_texture(texture.0.handle.0, constant_sampler.0.handle.0).unwrap();
        self.resource_dependency(CommandResource::ConstantSampler(constant_sampler.clone()));
        self.resource_dependency(CommandResource::Texture(texture.clone()));
    }

    pub fn present_texture(&mut self, texture: &Texture, presentation_fence: &mut Fence) {
        self.builder_internal.present_texture(texture.0.handle.0, &mut presentation_fence.0, false);
        self.resource_dependency(CommandResource::Texture(texture.clone()));
        self.resource_dependency(CommandResource::Fence(presentation_fence.clone()));
    }

    pub fn build(self) -> CommandBuffer {
        CommandBuffer {
            list_internal: self.builder_internal.finish(),
            dependencies: self.resources.into_boxed_slice(),
            submission_completion: None,
        }
    }
}

pub struct CommandBuffer {
    pub(crate) list_internal: CommandListInternal,
    pub(crate) dependencies: Box<[CommandResource]>,
    pub(crate) submission_completion: Option<CompletionInternal>,
}

impl CommandBuffer {
    pub(crate) fn transfer_dependency_sid_max(&self) -> usize {
        let mut max_sid = 0;
        for dependency in self.dependencies.iter() {
            if let Some(sid) = dependency.transfer_sid() {
                max_sid = max_sid.max(sid);
            }
        }
        max_sid
    }
}

