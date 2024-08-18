
use std::collections::VecDeque;
use std::sync::Arc;

use bytemuck::{Pod, cast_slice, cast_slice_mut};

use crate::gpu::shader_parser::parse_shader_bytecode;
use crate::interrupt_controller::{INTERRUPT_CONTROLLER, InterruptType};
use crate::machine::{Machine, ReadResult};
use crate::ui::main_window::MainWindow;
use crate::command_list::{CommandList, retire_commandlist};

use super::command::Command;
use super::pipeline_state::GraphicsPipelineState;
use super::shader::{ShaderModule, ShaderType, ShadingUnitConstantArray, ShadingUnitContext, ShadingUnitIOArrays};
use super::rasterizer::{run_rasterizer, RasterRect, RasterizerCall};
use super::texture::*;
use super::buffer::*;
use super::types::{ConstantSampler, VideoMode, PixelDataLayout, ImageDataLayout, PixelDataType, ColorBlendOp, AlphaBlendOp};

pub struct Core {
    command_lists:      VecDeque<CommandList>,
    video_mode:         VideoMode,
    constant_samplers:  [ConstantSampler;       64],
    textures:           [TextureModule;         64],
    buffers:            [BufferModule;         256],
    shaders:            [ShaderModule;         128],
    graphics_states:    [GraphicsPipelineState; 64],
    shader_context:     Box<ShadingUnitContext>,
    shader_constants:   ShadingUnitConstantArray,
    shader_io_arrays:   Box<ShadingUnitIOArrays>,

}

impl Core {
    pub fn new() -> Self {
        Self {
            command_lists: VecDeque::new(),
            video_mode: VideoMode { resolution: super::types::VideoResolution::V256x192, backgrounds: false, sprites: false, triangles: false },
            constant_samplers: [(); 64].map(|_| ConstantSampler::new()),
            textures: [(); 64].map(|_| TextureModule::new()),
            buffers: [(); 256].map(|_| BufferModule::new()),
            shaders: [(); 128].map(|_| ShaderModule::default()),
            graphics_states: [(); 64].map(|_| GraphicsPipelineState::default()),
            shader_context: ShadingUnitContext::new(),
            shader_constants: ShadingUnitConstantArray::new(),
            shader_io_arrays: ShadingUnitIOArrays::new(),
        }
    }

    pub fn add_command_list(&mut self, list: CommandList) {
        self.command_lists.push_back(list);
    }

    pub fn process(&mut self, machine: &Arc<Machine>, main_window: &MainWindow) {
        while let Some(command_list) = self.command_lists.pop_front() {
            self.execute_command_list(command_list, machine, main_window);
        }
    }

    fn execute_command_list(&mut self, command_list: CommandList, machine: &Arc<Machine>, main_window: &MainWindow) {
        let mut offset = 0;
        while (offset as usize) < command_list.len() {
            if let Some((new_offset, command)) = Command::read(&command_list, offset) {
                self.execute_command(command, machine, main_window);
                offset = new_offset;
            } else {
                println!("command parse failed!");
                // todo: Set an error when we find an invalid command
                // for now, just skip the rest of the offending command list
                break;
            }
        }
        retire_commandlist(command_list);
    }

    fn execute_command(&mut self, command: Command, machine: &Arc<Machine>, main_window: &MainWindow) {
        match command {
            Command::ClearTexture { texture, constant_sampler } => 
                self.clear_texture(texture, constant_sampler),
            Command::PresentTexture { texture, completion_addr, interrupt } => 
                self.present_texture(texture, completion_addr, interrupt, machine, main_window),
            Command::SetConstantSampler { constant_sampler, set } => 
                self.set_constant_sampler(constant_sampler, set),
            Command::SetVideoMode(mode) => 
                self.set_video_mode(mode, main_window),
            Command::WriteFlag { address, value, irq } => 
                self.write_flag(machine, address, value, irq),
            Command::ConfigureTexture { texture, pixel_layout, image_layout, width, height } => 
                self.configure_texture(texture, pixel_layout, image_layout, width, height),
            Command::UploadTexture { texture, src_image_layout, src_addr } => 
                self.upload_texture(texture, src_image_layout, src_addr, machine),
            Command::ConfigureBuffer { buffer, length } => 
                self.configure_buffer(buffer, length),
            Command::UploadBuffer { buffer, src_addr } =>
                self.upload_buffer(buffer, src_addr, machine),
            Command::DirectBlit { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height } => 
                self.direct_blit(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
            Command::CutoutBlit { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, pixel_type } => 
                self.cutout_blit(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, pixel_type),
            Command::DrawBlendedRect { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, src_pixel_type, dst_pixel_type, color_blend_op, alpha_blend_op } =>
                self.draw_blended_rect(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, src_pixel_type, dst_pixel_type, color_blend_op, alpha_blend_op),
            Command::UploadShader { size, index, kind, address } =>
                self.upload_shader(index, kind, size, address, machine),
            Command::UploadGraphicsPipelineState { index, flags, address } =>
                self.upload_graphics_pipeline_state(index, flags, address, machine),
            Command::ConfigureGraphicsResourceMappings { pipeline, buffer_mapping_count, texture_mapping_count, buffer_mappings_addr, texture_mappings_addr } =>
                self.configure_graphics_resource_mappings(pipeline, buffer_mapping_count, texture_mapping_count, buffer_mappings_addr, texture_mappings_addr),
            Command::DrawGraphicsPipeline { state_index, vertex_shader, fragment_shader, vertex_count, x_low, x_high, y_low, y_high } => {
                let target_rect = RasterRect {
                    upper_left:  (x_low  as u32, y_low  as u32),
                    lower_right: (x_high as u32, y_high as u32),
                };
                self.draw_graphics_pipeline(state_index, vertex_shader, fragment_shader, vertex_count, target_rect);
            },
            Command::WriteBuffer { buffer, src_addr, length, offset } => 
                self.write_buffer(buffer, src_addr, length, offset, machine),
        }
    }

    fn set_video_mode(&mut self, mode: VideoMode, main_window: &MainWindow) {
        println!("GPU: set_video_mode({:?})", mode);
        self.video_mode = mode;
        main_window.set_video_resolution(mode.resolution);
    }

    fn set_constant_sampler(&mut self, sampler: u8, value: ConstantSampler) {
        println!("GPU: set_constant_sampler({}, {:?})", sampler, value);
        if sampler >= 64 {
            println!("GPU: set_constant_sampler ERROR: constant sampler out of range!");
            return;
        }
        self.constant_samplers[sampler as usize] = value;
    }

    fn configure_texture(&mut self, texture: u8, pixel_layout: PixelDataLayout, image_layout: ImageDataLayout, width: u32, height: u32) {
        println!("GPU: configure_texture({}, pixel_layout: {:?}, image_layout: {:?}, width: {}, height: {})", texture, pixel_layout, image_layout, width, height);
        let texture_regs = &mut self.textures[texture as usize];
        texture_regs.config.width = width as u16;
        texture_regs.config.height = height as u16;
        texture_regs.config.image_layout = image_layout;
        texture_regs.config.pixel_layout = pixel_layout;
    }

    fn present_texture(&mut self, texture: u8, completion_addr: u32, interrupt: bool, machine: &Arc<Machine>, main_window: &MainWindow) {
        println!("GPU: present_texture({}, completion: {:08X}, interrupt: {:?})", texture, completion_addr, interrupt);
        if texture >= 64 {
            println!("GPU: present_texture ERROR: texture out of range!");
            return;
        }
        let texture = &self.textures[texture as usize];
        main_window.present_texture(texture.memory.as_ptr() as *const u8, completion_addr, interrupt, machine.clone())
    }

    fn clear_texture(&mut self, texture: u8, constant_sampler: u8) {
        println!("GPU: clear_texture(texture: {}, constant_sampler: {})", texture, constant_sampler);
        if texture >= 64 {
            println!("GPU: clear_texture ERROR: texture out of range!");
            return;
        }
        if constant_sampler >= 64 {
            println!("GPU: clear_texture ERROR: constant_sampler out of range!");
            return;
        }
        let sampler = self.constant_samplers[constant_sampler as usize];
        let value_abstract = sampler.get_abstract();
        self.textures[texture as usize].clear(value_abstract);
    }

    fn write_flag(&mut self, machine: &Arc<Machine>, address: u32, value: u32, interrupt: bool) {
        println!("GPU: write_flag(address: {:08X}, value: {:08X}, interrupt: {:?})", address, value, interrupt);
        if !machine.write_u32(address, value).is_ok() {
            println!("GPU: write_flag ERROR: bad address!");
        };
        std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
        if interrupt {
            INTERRUPT_CONTROLLER.trigger_interrupt(InterruptType::Gpu);
        }
    }

    fn upload_texture(&mut self, texture: u8, src_image_layout: ImageDataLayout, src_addr: u32, machine: &Arc<Machine>) {
        println!("GPU: upload_texture(texture: {texture}, src_image_layout: {:?}, src_addr: {:08X})", src_image_layout, src_addr);
        if texture >= 64 {
            return;
        }
        let texture = &mut self.textures[texture as usize];
        std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
        if src_image_layout == texture.config.image_layout {
            let _ = machine.read_block(src_addr, texture.data_slice_mut());
        } else {
            match texture.config.pixel_layout.pixel_bytes() {
                1 => Self::texcopy_cross_layout_internal::<1>(texture, src_image_layout, src_addr, machine),
                2 => Self::texcopy_cross_layout_internal::<2>(texture, src_image_layout, src_addr, machine),
                4 => Self::texcopy_cross_layout_internal::<4>(texture, src_image_layout, src_addr, machine),
                8 => Self::texcopy_cross_layout_internal::<8>(texture, src_image_layout, src_addr, machine),
                16 => Self::texcopy_cross_layout_internal::<16>(texture, src_image_layout, src_addr, machine),
                _ => unreachable!()
            }
        }
    }

    fn texcopy_cross_layout_internal<const N: usize>(texture: &mut TextureModule, src_image_layout: ImageDataLayout, src_addr: u32, machine: &Arc<Machine>)
        where [u8; N]: Pod {
        let mut pixel_val = [0u8; N];
        for y in 0..texture.config.height as u32 {
            for x in 0..texture.config.width as u32 {
                let src_index = src_image_layout.index(x, y, texture.config.width as u32);
                let src_offset = src_index * N as u32;
                machine.read_block(src_addr + src_offset, &mut pixel_val[0..N]);
                texture.store(x, y, pixel_val);
            }
        }
    }

    fn configure_buffer(&mut self, buffer: u8, length: u32) {
        println!("GPU: configure_buffer(buffer: {buffer}, buffer_length: {:08X}", length);
        self.buffers[buffer as usize].length = length.min(BUFFER_MAX_SIZE);
    }

    fn upload_buffer(&mut self, buffer: u8, src_addr: u32, machine: &Arc<Machine>) {
        println!("GPU: upload_buffer(buffer: {buffer}, src_addr: {:08X}) buffer_length: {:08X}", src_addr, self.buffers[buffer as usize].length);
        let buffer_slice = self.buffers[buffer as usize].bytes_mut();
        std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
        match machine.read_block(src_addr, buffer_slice) {
            ReadResult::Ok(_) => {},
            ReadResult::InvalidAddress => println!("invalid buffer address!"),
        }
    }

    fn direct_blit(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16) {
        println!("GPU: direct_blit(src_texture: {src_tex}, dest_texture: {dst_tex}, src_x: {src_x}, src_y: {src_y}, dst_x: {dst_x}, dst_y: {dst_y}, width: {width}, height: {height})");
        if src_tex == dst_tex {
            return;
        }
        unsafe {
            let src_tex = &*(&self.textures[src_tex as usize] as *const TextureModule);
            let dst_tex = &mut *(&mut self.textures[dst_tex as usize] as *mut TextureModule);
            if src_tex.config.pixel_layout == dst_tex.config.pixel_layout {
                match dst_tex.config.pixel_layout.pixel_bytes() {
                    1  => Self::direct_blit_internal::< 1>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
                    2  => Self::direct_blit_internal::< 2>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
                    4  => Self::direct_blit_internal::< 4>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
                    8  => Self::direct_blit_internal::< 8>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
                    16 => Self::direct_blit_internal::<16>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height),
                    _ => unreachable!()
                }
            }
        }
    }

    fn direct_blit_internal<const N: usize>(src_tex: &TextureModule, dst_tex: &mut TextureModule, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16)
        where [u8; N]: Pod {
        println!("direct_blit_internal::<{}> src config: {:?}, dst config: {:?}", N, src_tex.config, dst_tex.config);
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let pixel = src_tex.fetch::<[u8; N]>(x + src_x as u32, y + src_y as u32);
                dst_tex.store::<[u8; N]>(x + dst_x as u32, y + dst_y as u32, pixel);
            }
        }
    }

    fn cutout_blit(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, pixel_type: PixelDataType) {
        println!("GPU: direct_blit(src_texture: {src_tex}, dest_texture: {dst_tex}, src_x: {src_x}, src_y: {src_y}, dst_x: {dst_x}, dst_y: {dst_y}, width: {width}, height: {height})");
        if src_tex == dst_tex {
            return;
        }
        unsafe {
            let src_tex = &*(&self.textures[src_tex as usize] as *const TextureModule);
            let dst_tex = &mut *(&mut self.textures[dst_tex as usize] as *mut TextureModule);
            if src_tex.config.pixel_layout == dst_tex.config.pixel_layout {
                match (pixel_type, dst_tex.config.pixel_layout.pixel_bytes()) {
                    (PixelDataType::RgbaUNorm8,  4) => Self::cutout_blit_internal::<4, _>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, |bytes| bytes[3] != 0),
                    (PixelDataType::RgbaF32,    16) => Self::cutout_blit_internal::<16, _>(src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, |bytes| cast_slice::<_, f32>(&bytes[..])[3] > 0.0),
                    _ => unreachable!()
                }
            } else {
                println!("WARNING: cutout_blit with dest type != src type unimplemented!");
            }
        }
    }

    fn cutout_blit_internal<const N: usize, F: Fn(&[u8; N]) -> bool>(src_tex: &TextureModule, dst_tex: &mut TextureModule, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, test_fn: F)
        where [u8; N]: Pod {
        println!("cutout_blit_internal::<{}> src config: {:?}, dst config: {:?}", N, src_tex.config, dst_tex.config);
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let pixel = src_tex.fetch::<[u8; N]>(x + src_x as u32, y + src_y as u32);
                if test_fn(&pixel) {
                    dst_tex.store::<[u8; N]>(x + dst_x as u32, y + dst_y as u32, pixel);
                }
            }
        }
    }

    fn draw_blended_rect(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, src_pixel_type: PixelDataType, dst_pixel_type: PixelDataType, color_blend_op: ColorBlendOp, alpha_blend_op: AlphaBlendOp) {
        if src_pixel_type.component_count() != dst_pixel_type.component_count() {
            return;
        }
        match (src_pixel_type, dst_pixel_type) {
            (PixelDataType::RgbaUNorm8, PixelDataType::RgbaUNorm8) => self.draw_blended_rect_rgba(
                src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, color_blend_op, alpha_blend_op,
                &read_rgba_unorm8,
                &read_rgba_unorm8,
                &write_rgba_unorm8
            ),
            (PixelDataType::RgbaF32, PixelDataType::RgbaUNorm8) => self.draw_blended_rect_rgba(
                src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, color_blend_op, alpha_blend_op,
                &read_rgba_f32,
                &read_rgba_unorm8,
                &write_rgba_unorm8
            ),
            (PixelDataType::RgbaUNorm8, PixelDataType::RgbaF32) => self.draw_blended_rect_rgba(
                src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, color_blend_op, alpha_blend_op,
                &read_rgba_unorm8,
                &read_rgba_f32,
                &write_rgba_f32,
            ),
            (PixelDataType::RgbaF32, PixelDataType::RgbaF32) => self.draw_blended_rect_rgba(
                src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, color_blend_op, alpha_blend_op,
                &read_rgba_f32,
                &read_rgba_f32,
                &write_rgba_f32,
            ),
            _ => println!("GPU: unimplemented pixel data type set in draw_blended_rect(): src: {:?}, dst: {:?}", src_pixel_type, dst_pixel_type)
        }
    }
    
    fn draw_blended_rect_rgba<const N_SRC: usize, const N_DST: usize, FSrcRead: Fn(&[u8; N_SRC]) -> [f32; 4], FDstRead: Fn(&[u8; N_DST])-> [f32; 4], FDstWrite: Fn(&mut [u8; N_DST], [f32; 4])>(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, color_blend_op: ColorBlendOp, alpha_blend_op: AlphaBlendOp, f_src_read: &FSrcRead, f_dst_read: &FDstRead, f_dst_write: &FDstWrite)
        where [u8; N_SRC]: Pod, [u8; N_DST]: Pod {
        let color_fn: fn([f32; 4], [f32; 4]) -> [f32; 3] = match color_blend_op {
            ColorBlendOp::Zero  => |  _,   _| [
                0.0,
                0.0,
                0.0
            ],
            ColorBlendOp::Src   => |src,   _| [
                src[0],
                src[1],
                src[2],
            ],
            ColorBlendOp::Dst   => |  _, dst| [
                dst[0],
                dst[1],
                dst[2],
            ],
            ColorBlendOp::Add   => |src, dst| [
                src[0] + dst[0],
                src[1] + dst[1],
                src[2] + dst[2]
            ],
            ColorBlendOp::Sub   => |src, dst| [
                dst[0] - src[0],
                dst[1] - src[1],
                dst[2] - src[2]
            ],
            ColorBlendOp::RSub  => |src, dst| [
                src[0] - dst[0],
                src[1] - dst[1],
                src[2] - dst[2]
            ],
            ColorBlendOp::Avg   => |src, dst| [
                (src[0] + dst[0]) * 0.5,
                (src[1] + dst[1]) * 0.5,
                (src[2] + dst[2]) * 0.5
            ],
            ColorBlendOp::Blend => |src, dst| [
                src[0] * src[3] + dst[0] * (1.0 - src[3]),
                src[1] * src[3] + dst[1] * (1.0 - src[3]),
                src[2] * src[3] + dst[2] * (1.0 - src[3]),
            ],
            ColorBlendOp::RBlend => |src, dst| [
                src[0] * (1.0 - src[3]) + dst[0] * src[3],
                src[1] * (1.0 - src[3]) + dst[1] * src[3],
                src[2] * (1.0 - src[3]) + dst[2] * src[3],
            ],
        };
        let alpha_fn = match alpha_blend_op {
            AlphaBlendOp::Zero => |_, _| 0.0,
            AlphaBlendOp::One => |_, _| 1.0,
            AlphaBlendOp::Src => |src, _| src,
            AlphaBlendOp::Dst => |_, dst| dst,
            AlphaBlendOp::Avg => |src, dst| (src + dst) * 0.5,
            AlphaBlendOp::Add => |src, dst| src + dst,
            AlphaBlendOp::Sub => |src, dst| dst - src,
            AlphaBlendOp::RSub => |src, dst| src - dst,
            AlphaBlendOp::Blend => |src, dst| dst + (1.0 - dst) * src,
        };
        let src_texture = unsafe { & (*(&self.textures[src_tex as usize] as *const _)) };
        let dst_texture = unsafe { &mut (*(&mut self.textures[dst_tex as usize] as *mut _)) };
        Self::draw_blended_rect_internal(
            src_texture, dst_texture, src_x, src_y, dst_x, dst_y, width, height,
            |src_bytes, dst_bytes| {
                let src = f_src_read(src_bytes);
                let dst = f_dst_read(dst_bytes);
                let color_result = color_fn(src, dst);
                let alpha_result = alpha_fn(src[3], dst[3]);
                f_dst_write(dst_bytes, [color_result[0], color_result[1], color_result[2], alpha_result]);
            }
        )
    }
    
    #[allow(unused)]
    fn draw_blended_rect_color<const N_SRC: usize, const N_DST: usize, F: Fn(&[u8; N_SRC], &mut [u8; N_DST])>(&mut self, src_tex: u8, dst_tex: u8, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, width: u16, height: u16, color_blend_op: ColorBlendOp) {
        todo!()
    }

    fn draw_blended_rect_internal<const N_SRC: usize, const N_DST: usize, F: Fn(&[u8; N_SRC], &mut [u8; N_DST])>(src_tex: &TextureModule, dst_tex: &mut TextureModule, src_x: u16, src_y: u16, dst_x: u16, dst_y: u16, mut width: u16, mut height: u16, blend_fn: F)
        where [u8; N_SRC]: Pod, [u8; N_DST]: Pod {
        println!("    GPU::draw_blended_rect_internal::<N_SRC: {N_SRC}, N_DST: {N_DST}>(): src config: {:?}, dst config: {:?}", src_tex.config, dst_tex.config);
        if src_x >= src_tex.config.width || src_y >= src_tex.config.width || dst_x >= dst_tex.config.width || dst_y >= dst_tex.config.height {
            return;
        }
        width = width.min(src_tex.config.width - src_x).min(dst_tex.config.width - dst_x);
        height = height.min(src_tex.config.height - src_y).min(dst_tex.config.height - dst_y);
        let src_pixels = cast_slice::<u8, [u8; N_SRC]>(src_tex.data_slice());
        let dst_pixels = cast_slice_mut::<u8, [u8; N_DST]>(dst_tex.data_slice_mut());
        for y in 0..height as u32 {
            for x in 0..width as u32 {
                let src_bytes = &src_pixels[((y + src_y as u32) * src_tex.config.width as u32 + x + src_x as u32) as usize];
                let dst_bytes = &mut dst_pixels[((y + dst_y as u32) * src_tex.config.width as u32 + x + dst_x as u32) as usize];
                blend_fn(src_bytes, dst_bytes);
            }
        }
    }

    fn upload_shader(&mut self, index: u8, kind: ShaderType, size: u16, address: u32, machine: &Arc<Machine>) {
        println!("GPU: upload_shader(index: {}, kind: {:?}, size: {}, address: {:08X})", index, kind, size, address);
        let mut bytes = vec![0u8; size as usize].into_boxed_slice();
        match machine.read_block(address, &mut bytes) {
            crate::machine::ReadResult::Ok(_) => {},
            crate::machine::ReadResult::InvalidAddress => return,
        };
        let module = &mut self.shaders[(index & 0x7F) as usize];
        match parse_shader_bytecode(kind, &bytes[..], module) {
            Ok(()) => {},
            Err(e) => println!("GPU: shader bytecode parse failed: {:?}", e)
        }
    }

    fn upload_graphics_pipeline_state(&mut self, index: u8, flags: u8, address: u32, machine: &Arc<Machine>) {
        println!("GPU: upload_graphics_pipeline_state(index: {}, flags: {:b}, address: {:08X}", index, flags, address);
        if let Some(state) = GraphicsPipelineState::read_from_address(address, machine) {
            self.graphics_states[index as usize] = state;
        } else {
            println!("Pipeline state upload failed");
        }
    }

    fn configure_graphics_resource_mappings(&mut self, state_index: u8, buffer_count: u8, texture_count: u8, buffer_mapping_list_addr: u32, texture_mapping_list_addr: u32) {
        println!("GPU: configure_graphics_resource_mappings(state_index: {}, buffer_count: {}, texture_count: {}, buffer_mapping_list_addr: {}, texture_mapping_list_addr: {})", state_index, buffer_count, texture_count, buffer_mapping_list_addr, texture_mapping_list_addr);
    }

    fn draw_graphics_pipeline(&mut self, state: u8, vertex_shader: u8, fragment_shader: u8, vertex_count: u32, target_rect: RasterRect) {
        let state = &self.graphics_states[state as usize];
        let rasterizer_call = RasterizerCall {
            constant_array: &mut self.shader_constants,
            io_arrays: &mut self.shader_io_arrays.0,
            buffer_modules: &mut self.buffers,
            texture_modules: &mut self.textures,
            shader_modules: &mut self.shaders,
            vertex_count: vertex_count as usize,
            shading_unit_context: &mut self.shader_context,
            state: &state.raster_state,
            vertex_shader,
            fragment_shader,
            vertex_state: &state.vertex_state,
            fragment_state: &state.fragment_state,
            target_rect,
            resource_map: &state.raster_state.resource_map,
        };
        run_rasterizer(rasterizer_call);
    }

    fn write_buffer(&mut self, buffer: u8, src_addr: u32, length: u32, offset: u32, machine: &Arc<Machine>) {
        let buffer_slice = self.buffers[buffer as usize].bytes_mut();
        println!("GPU: write_buffer(buffer: {}, src_addr: {:08X}, length: {:X}, offset: {:08X})", buffer, src_addr, length, offset);
        if (offset + length) as usize > buffer_slice.len() {
            println!("buffer write overflows buffer!");
            return;
        }
        std::sync::atomic::fence(std::sync::atomic::Ordering::AcqRel);
        match machine.read_block(src_addr, &mut buffer_slice[offset as usize..(offset + length) as usize]) {
            ReadResult::Ok(_) => {},
            ReadResult::InvalidAddress => println!("invalid data address address!"),
        }
    }
}

fn unorm8_to_f32(x: u8) -> f32 {
    x as f32 * 0.00392156862745098
}

fn f32_to_unorm8(x: f32) -> u8 {
    (x * 255.999) as u8
}

#[allow(unused)]
fn read_r_unorm8(bytes: &[u8; 1]) -> [f32; 4] {
    [
        unorm8_to_f32(bytes[0]),
        0.0,
        0.0,
        0.0,
    ]
}

#[allow(unused)]
fn read_rg_unorm8(bytes: &[u8; 2]) -> [f32; 4] {
    [
        unorm8_to_f32(bytes[0]),
        unorm8_to_f32(bytes[1]),
        0.0,
        0.0,
    ]
}

fn read_rgba_unorm8(bytes: &[u8; 4]) -> [f32; 4] {
    [
        unorm8_to_f32(bytes[0]),
        unorm8_to_f32(bytes[1]),
        unorm8_to_f32(bytes[2]),
        unorm8_to_f32(bytes[3])
    ]
}

fn write_rgba_unorm8(bytes: &mut [u8; 4], value: [f32; 4]) {
    bytes[0] = f32_to_unorm8(value[0]);
    bytes[1] = f32_to_unorm8(value[1]);
    bytes[2] = f32_to_unorm8(value[2]);
    bytes[3] = f32_to_unorm8(value[3]);
}

fn read_rgba_f32(bytes: &[u8; 16]) -> [f32; 4] {
    cast_slice::<u8, [f32; 4]>(bytes)[0]
}

fn write_rgba_f32(bytes: &mut [u8; 16], value: [f32; 4]) {
    cast_slice_mut::<u8, [f32; 4]>(bytes)[0] = value;
}
