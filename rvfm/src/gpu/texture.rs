use std::fmt::Debug;

use bytemuck::{cast_slice, Pod, cast_slice_mut};

use super::types::*;

pub struct TextureModule {
    pub memory: Box<[u32]>,
    pub config: Texture,
}

impl TextureModule {
    pub fn new() -> Self {
        Self {
            memory: vec![0u32; 512 * 512 * 4].into_boxed_slice().into(),
            config: Texture {
                width: 0,
                height: 0,
                pixel_layout: PixelDataLayout::D8x1,
                image_layout: ImageDataLayout::Contiguous,
            }
        }
    }

    pub fn data_slice_mut(&mut self) -> &mut [u8] {
        bytemuck::cast_slice_mut(&mut self.memory[0..self.config.pixel_layout.pixel_bytes() * self.config.width as usize * self.config.height as usize])
    }

    pub fn data_slice(&self) -> &[u8] {
        bytemuck::cast_slice(&self.memory[0..self.config.pixel_layout.pixel_bytes() * self.config.width as usize * self.config.height as usize])
    }

    pub fn clear(&mut self, data: AbstractPixelData) {
        println!("texture::clear() - pixel_layout: {:?}, data: {:?}", self.config.pixel_layout, data);
        match (data, self.config.pixel_layout) {
            (AbstractPixelData::U32(data), PixelDataLayout::D8x1) => self.clear_internal(data[0] as u8),
            (AbstractPixelData::U32(data), PixelDataLayout::D8x2) => self.clear_internal([data[0] as u8, data[1] as u8]),
            (AbstractPixelData::U32(data), PixelDataLayout::D8x4) => self.clear_internal(data.map(|d| d as u8)),
            (AbstractPixelData::U32(data), PixelDataLayout::D16x1) => self.clear_internal(data[0] as u16),
            (AbstractPixelData::U32(data), PixelDataLayout::D16x2) => self.clear_internal([data[0] as u16, data[1] as u16]),
            (AbstractPixelData::U32(data), PixelDataLayout::D16x4) => self.clear_internal(data.map(|d| d as u16)),
            (AbstractPixelData::U32(data), PixelDataLayout::D32x1) => self.clear_internal(data[0]),
            (AbstractPixelData::U32(data), PixelDataLayout::D32x2) => self.clear_internal([data[0], data[1]]),
            (AbstractPixelData::U32(data), PixelDataLayout::D32x4) => self.clear_internal(data),

            (AbstractPixelData::I32(data), PixelDataLayout::D8x1) => self.clear_internal(data[0] as i8),
            (AbstractPixelData::I32(data), PixelDataLayout::D8x2) => self.clear_internal([data[0] as i8, data[1] as i8]),
            (AbstractPixelData::I32(data), PixelDataLayout::D8x4) => self.clear_internal(data.map(|d| d as i8)),
            (AbstractPixelData::I32(data), PixelDataLayout::D16x1) => self.clear_internal(data[0] as i16),
            (AbstractPixelData::I32(data), PixelDataLayout::D16x2) => self.clear_internal([data[0] as i16, data[1] as i16]),
            (AbstractPixelData::I32(data), PixelDataLayout::D16x4) => self.clear_internal(data.map(|d| d as i16)),
            (AbstractPixelData::I32(data), PixelDataLayout::D32x1) => self.clear_internal(data[0]),
            (AbstractPixelData::I32(data), PixelDataLayout::D32x2) => self.clear_internal([data[0], data[1]]),
            (AbstractPixelData::I32(data), PixelDataLayout::D32x4) => self.clear_internal(data),

            (AbstractPixelData::UNorm32(data), PixelDataLayout::D8x1) => self.clear_internal(data[0] as u8),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D8x2) => self.clear_internal([data[0] as u8, data[1] as u8]),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D8x4) => self.clear_internal(data.map(|d| d as u8)),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D16x1) => self.clear_internal(data[0] as u16),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D16x2) => self.clear_internal([data[0] as u16, data[1] as u16]),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D16x4) => self.clear_internal(data.map(|d| d as u16)),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D32x1) => self.clear_internal(data[0]),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D32x2) => self.clear_internal([data[0], data[1]]),
            (AbstractPixelData::UNorm32(data), PixelDataLayout::D32x4) => self.clear_internal(data),

            (AbstractPixelData::INorm32(data), PixelDataLayout::D8x1) => self.clear_internal(data[0] as i8),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D8x2) => self.clear_internal([data[0] as i8, data[1] as i8]),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D8x4) => self.clear_internal(data.map(|d| d as i8)),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D16x1) => self.clear_internal(data[0] as i16),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D16x2) => self.clear_internal([data[0] as i16, data[1] as i16]),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D16x4) => self.clear_internal(data.map(|d| d as i16)),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D32x1) => self.clear_internal(data[0]),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D32x2) => self.clear_internal([data[0], data[1]]),
            (AbstractPixelData::INorm32(data), PixelDataLayout::D32x4) => self.clear_internal(data),

            (AbstractPixelData::F32(data), PixelDataLayout::D32x1) => self.clear_internal(data[0]),
            (AbstractPixelData::F32(data), PixelDataLayout::D32x2) => self.clear_internal([data[0], data[1]]),
            (AbstractPixelData::F32(data), PixelDataLayout::D32x4) => self.clear_internal(data),

            _ => {}
        }
    }

    fn clear_internal<T: Copy + Pod + Debug>(&mut self, data: T) {
        let image_size = self.config.width as usize * self.config.height as usize;
        let image_data = &mut cast_slice_mut::<_, T>(&mut self.memory[..])[0..image_size];
        image_data.fill(data);
    }

    pub fn fetch<T: Pod + Copy>(&self, x: u32, y: u32) -> T {
        let index = self.config.image_layout.index(x, y, self.config.width as u32);
        cast_slice(&self.memory[..])[index as usize]
    }

    pub fn store<T: Pod + Copy>(&mut self, x: u32, y: u32, value: T) {
        let index = self.config.image_layout.index(x, y, self.config.width as u32);
        cast_slice_mut(&mut self.memory[..])[index as usize] = value;
    }

}
