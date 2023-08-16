use bytemuck::{Pod, cast_slice};

use super::types::AbstractPixelData;

pub struct BufferModule {
    pub memory: Box<[u8]>,
    pub length: u32,
}

pub const BUFFER_MAX_SIZE: u32 = 1024 * 128;

impl BufferModule {
    pub fn new() -> Self {
        Self {
            memory: vec![0u8; BUFFER_MAX_SIZE as usize].into_boxed_slice(),
            length: 0,
        }
    }

    pub fn bytes(&self) -> &[u8] {
        &self.memory[0..self.length as usize]
    }

    pub fn bytes_mut(&mut self) -> &mut [u8] {
        &mut self.memory[0..self.length as usize]
    }

    pub fn read<T: Pod + Default>(&self, offset: u32) -> T {
        let read_size = std::mem::size_of::<T>();
        if offset as usize + read_size < self.length as usize {
            cast_slice(&self.memory[offset as usize..offset as usize + read_size])[0]
        } else {
            T::default()
        }
    }
}
