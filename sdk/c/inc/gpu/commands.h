#pragma once

#include "command_list.h"

typedef enum {
    PixelDataLayoutD8x1 = 0,
    PixelDataLayoutD8x2 = 1,
    PixelDataLayoutD8x4 = 2,
    PixelDataLayoutD16x1 = 3,
    PixelDataLayoutD16x2 = 4,
    PixelDataLayoutD16x4 = 5,
    PixelDataLayoutD32x1 = 6,
    PixelDataLayoutD32x2 = 7,
    PixelDataLayoutD32x4 = 8,
} PixelDataLayout;

typedef enum {
    ImageDataLayoutContiguous = 0,
    ImageDataLayoutBlock8x8 = 1,
    ImageDataLayoutBlock4x4 = 2,
} ImageDataLayout;

typedef enum {
    PixelDataTypeRUNorm8 = 0,
    PixelDataTypeRgUNorm8 = 1,
    PixelDataTypeRgbUNorm8 = 2,
    PixelDataTypeRgbaUNorm8 = 3,
    PixelDataTypeRF32 = 4,
    PixelDataTypeRgF32 = 5,
    PixelDataTypeRgbF32 = 6,
    PixelDataTypeRgbaF32 = 7,
} PixelDataType;

#define COMMAND_ENCODED_U16(x) \
    (u8) (((u32) x) >> 0), \
    (u8) (((u32) x) >> 8)

#define COMMAND_ENCODED_U32(x) \
    (u8) (((u32) x) >> 0), \
    (u8) (((u32) x) >> 8), \
    (u8) (((u32) x) >> 16), \
    (u8) (((u32) x) >> 24)

#define PUSH_COMMAND \
    push_command(recorder, data, sizeof(data))

inline static bool gpu_command_clear_texture(CommandListRecorder * recorder, u8 texture, u8 constant_sampler) {
    u8 data[] = {
        0x00,
        0x00,
        texture,
        constant_sampler
    };
    return PUSH_COMMAND;
}

// present_texture <texture> <interrupt> <completion addr> 
// [       00 01 ] [    XX ] [      ZZ ] [   YY YY YY YY ] 

inline static bool gpu_command_present_texture(CommandListRecorder * recorder, u8 texture, volatile u32 * completion_flag, bool completion_interrupt) {
    u8 data[] = {
        0x01,
        0x00,
        texture,
        completion_interrupt ? 1 : 0,
        COMMAND_ENCODED_U32(completion_flag)
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_set_constant_sampler_rgba_f32(CommandListRecorder * recorder, u8 constant_sampler, const f32 colors[4]) {
    const u8 * colors_raw = (const u8 *) colors;
    u8 data[] = {
        0x02,
        0x00,
        constant_sampler,
        0x07,             // PixelDataType::RgbaF32
        colors_raw[0],
        colors_raw[1],
        colors_raw[2],
        colors_raw[3],
        colors_raw[4],
        colors_raw[5],
        colors_raw[6],
        colors_raw[7],
        colors_raw[8],
        colors_raw[9],
        colors_raw[10],
        colors_raw[11],
        colors_raw[12],
        colors_raw[13],
        colors_raw[14],
        colors_raw[15],
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_set_constant_sampler_rgba_unorm8(CommandListRecorder * recorder, u8 constant_sampler, const u8 colors[4]) {
    u8 data[] = {
        0x02,
        0x00,
        constant_sampler,
        0x03,             // PixelDataType::RgbaF32
        colors[0],
        colors[1],
        colors[2],
        colors[3],
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
    };
    return PUSH_COMMAND;
}

typedef enum {
    VideoResolution512x384 = 1,
    VideoResolution256x192 = 0,
} VideoResolution;

inline static bool gpu_command_set_video_mode(CommandListRecorder * recorder, VideoResolution resolution, bool backgrounds, bool sprites, bool triangles) {
    u8 mode_bits = 
        (u8) resolution |
        (backgrounds ? 2 : 0) |
        (sprites     ? 4 : 0) |
        (triangles   ? 8 : 0);
    u8 data[] = {
        0x03,
        0x00,
        0x00,
        mode_bits,
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_write_flag(CommandListRecorder * recorder, volatile u32 * flag, u32 value, bool interrupt) {
    usize address = (usize) flag;
    u8 data[] = {
        0x04,
        0x00,
        0x00,
        interrupt ? 0x01 : 0x00,
        COMMAND_ENCODED_U32(address),
        COMMAND_ENCODED_U32(value)
    };
    return PUSH_COMMAND;
}

/*
configure_texture < width > <height > <texture> <pixel_layout> <image_layout>    ..
[         00 05 ] [ XX XX ] [ YY YY ] [    ZZ ] [         UU ] [         VV ] [ 00 00 00 ]
*/

inline static bool gpu_command_configure_texture(CommandListRecorder * recorder, u8 texture, PixelDataLayout pixel_layout, ImageDataLayout image_layout, u16 width, u16 height) {
    u8 data[] = {
        0x05,
        0x00,
        COMMAND_ENCODED_U16(width),
        COMMAND_ENCODED_U16(height),
        texture,
        (u8) pixel_layout,
        (u8) image_layout,
        0x00,
        0x00,
        0x00,
    };
    return PUSH_COMMAND;
}

/*
upload_texture <texture> < src_image_layout> < src_address >
[      06 00 ] [    ZZ ] [              UU ] [ VV VV VV VV ]
*/

inline static bool gpu_command_upload_texture(CommandListRecorder * recorder, u8 texture, ImageDataLayout src_layout, volatile u8 * texture_data) {
    u8 data[] = {
        0x06,
        0x00,
        texture,
        (u8) src_layout,
        COMMAND_ENCODED_U32(texture_data)
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_configure_buffer(CommandListRecorder * recorder, u8 buffer, u32 length) {
    u8 data[] = {
        0x07,
        0x00,
        0x00,
        buffer,
        COMMAND_ENCODED_U32(length)
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_upload_buffer(CommandListRecorder * recorder, u8 buffer, volatile u8 * src_addr) {
    u8 data[] = {
        0x08,
        0x00,
        0x00,
        buffer,
        COMMAND_ENCODED_U32(src_addr)
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_direct_blit(CommandListRecorder * recorder, u8 src_tex, u8 dst_tex, u16 src_x, u16 src_y, u16 dst_x, u16 dst_y, u16 width, u16 height) {
    u8 data[] = {
        0x09,
        0x00,
        src_tex,
        dst_tex,
        COMMAND_ENCODED_U16(src_x),
        COMMAND_ENCODED_U16(src_y),
        COMMAND_ENCODED_U16(dst_x),
        COMMAND_ENCODED_U16(dst_y),
        COMMAND_ENCODED_U16(width),
        COMMAND_ENCODED_U16(height),
    };
    return PUSH_COMMAND;
}

inline static bool gpu_command_cutout_blit(CommandListRecorder * recorder, u8 src_tex, u8 dst_tex, u16 src_x, u16 src_y, u16 dst_x, u16 dst_y, u16 width, u16 height, PixelDataType src_alpha_data_type) {
    u8 data[] = {
        0x0A,
        0x00,
        src_tex,
        dst_tex,
        COMMAND_ENCODED_U16(src_x),
        COMMAND_ENCODED_U16(src_y),
        COMMAND_ENCODED_U16(dst_x),
        COMMAND_ENCODED_U16(dst_y),
        COMMAND_ENCODED_U16(width),
        COMMAND_ENCODED_U16(height),
        0x00,
        0x00,
        0x00,
        (u8) src_alpha_data_type
    };
    return PUSH_COMMAND;
}


