#![allow(unused)]

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VideoResolution {
    V512x384,
    V256x192,
}

impl VideoResolution {
    pub fn as_w_h(&self) -> (u32, u32) {
        match self {
            Self::V256x192 => (256, 192),
            Self::V512x384 => (512, 384),
        }
    }

    pub fn pixel_count(&self) -> usize {
        let (w, h) = self.as_w_h();
        w as usize * h as usize
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct VideoMode {
    pub resolution:   VideoResolution,
    pub backgrounds:  bool,
    pub sprites:      bool,
    pub triangles:    bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageDataLayout {
    Contiguous,
    Block4x4,
    Block8x8,
}

impl ImageDataLayout {
    pub fn from_u8(val: u8) -> Option<Self> {
        Some(match val {
            0 => ImageDataLayout::Contiguous,
            1 => ImageDataLayout::Block8x8,
            2 => ImageDataLayout::Block4x4,
            _ => None?
        })
    }

    pub fn index(&self, x: u32, y: u32, w: u32) -> u32 {
        match self {
            ImageDataLayout::Contiguous => x + (y * w) as u32,
            ImageDataLayout::Block4x4 => {
                let block = (x >> 2) + (y >> 2) * (w as u32 >> 2);
                let b_x = x & 3;
                let b_y = y & 3;
                block * 16 + b_x + (b_y << 2)
            },
            ImageDataLayout::Block8x8 => {
                let block = (x >> 3) + (y >> 3) * (w as u32 >> 3);
                let b_x = x & 7;
                let b_y = y & 7;
                block * 64 + b_x + (b_y << 3)
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelDataLayout {
    D8x1,
    D8x2,
    D8x4,
    D16x1,
    D16x2,
    D16x4,
    D32x1,
    D32x2,
    D32x4,
}

impl PixelDataLayout {
    pub fn from_u8(val: u8) -> Option<Self> {
        Some(match val {
            0 => PixelDataLayout::D8x1,
            1 => PixelDataLayout::D8x2,
            2 => PixelDataLayout::D8x4,
            3 => PixelDataLayout::D16x1,
            4 => PixelDataLayout::D16x2,
            5 => PixelDataLayout::D16x4,
            6 => PixelDataLayout::D32x1,
            7 => PixelDataLayout::D32x2,
            8 => PixelDataLayout::D32x4,
            _ => None?
        })
    }

    pub fn pixel_bytes(&self) -> usize {
        match self {
            Self::D8x1 =>  1,
            Self::D8x2 =>  2,
            Self::D8x4 =>  4,
            Self::D16x1 => 2,
            Self::D16x2 => 4,
            Self::D16x4 => 8,
            Self::D32x1 => 4,
            Self::D32x2 => 8,
            Self::D32x4 => 16,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelDataType {
    RUNorm8,
    RgUNorm8,
    RgbUNorm8,
    RgbaUNorm8,

    RF32,
    RgF32,
    RgbF32,
    RgbaF32,
}

impl PixelDataType {
    pub fn from_u8(val: u8) -> Option<Self> {
        Some(match val {
            0 => Self::RUNorm8,
            1 => Self::RgUNorm8,
            2 => Self::RgbUNorm8,
            3 => Self::RgbaUNorm8,
            4 => Self::RF32,
            5 => Self::RgF32,
            6 => Self::RgbF32,
            7 => Self::RgbaF32,
            _ => None?,
        })
    }
    
    pub fn component_count(&self) -> u8 {
        match self {
            Self::RUNorm8 | Self::RF32 => 1,
            Self::RgUNorm8 | Self::RgF32 => 2,
            Self::RgbUNorm8 | Self::RgbF32 => 3,
            Self::RgbaUNorm8 | Self::RgbaF32 => 4,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ColorBlendOp {
    Zero,  // 0
    Src,   // 1
    Dst,   // dst
    Add,   // dst + src
    Avg,   // dst + src
    Sub,   // dst - src
    RSub,  // src - dst
    Blend, // src * src.a + dst * (1 - src.a)
    RBlend,// src * (1 - src.a) + dst * src.a
}

impl ColorBlendOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        Some(match val {
            0 => Self::Zero,
            1 => Self::Src,
            2 => Self::Dst,
            3 => Self::Add,
            4 => Self::Avg,
            5 => Self::Sub,
            6 => Self::RSub,
            7 => Self::Blend,
            8 => Self::RBlend,
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AlphaBlendOp {
    Zero, // 0
    One,  // 1
    Dst,  // dst
    Src,  // src
    Avg,  // (dst + src) / 2
    Add,  // dst + src
    Sub,  // dst - src
    RSub, // src - dst
    Blend // dst + (1 - dst) * src
}

impl AlphaBlendOp {
    pub fn from_u8(val: u8) -> Option<Self> {
        Some(match val {
            0 => Self::Zero,
            1 => Self::One,
            2 => Self::Dst,
            3 => Self::Src,
            4 => Self::Avg,
            5 => Self::Add,
            6 => Self::Sub,
            7 => Self::RSub,
            8 => Self::Blend,
            _ => None?
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Texture {
    pub width  : u16,
    pub height : u16,
    pub pixel_layout : PixelDataLayout,
    pub image_layout: ImageDataLayout,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Rect {
    pub position : I32x2,
    pub size     : I32x2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct I32x2 {
    pub x: i32,
    pub y: i32
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Background {
    pub enabled : bool,
    pub sampler : Sampler,
    pub shape   : Rect,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum Sampler {
    Constant(u8),
    Texture(u8),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ConstantSampler {
    pub constant_data : [u32; 4],
    pub data_type     : PixelDataType,
}

fn expand_unorm8_to_32(val: u8) -> u32 {
    (val as u32) << 0 |
    (val as u32) << 8 |
    (val as u32) << 16 |
    (val as u32) << 24
}

impl ConstantSampler {
    pub fn new() -> Self {
        Self {
            constant_data: [0; 4],
            data_type: PixelDataType::RgbaUNorm8,
        }
    }

    pub fn get_abstract(&self) -> AbstractPixelData {
        let float_data = bytemuck::cast_slice::<_, f32>(&self.constant_data[..]);
        match self.data_type {
            PixelDataType::RUNorm8 => AbstractPixelData::UNorm32([
                expand_unorm8_to_32((self.constant_data[0] >> 0) as u8),
                0,
                0,
                0,
            ]),
            PixelDataType::RgUNorm8 => AbstractPixelData::UNorm32([
                expand_unorm8_to_32((self.constant_data[0] >> 0) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 8) as u8),
                0,
                0,
            ]),

            PixelDataType::RgbUNorm8 => AbstractPixelData::UNorm32([
                expand_unorm8_to_32((self.constant_data[0] >> 0) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 8) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 16) as u8),
                0,
            ]),

            PixelDataType::RgbaUNorm8 => AbstractPixelData::UNorm32([
                expand_unorm8_to_32((self.constant_data[0] >> 0) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 8) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 16) as u8),
                expand_unorm8_to_32((self.constant_data[0] >> 24) as u8),
            ]),

            PixelDataType::RF32 => AbstractPixelData::F32([
                float_data[0],
                0.0,
                0.0,
                0.0
            ]),
            PixelDataType::RgF32 => AbstractPixelData::F32([
                float_data[0],
                float_data[1],
                0.0,
                0.0
            ]),
            PixelDataType::RgbF32 => AbstractPixelData::F32([
                float_data[0],
                float_data[1],
                float_data[2],
                0.0
            ]),
            PixelDataType::RgbaF32 => AbstractPixelData::F32([
                float_data[0],
                float_data[1],
                float_data[2],
                float_data[3]
            ]),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OverflowMode {
    Repeate,
    Mirror,
    Clamp,
    Zero,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureSampler {
    pub texture       : u8,
    pub data_type     : PixelDataType,
    pub sample_region : Rect,
    pub overflow_mode : OverflowMode,
}

#[derive(Copy, Clone, Debug)]
pub struct Sprite {
    pub layer: i8,
    pub sampler: Sampler,
    pub transform: [[f32; 3]; 2],
}

#[derive(Copy, Clone, Debug)]
pub enum AbstractPixelData {
    F32([f32; 4]),
    U32([u32; 4]),
    I32([i32; 4]),
    INorm32([i32; 4]),
    UNorm32([u32; 4]),
}

impl AbstractPixelData {
    fn as_f32(&self) -> [f32; 4] {
        match self {
            Self::F32(data) => data.clone(),
            Self::I32(data) => data.map(|d| d as f32),
            Self::U32(data) => data.map(|d| d as f32),
            Self::INorm32(data) => data.map(|d| d as f32 / 2147483648.0),
            Self::UNorm32(data) => data.map(|d| d as f32 / 4294967295.0),
        }
    }

    fn as_u32(&self) -> [u32; 4] {
        match self {
            Self::U32(data) => data.clone(),
            Self::F32(data) => data.map(|d| d as u32),
            Self::I32(data) => data.map(|d| d as u32),
            Self::INorm32(data) => data.map(|d| if d == 0x7FFF_FFFF { 1 } else { 0 }),
            Self::UNorm32(data) => data.map(|d| if d == 0xFFFF_FFFF { 1 } else { 0 }),
        }
    }

    fn as_i32(&self) -> [i32; 4] {
        match self {
            Self::I32(data) => data.clone(),
            Self::F32(data) => data.map(|d| d as i32),
            Self::U32(data) => data.map(|d| d as i32),
            Self::INorm32(data) => data.map(|d| if d == 0x7FFF_FFFF { 1 } else { 0 }),
            Self::UNorm32(data) => data.map(|d| if d == 0xFFFF_FFFF { 1 } else { 0 }),
        }
    }

    fn as_inorm32(&self) -> [i32; 4] {
        match self {
            Self::INorm32(data) => data.clone(),
            Self::UNorm32(data) => data.map(|d| (d >> 1) as i32),
            Self::I32(data) => data.map(|d| if (d & 1) != 0 { 0x7FFF_FFFF } else { 0 }),
            Self::U32(data) => data.map(|d| if (d & 1) != 0 { 0x7FFF_FFFF } else { 0 }),
            Self::F32(data) => data.map(|d| (d * 2147483648.0) as i32),
        }
    }

    fn as_unorm32(&self) -> [u32; 4] {
        match self {
            Self::UNorm32(data) => data.clone(),
            Self::INorm32(data) => data.map(|d| (d as u32) << 1),
            Self::I32(data) => data.map(|d| if (d & 1) != 0 { 0xFFFF_FFFF } else { 0 }),
            Self::U32(data) => data.map(|d| if (d & 1) != 0 { 0xFFFF_FFFF } else { 0 }),
            Self::F32(data) => data.map(|d| (d * 4294967295.0) as u32),
        }
    }
}

pub struct Blit {
    pub w: u32,
    pub h: u32,
    pub src_x: u32,
    pub src_y: u32,
    pub dst_x: u32,
    pub dst_y: u32,
}
