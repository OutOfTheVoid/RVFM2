use crate::command_list::*;
use super::pipeline_state::GraphicsPipelineState;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum VideoResolution {
    R512x384 = 1,
    R256x192 = 0,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelDataLayout {
    D8x1 = 0,
    D8x2 = 1,
    D8x4 = 2,
    D16x1 = 3,
    D16x2 = 4,
    D16x4 = 5,
    D32x1 = 6,
    D32x2 = 7,
    D32x4 = 8,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ImageDataLayout {
    Contiguous = 0,
    Block8x8 = 1,
    Block4x4 = 2,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TextureConfig {
    pub pixel_layout: PixelDataLayout,
    pub image_layout: ImageDataLayout,
    pub width: u16,
    pub height: u16,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum PixelDataType {
    RUNorm8 = 0,
    RgUNorm8 = 1,
    RgbUNorm8 = 2,
    RgbaUNorm8 = 3,
    RF32 = 4,
    RgF32 = 5,
    RgbF32 = 6,
    RgbaF32 = 7,
}

#[repr(u8)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum ShaderKind {
    Vertex = 0,
    Fragment = 1,
    Compute = 2,
}

pub struct GpuCommands;

pub struct ClippingRect {
    pub x_low: u16,
    pub x_high: u16,
    pub y_low: u16,
    pub y_high: u16,
}

pub trait GpuCommandBuilderExt: Sized {
    fn clear_texture(&mut self, texture: u8, constant_sampler: u8) -> Result<(), GpuCommandBuilderError>;
    fn present_texture<Completion: CommandListCompletion>(&mut self, texture: u8, completion: &mut Completion, interrupt: bool) -> Result<(), GpuCommandBuilderError>;
    fn set_constant_sampler_f32(&mut self, constant_sampler: u8, color: [f32; 4]) -> Result<(), GpuCommandBuilderError>;
    fn set_constant_sampler_unorm8(&mut self, constant_sampler: u8, color: [u8; 4]) -> Result<(), GpuCommandBuilderError>;
    fn set_video_mode(&mut self, resolution: VideoResolution, backgrounds: bool, sprites: bool, triangles: bool) -> Result<(), GpuCommandBuilderError>;
    fn write_flag(&mut self, flag_address: u32, value: u32, interrupt: bool) -> Result<(), GpuCommandBuilderError>;
    fn configure_texture(&mut self, texture: u8, config: &TextureConfig) -> Result<(), GpuCommandBuilderError>;
    fn upload_texture(&mut self, texture: u8, texture_data_layout: ImageDataLayout, texture_data: *const u8) -> Result<(), GpuCommandBuilderError>;
    fn configure_buffer(&mut self, buffer: u8, length: u32) -> Result<(), GpuCommandBuilderError>;
    fn upload_buffer(&mut self, buffer: u8, data: *const u8) -> Result<(), GpuCommandBuilderError>;
    fn direct_blit(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16) -> Result<(), GpuCommandBuilderError>;
    fn cutout_blit(&mut self, src_x: u8, src_y: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, src_alpha_data_type: PixelDataType) -> Result<(), GpuCommandBuilderError>;
    fn upload_shader(&mut self, index: u8, kind: ShaderKind, shader_code: &'static [u8]) -> Result<(), GpuCommandBuilderError>;
    fn upload_graphics_pipeline_state(&mut self, index: u8, /*flags: u8, */state: &'static GraphicsPipelineState) -> Result<(), GpuCommandBuilderError>;
    fn draw_graphics_pipeline(&mut self, index: u8, vertex_shader: u8, fragment_shader: u8, vertex_count: u32, clipping_rect: ClippingRect) -> Result<(), GpuCommandBuilderError>;
}

#[derive(Debug, Copy, Clone)]
pub enum GpuCommandBuilderError {
    OutOfSpace,
}

impl<D: CommandListData<GpuCommands>, Builder: CommandListBuilder<GpuCommands, Data = D>> GpuCommandBuilderExt for Builder {
    fn clear_texture(&mut self, texture: u8, constant_sampler: u8) -> Result<(), GpuCommandBuilderError> {
        let data = &[
            0x00,
            0x00,
            texture,
            constant_sampler
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn present_texture<Completion: CommandListCompletion>(&mut self, texture: u8, completion: &mut Completion, interrupt: bool) -> Result<(), GpuCommandBuilderError> {
        let completion_flag_bytes = command_u32_bytes(unsafe { completion.raw_ptr() } as usize as u32);
        let data = &[
            0x01,
            0x00,
            texture,
            if interrupt { 0x01 } else { 0x00 },
            completion_flag_bytes[0],
            completion_flag_bytes[1],
            completion_flag_bytes[2],
            completion_flag_bytes[3],
        ];
        if !self.push_command(data) {
            Err(GpuCommandBuilderError::OutOfSpace)
        } else {
            Ok(())
        }
    }

    fn set_constant_sampler_f32(&mut self, constant_sampler: u8, color: [f32; 4]) -> Result<(), GpuCommandBuilderError> {
        let color_data = bytemuck::cast_slice::<_, u8>(&color[..]);
        let data = &[
            0x02,
            0x00,
            constant_sampler,
            0x07,
            color_data[0],
            color_data[1],
            color_data[2],
            color_data[3],
            color_data[4],
            color_data[5],
            color_data[6],
            color_data[7],
            color_data[8],
            color_data[9],
            color_data[10],
            color_data[11],
            color_data[12],
            color_data[13],
            color_data[14],
            color_data[15],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn set_constant_sampler_unorm8(&mut self, constant_sampler: u8, color: [u8; 4]) -> Result<(), GpuCommandBuilderError> {
        let data = &[
            0x02,
            0x00,
            constant_sampler,
            0x03,
            color[0],
            color[1],
            color[2],
            color[3],
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
            0x00,
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn set_video_mode(&mut self, resolution: VideoResolution, backgrounds: bool, sprites: bool, triangles: bool) -> Result<(), GpuCommandBuilderError> {
        let mode_bits =
            (resolution as u8) |
            if backgrounds { 2 } else { 0 } |
            if sprites { 4 } else { 0 } |
            if triangles { 8 } else { 0 };
        let data = &[
            0x03,
            0x00,
            0x00,
            mode_bits
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn write_flag(&mut self, flag_address: u32, value: u32, interrupt: bool) -> Result<(), GpuCommandBuilderError> {
        let flag_address_bytes = command_u32_bytes(flag_address);
        let value_bytes        = command_u32_bytes(value);
        let data = &[
            0x02,
            if interrupt { 0x01 } else { 0x00 },
            flag_address_bytes[0],
            flag_address_bytes[1],
            flag_address_bytes[2],
            flag_address_bytes[3],
            value_bytes[0],
            value_bytes[1],
            value_bytes[2],
            value_bytes[3],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn configure_texture(&mut self, texture: u8, config: &TextureConfig) -> Result<(), GpuCommandBuilderError> {
        let width_bytes  = command_u16_bytes(config.width);
        let height_bytes = command_u16_bytes(config.height);
        let data = &[
            0x05,
            0x00,
            width_bytes[0],
            width_bytes[1],
            height_bytes[0],
            height_bytes[1],
            texture,
            config.pixel_layout as u8,
            config.image_layout as u8,
            0x00,
            0x00,
            0x00,
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn upload_texture(&mut self, texture: u8, texture_data_layout: ImageDataLayout, texture_data: *const u8) -> Result<(), GpuCommandBuilderError> {
        let data_address_bytes = command_u32_bytes(texture_data as usize as u32);
        let data = &[
            0x06,
            0x00,
            texture,
            texture_data_layout as u8,
            data_address_bytes[0],
            data_address_bytes[1],
            data_address_bytes[2],
            data_address_bytes[3],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn configure_buffer(&mut self, buffer: u8, length: u32) -> Result<(), GpuCommandBuilderError> {
        let length_bytes = command_u32_bytes(length as usize as u32);
        let data = &[
            0x07,
            0x00,
            0x00,
            buffer,
            length_bytes[0],
            length_bytes[1],
            length_bytes[2],
            length_bytes[3],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn upload_buffer(&mut self, buffer: u8, data: *const u8) -> Result<(), GpuCommandBuilderError> {
        let data_address_bytes = command_u32_bytes(data as usize as u32);
        let data = &[
            0x08,
            0x00,
            0x00,
            buffer,
            data_address_bytes[0],
            data_address_bytes[1],
            data_address_bytes[2],
            data_address_bytes[3],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn direct_blit(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16) -> Result<(), GpuCommandBuilderError> {
        let src_x_bytes = command_u16_bytes(src_x);
        let src_y_bytes = command_u16_bytes(src_y);
        let dst_x_bytes = command_u16_bytes(dst_x);
        let dst_y_bytes = command_u16_bytes(dst_y);
        let width_bytes = command_u16_bytes(width);
        let height_bytes = command_u16_bytes(height);
        let data = &[
            0x09,
            0x00,
            src_tex,
            dst_tex,
            src_x_bytes[0],
            src_x_bytes[1],
            src_y_bytes[0],
            src_y_bytes[1],
            dst_x_bytes[0],
            dst_x_bytes[1],
            dst_y_bytes[0],
            dst_y_bytes[1],
            width_bytes[0],
            width_bytes[1],
            height_bytes[0],
            height_bytes[1],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn cutout_blit(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, src_alpha_data_type: PixelDataType) -> Result<(), GpuCommandBuilderError> {
        let src_x_bytes = command_u16_bytes(src_x);
        let src_y_bytes = command_u16_bytes(src_y);
        let dst_x_bytes = command_u16_bytes(dst_x);
        let dst_y_bytes = command_u16_bytes(dst_y);
        let width_bytes = command_u16_bytes(width);
        let height_bytes = command_u16_bytes(height);
        let data = &[
            0x0A,
            0x00,
            src_tex,
            dst_tex,
            src_x_bytes[0],
            src_x_bytes[1],
            src_y_bytes[0],
            src_y_bytes[1],
            dst_x_bytes[0],
            dst_x_bytes[1],
            dst_y_bytes[0],
            dst_y_bytes[1],
            width_bytes[0],
            width_bytes[1],
            height_bytes[0],
            height_bytes[1],
            0x00,
            0x00,
            0x00,
            src_alpha_data_type as u8,
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn upload_shader(&mut self, index: u8, kind: ShaderKind, shader_code: &'static [u8]) -> Result<(), GpuCommandBuilderError> {
        let size_bytes = command_u16_bytes(shader_code.len() as u16);
        let address_bytes = command_u32_bytes(shader_code.as_ptr() as usize as u32);
        let data = &[
            0x0C,
            0x00,
            size_bytes[0],
            size_bytes[1],
            address_bytes[0],
            address_bytes[1],
            address_bytes[2],
            address_bytes[3],
            0x00,
            0x00,
            index,
            kind as u8
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn upload_graphics_pipeline_state(&mut self, index: u8, /*flags: u8, */state: &'static GraphicsPipelineState) -> Result<(), GpuCommandBuilderError> {
        let flags = 0u8;
        let state_address_bytes = command_u32_bytes(state as *const _ as usize as u32);
        let data = &[
            0x0D,
            0x00,
            index,
            flags,
            state_address_bytes[0],
            state_address_bytes[1],
            state_address_bytes[2],
            state_address_bytes[3],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }

    fn draw_graphics_pipeline(&mut self, state_index: u8, vertex_shader: u8, fragment_shader: u8, vertex_count: u32, clipping_rect: ClippingRect) -> Result<(), GpuCommandBuilderError> {
        let vertex_count_bytes = command_u32_bytes(vertex_count);
        let clipping_rect_x_low_bytes  = command_u16_bytes(clipping_rect.x_low );
        let clipping_rect_x_high_bytes = command_u16_bytes(clipping_rect.x_high);
        let clipping_rect_y_low_bytes  = command_u16_bytes(clipping_rect.y_low );
        let clipping_rect_y_high_bytes = command_u16_bytes(clipping_rect.y_high);
        let data = &[
            0x0F,
            0x00,
            state_index,
            fragment_shader,

            vertex_shader,
            0x00,
            0x00,
            0x00,

            vertex_count_bytes[0],
            vertex_count_bytes[1],
            vertex_count_bytes[2],
            vertex_count_bytes[3],

            clipping_rect_x_low_bytes[0],
            clipping_rect_x_low_bytes[1],
            clipping_rect_x_high_bytes[0],
            clipping_rect_x_high_bytes[1],

            clipping_rect_y_low_bytes[0],
            clipping_rect_y_low_bytes[1],
            clipping_rect_y_high_bytes[0],
            clipping_rect_y_high_bytes[1],
        ];
        if !self.push_command(data) {
			Err(GpuCommandBuilderError::OutOfSpace)
		} else {
			Ok(())
		}
    }
}

const GPU_COMMANDLIST_SUBMISSION_PORT: usize = 0x80010000;

pub fn gpu_submit<'c, Completion: CommandListCompletion, CommandList: CommandListData<GpuCommands>>(command_list: &mut CommandList, completion: &mut Completion) {
    unsafe {
        (completion.raw_ptr()).write_volatile(0);
        command_list.command_list_bytes()[4..8].copy_from_slice(&command_u32_bytes(completion.raw_ptr() as usize as u32));
        core::ptr::write(GPU_COMMANDLIST_SUBMISSION_PORT as * mut u32, command_list.command_list_bytes().as_ptr() as usize as u32);
    }
}
