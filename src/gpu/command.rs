use super::{types::*, command_list::CommandList};

pub enum Command {
    /*
    clear_texture <texture> <c. sampler>
    [     00 00 ] [    XX ] [       yy ]
     */
    ClearTexture { texture: u8, constant_sampler: u8 },
    /*
    present_texture <texture> <interrupt> <completion addr> 
    [       01 00 ] [    XX ] [      ZZ ] [   YY YY YY YY ] 
     */
    PresentTexture { texture: u8, completion_addr: u32, interrupt: bool },
    /*
    set_constant_sampler <sampler> <data_type> <  data..  > < padding to 20 bytes      >
    [            02 00 ] [    XX ] [      YY ] [ ZZ (1-16)] [                    00 .. ]
     */
    SetConstantSampler { constant_sampler: u8, set: ConstantSampler },
    /*
    set_video_mode   ..   <video mode>
    [      03 00 ] [ 00 ] [       0X ] 
    X: bits:
        0: resolution:
            0 - 256 x 192
            1 - 512 x 384
        1: enable sprites
        2: enable backgrounds
        3: enable triangles
     */
    SetVideoMode(VideoMode),
    /*
    write_flag   ..   < IRQ> <   address   > <    value    >
    [  04 00 ] [ 00 ] [ XX ] [ ZZ ZZ ZZ ZZ ] [ YY YY YY YY ]
     */
    WriteFlag {
        address: u32,
        value: u32,
        irq: bool
    },
    /*
    configure_texture < width > <height > <texture> <pixel_layout> <image_layout>    ..
    [         05 00 ] [ XX XX ] [ YY YY ] [    ZZ ] [         UU ] [         VV ] [ 00 00 00 ]
     */
    ConfigureTexture {
        width: u32,
        height: u32,
        texture: u8,
        pixel_layout: PixelDataLayout,
        image_layout: ImageDataLayout,
    },
    /*
    upload_texture <texture> < src_image_layout> < src_address >
    [      06 00 ] [    ZZ ] [              UU ] [ VV VV VV VV ]
     */
    UploadTexture {
        texture: u8,
        src_image_layout: ImageDataLayout,
        src_addr: u32,
    },
    /*
    configure_buffer   ..   <buffer> <    length   >
    [        07 00 ] [ 00 ] [   XX ] [ YY YY YY YY ]
     */
    ConfigureBuffer {
        buffer: u8,
        length: u32,
    },
    /*
    upload_buffer   ..   <buffer> <   src_addr   >
    [     08 00 ] [ 00 ] [   XX ] [  YY YY YY YY ]
     */
    UploadBuffer {
        buffer: u8,
        src_addr: u32,
    },
    /*
    direct_blit < src_tex > < dst_tex > < src_x > < src_y > < dst_x > < dst_y > < width > < height>
    [   09 00 ] [      SS ] [      DD ] [ XX XX ] [ YY YY ] [ ZZ ZZ ] [ WW WW ] [ UU UU ] [ VV VV ]
     */
    DirectBlit {
        src_tex: u8,
        dst_tex: u8,
        src_x: u16,
        src_y: u16,
        dst_x: u16,
        dst_y: u16,
        width: u16,
        height: u16
    },
    
    /*
    cutout_blit < src_tex > < dst_tex > < src_x > < src_y > < dst_x > < dst_y > < width > < height>      ..      < src_pixel_dt >
    [   0A 00 ] [      SS ] [      DD ] [ XX XX ] [ YY YY ] [ ZZ ZZ ] [ WW WW ] [ UU UU ] [ VV VV ] [ 00 00 00 ] [           TT ]
     */
    CutoutBlit {
        src_tex: u8,
        dst_tex: u8,
        src_x: u16,
        src_y: u16,
        dst_x: u16,
        dst_y: u16,
        width: u16,
        height: u16,
        pixel_type: PixelDataType,
    },
    /*
    draw_blended_rect < src_tex > < dst_tex > < src_x > < src_y > < dst_x > < dst_y > < width > < height> < src_pixel_dt > < dst_pixel_dt > < blend_op_rgb > < blend_op_alpha >
    [         0B 00 ] [      SS ] [      DD ] [ XX XX ] [ yy yy ] [ ZZ ZZ ] [ WW WW ] [ UU UU ] [ VV VV ] [           TT ] [           QQ ] [           RR ] [             AA ]
     */
    DrawBlendedRect {
        src_tex: u8,
        dst_tex: u8,
        src_x: u16,
        src_y: u16,
        dst_x: u16,
        dst_y: u16,
        width: u16,
        height: u16,
        src_pixel_type: PixelDataType,
        dst_pixel_type: PixelDataType,
        color_blend_op: ColorBlendOp,
        alpha_blend_op: AlphaBlendOp,
    }
}

impl Command {
    pub fn read(command_list: &CommandList, offset: u32) -> Option<(u32, Self)> {
        match command_list.read_u16(offset) {
            Some(0x00_00) => {
                let texture = command_list.read_u8(offset + 2)? & 0x1F;
                let constant_sampler = command_list.read_u8(offset + 3)? & 0x3F;
                Some((offset + 4, Command::ClearTexture { texture, constant_sampler }))
            },
            Some(0x00_01) => {
                let texture = command_list.read_u8(offset + 2)? & 0x1F;
                let interrupt = command_list.read_u8(offset + 3)? != 0;
                let completion_addr = command_list.read_u32(offset + 4)?;
                Some((offset + 8, Command::PresentTexture { texture, completion_addr, interrupt }))
            },
            Some(0x00_02) => {
                let constant_sampler = command_list.read_u8(offset + 2)? & 0x3F;
                let data_type = command_list.read_u8(offset + 3)? & 0x7;
                let (data_type, buffer_size) = match data_type {
                    0 => (PixelDataType::RUNorm8,     1),
                    1 => (PixelDataType::RgUNorm8,    2),
                    2 => (PixelDataType::RgbUNorm8,   3),
                    3 => (PixelDataType::RgbaUNorm8,  4),
                    4 => (PixelDataType::RF32,        4),
                    5 => (PixelDataType::RgF32,       8),
                    6 => (PixelDataType::RgbF32,     12),
                    7 => (PixelDataType::RgbaF32,    16),
                    _ => unreachable!()
                };
                let mut constant_data = [0u8; 16];
                {
                    let data_slice = &mut constant_data[0..buffer_size];
                    data_slice.iter_mut().enumerate().for_each(|(i, x)| *x = command_list.read_u8(offset + 4 + i as u32).unwrap_or_default());
                }
                Some((offset + 20, Command::SetConstantSampler {
                    constant_sampler,
                    set: ConstantSampler { constant_data, data_type }
                }))
            },
            Some(0x00_03) => {
                let vide_mode = command_list.read_u8(offset + 3)? & 0x0F;
                Some((offset + 4, Command::SetVideoMode(
                    VideoMode {
                        resolution: if vide_mode & 1 != 0 { VideoResolution::V512x384 } else { VideoResolution::V256x192 },
                        backgrounds:   vide_mode & 2 != 0,
                        sprites:       vide_mode & 4 != 0,
                        triangles:     vide_mode & 8 != 0,
                    }
                )))
            },
            Some(0x00_04) => {
                let irq = command_list.read_u8(offset + 3)? != 0;
                let address = command_list.read_u32(offset + 4)?;
                let value = command_list.read_u32(offset + 8)?;
                Some((offset + 12, Command::WriteFlag { address, value, irq }))
            },
            Some(0x00_05) => {
                let width = command_list.read_u16(offset + 2)? as u32;
                let height = command_list.read_u16(offset + 4)? as u32;
                let texture = command_list.read_u8(offset + 6)?;
                let pixel_layout = PixelDataLayout::from_u8(command_list.read_u8(offset + 7)?)?;
                let image_layout = ImageDataLayout::from_u8(command_list.read_u8(offset + 8)?)?;
                Some((offset + 12, Command::ConfigureTexture { texture, pixel_layout, image_layout, width, height }))
            },
            Some(0x00_06) => {
                let texture = command_list.read_u8(offset + 2)?;
                let src_image_layout = ImageDataLayout::from_u8(command_list.read_u8(offset + 3)?)?;
                let src_addr = command_list.read_u32(offset + 4)?;
                Some((offset + 8, Command::UploadTexture { texture, src_image_layout, src_addr }))
            },
            Some(0x00_07) => {
                let buffer = command_list.read_u8(offset + 3)?;
                let length = command_list.read_u32(4)?;
                Some((offset + 8, Command::ConfigureBuffer { buffer, length }))
            },
            Some(0x00_08) => {
                let buffer = command_list.read_u8(offset + 3)?;
                let src_addr = command_list.read_u32(4)?;
                Some((offset + 8, Command::UploadBuffer { buffer, src_addr }))
            },
            Some(0x00_09) => {
                let src_tex = command_list.read_u8(offset + 2)?;
                let dst_tex = command_list.read_u8(offset + 3)?;
                let src_x = command_list.read_u16(offset + 4)?;
                let src_y = command_list.read_u16(offset + 6)?;
                let dst_x = command_list.read_u16(offset + 8)?;
                let dst_y = command_list.read_u16(offset + 10)?;
                let width = command_list.read_u16(offset + 12)?;
                let height = command_list.read_u16(offset + 14)?;
                Some((offset + 16, Command::DirectBlit { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height }))
            },
            Some(0x00_0A) => {
                let src_tex = command_list.read_u8(offset + 2)?;
                let dst_tex = command_list.read_u8(offset + 3)?;
                let src_x = command_list.read_u16(offset + 4)?;
                let src_y = command_list.read_u16(offset + 6)?;
                let dst_x = command_list.read_u16(offset + 8)?;
                let dst_y = command_list.read_u16(offset + 10)?;
                let width = command_list.read_u16(offset + 12)?;
                let height = command_list.read_u16(offset + 14)?;
                let pixel_type = PixelDataType::from_u8(command_list.read_u8(offset + 19)?)?;
                Some((offset + 20, Command::CutoutBlit { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, pixel_type }))
            },
            Some(0x00_0B) => {
                let src_tex = command_list.read_u8(offset + 2)?;
                let dst_tex = command_list.read_u8(offset + 3)?;
                let src_x = command_list.read_u16(offset + 4)?;
                let src_y = command_list.read_u16(offset + 6)?;
                let dst_x = command_list.read_u16(offset + 8)?;
                let dst_y = command_list.read_u16(offset + 10)?;
                let width = command_list.read_u16(offset + 12)?;
                let height = command_list.read_u16(offset + 14)?;
                let src_pixel_type = PixelDataType::from_u8(command_list.read_u8(offset + 16)?)?;
                let dst_pixel_type = PixelDataType::from_u8(command_list.read_u8(offset + 17)?)?;
                let color_blend_op = ColorBlendOp::from_u8(command_list.read_u8(offset + 18)?)?;
                let alpha_blend_op = AlphaBlendOp::from_u8(command_list.read_u8(offset + 19)?)?;
                Some((offset + 20, Command::DrawBlendedRect { src_tex, dst_tex, src_x, src_y, dst_x, dst_y, width, height, src_pixel_type, dst_pixel_type, color_blend_op, alpha_blend_op }))
            },
            Some(0x00_0C) => {
                None
            },
            Some(0x00_0D) => {
                None
            },
            Some(0x00_0E) => {
                None
            },
            Some(0x00_0F) => {
                None
            }
            _  => None,
        }
    }
}
