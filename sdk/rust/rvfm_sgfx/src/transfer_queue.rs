use core::sync::atomic::{self, AtomicU32, AtomicUsize};

use alloc::{boxed::Box, collections::vec_deque::VecDeque, sync::Arc, vec::Vec};
use rvfm_platform::{__println__, command_list::CommandListCompletion, gpu::{gpu_submit, GpuCommandBuilderExt, ShaderKind, TextureConfig, VideoResolution}, multihart::spinlock::SpinLock};

use crate::{buffer::{Buffer, BufferInternal, BufferState}, fence::Fence, resource_tracker::{BufferHandle, ResourceTracker, ShaderHandle, TextureHandle}, shader::{self, ShaderInternal}, texture::{Texture, TextureInternal, TextureState}, Shader};

use super::command_list_internal::*;

pub trait RetainedData: AsRef<[u8]> + 'static {}

impl<T: AsRef<[u8]> + 'static> RetainedData for T {}

pub(crate) struct TransferQueue {
    completion_internal: CompletionInternal,
    pending_command_list: CommandListBuilderInternal,
    retained_datas: VecDeque<(Option<Box<dyn RetainedData>>, usize)>,
    sequence_counter: Arc<AtomicUsize>,
}

impl TransferQueue {
    pub fn new(sequence_counter: Arc<AtomicUsize>) -> Self {
        Self {
            completion_internal: CompletionInternal::new(),
            pending_command_list: CommandListBuilderInternal::new(),
            sequence_counter,
            retained_datas: VecDeque::new(),
        }
    }

    pub fn submit(&mut self) -> TransferCompletion {
        let mut transfer_completion_internal = CompletionInternal::new();
        let mut command_list = self.pending_command_list.finish_and_reset();
        gpu_submit(&mut command_list, &mut transfer_completion_internal);
        TransferCompletion {
            command_list,
            completion: transfer_completion_internal,
        }
    }

    pub fn append_buffer_creation(&mut self, buffer_handle: BufferHandle, tracker: ResourceTracker, size: usize, write_fence: bool) -> (Buffer, u32) {
        self.pending_command_list.configure_buffer(buffer_handle.0, size as u32);
        
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        let buffer_state = BufferState {
            sid,
            size
        };

        (
            Buffer(Arc::new(BufferInternal {
                handle: buffer_handle,
                state: SpinLock::new(buffer_state),
                tracker
            })),
            sid as u32
        )
    }

    pub fn append_buffer_upload(&mut self, buffer: &Buffer, data: *const u8, write_fence: bool) -> u32 {
        self.pending_command_list.upload_buffer(buffer.0.handle.0, data);
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        if write_fence {
            self.pending_command_list.write_flag(
                unsafe { self.completion_internal.raw_ptr() as usize as u32 },
                sid as u32,
                false
            );
        }
        let mut buffer_state = buffer.0.state.lock();
        buffer_state.sid = sid;

        sid as u32
    }

    pub fn append_buffer_write(&mut self, buffer: &Buffer, data: *const u8, length: usize, offset: usize, write_fence: bool) -> u32 {
        self.pending_command_list.write_buffer(buffer.0.handle.0, data, length as u32, offset as u32);
        
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        if write_fence {
            self.pending_command_list.write_flag(
                unsafe { self.completion_internal.raw_ptr() as usize as u32 },
                sid as u32,
                false
            );
        }
        let mut buffer_state = buffer.0.state.lock();
        buffer_state.sid = sid;

        sid as u32
    }

    pub fn append_buffer_write_retained(&mut self, buffer: &Buffer, data: impl RetainedData, offset: usize, write_fence: bool) -> u32 {
        self.pending_command_list.write_buffer(buffer.0.handle.0, data.as_ref().as_ptr(), data.as_ref().len() as u32, offset as u32);
        
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        if write_fence {
            self.pending_command_list.write_flag(
                unsafe { self.completion_internal.raw_ptr() as usize as u32 },
                sid as u32,
                false
            );
        }
        let mut buffer_state = buffer.0.state.lock();
        buffer_state.sid = sid;
        
        let boxed_retained_data = Box::new(data);
        self.retained_datas.push_back((Some(boxed_retained_data), buffer_state.sid));

        sid as u32
    }

    pub fn append_texture_creation(&mut self, texture_handle: TextureHandle, tracker: ResourceTracker, config: &TextureConfig, write_fence: bool) -> (Texture, u32) {
        self.pending_command_list.configure_texture(texture_handle.0, config);
        
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        let texture_state = TextureState {
            sid,
            config: *config
        };

        (
            Texture(Arc::new(TextureInternal {
                handle: texture_handle,
                state: SpinLock::new(texture_state),
                tracker
            })),
            sid as u32
        )
    }

    pub fn resolve_retained_writes(&mut self) {
        let current_sid = self.completion_internal.read();
        for (retained, sid) in self.retained_datas.iter_mut() {
            if retained.is_some() && *sid <= current_sid as usize {
                *retained = None;
            }
        }
        while let Some((None, _)) = self.retained_datas.front() {
            self.retained_datas.pop_front();
        }
    }

    pub fn transfer_completion(&self) -> CompletionInternal {
        self.completion_internal.clone()
    }

    pub fn barrier(&mut self) -> u32 {
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel) as u32;
        self.pending_command_list.write_flag(unsafe { self.completion_internal.raw_ptr() as usize as u32 }, sid, false).unwrap();
        sid
    }

    pub fn append_set_video_mode(&mut self, resolution: VideoResolution) {
        self.pending_command_list.set_video_mode(resolution, true, true, true);
    }

    pub fn append_shader_write_retained(&mut self, shader_handle: ShaderHandle, kind: ShaderKind, tracker: ResourceTracker, data: impl RetainedData, write_fence: bool) -> (Shader, u32) {
        self.pending_command_list.upload_shader(shader_handle.0, kind, data.as_ref().as_ptr(), data.as_ref().len() as u32);
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        if write_fence {
            self.pending_command_list.write_flag(
                unsafe { self.completion_internal.raw_ptr() as usize as u32 },
                sid as u32,
                false
            );
        }
        let boxed_retained_data = Box::new(data);
        self.retained_datas.push_back((Some(boxed_retained_data), sid));

        (
            Shader(Arc::new(ShaderInternal {
                handle: shader_handle,
                tracker,
                kind,
                sid
            })),
            sid as u32
        )
    }

    pub fn append_shader_write(&mut self, shader_handle: ShaderHandle, kind: ShaderKind, tracker: ResourceTracker, data: *const u8, length: u32, write_fence: bool) -> (Shader, u32) {
        self.pending_command_list.upload_shader(shader_handle.0, kind, data, length);
        let sid = self.sequence_counter.fetch_add(1, atomic::Ordering::AcqRel);
        if write_fence {
            self.pending_command_list.write_flag(
                unsafe { self.completion_internal.raw_ptr() as usize as u32 },
                sid as u32,
                false
            );
        }
        (
            Shader(Arc::new(ShaderInternal {
                handle: shader_handle,
                tracker,
                kind,
                sid
            })),
            sid as u32
        )
    }
}

pub(crate) struct TransferCompletion {
    command_list: CommandListInternal,
    completion: CompletionInternal,
}

impl TransferCompletion {
    pub fn wait(&mut self) {
        self.completion.wait_nonzero();
    }
}

impl Drop for TransferCompletion {
    fn drop(&mut self) {
        self.wait();
    }
}
