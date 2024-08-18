use core::sync::atomic::AtomicU32;

use alloc::sync::Arc;
use rvfm_platform::multihart::spinlock::SpinLock;
pub use rvfm_platform::gpu::{TextureConfig, PixelDataLayout, ImageDataLayout};

use crate::{instance::Instance, resource_tracker::{ResourceTracker, TextureHandle}};

pub(crate) struct TextureState {
    pub config: TextureConfig,
    pub sid: usize,
}

pub(crate) struct TextureInternal {
    pub handle: TextureHandle,
    pub state: SpinLock<TextureState>,
    pub tracker: ResourceTracker,
}

impl Drop for TextureInternal {
    fn drop(&mut self) {
        self.tracker.free_texture(self.handle);
    }
}

#[derive(Clone)]
pub struct Texture(pub(crate) Arc<TextureInternal>);


impl Texture {
    pub(crate) fn new(handle: TextureHandle, tracker: ResourceTracker, config: TextureConfig, creation_sid: usize) -> Self {
        let state = TextureState {
            config,
            sid: creation_sid,
        };
        Self(Arc::new(TextureInternal {
            handle,
            state: SpinLock::new(state),
            tracker
        }))
    }
}
