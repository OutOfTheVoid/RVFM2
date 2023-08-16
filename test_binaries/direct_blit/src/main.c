#include "dbg.h"
#include "intrin.h"
#include "interrupt.h"
#include "gpu/gpu.h"

INTERRUPT_HANDLER void gpu_interrupt();

#define NULL 0

void (* INTERRUPT_HANDLER interrupt_table[])() = {
    NULL,
    NULL,
    NULL,
    NULL,

    NULL,
    NULL,
    NULL,
    NULL,

    NULL,
    NULL,
    NULL,
    gpu_interrupt,
};

u8 gpu_command_buffer_memory[0x100];

u8 texture[4 * 256 * 192];

void main() {
    MemBuffer command_buffer_membuff;
    command_buffer_membuff.buffer = (volatile u8 *) gpu_command_buffer_memory;
    command_buffer_membuff.length = sizeof(gpu_command_buffer_memory);

    CommandListRecorder recorder;
    init_commandlist_recorder(&recorder, command_buffer_membuff);

    volatile u32 present_completion = 0;
    volatile u32 submit_completion = 0;

    u8 colors[4 * 6] = {
        0xFF, 0x00, 0x00, 0xFF,
        0xFF, 0x88, 0x00, 0xFF,
        0xFF, 0xFF, 0x00, 0xFF,
        0x00, 0xFF, 0x00, 0xFF,
        0x22, 0x22, 0xFF, 0xFF,
        0xFF, 0x22, 0xFF, 0xFF,
    };

    for (int y = 0; y < 32; y ++) {
        for (int x = 0; x < 32; x ++) {
            u32 pixel_offset = (x + y * 32) << 2;
            u32 color_offset = ((x >> 2) % 6) << 2;
            texture[pixel_offset | 0] = colors[color_offset | 0];
            texture[pixel_offset | 1] = colors[color_offset | 1];
            texture[pixel_offset | 2] = colors[color_offset | 2];
            texture[pixel_offset | 3] = colors[color_offset | 3];
        }
    }

    gpu_command_configure_texture(&recorder, 0, PixelDataLayoutD8x4, ImageDataLayoutContiguous, 32, 32);
    gpu_command_upload_texture(&recorder, 0, ImageDataLayoutContiguous, (volatile u8 *) texture);

    u8 clear_color[] = {0, 0, 0, 0};
    gpu_command_set_constant_sampler_rgba_unorm8(&recorder, 0, clear_color);
    gpu_command_configure_texture(&recorder, 1, PixelDataLayoutD8x4, ImageDataLayoutContiguous, 256, 192);
    gpu_command_clear_texture(&recorder, 1, 0);

    gpu_command_set_video_mode(&recorder, VideoResolution256x192, false, false, false);
    gpu_command_direct_blit(&recorder, 0, 1, 0, 0, 16, 16, 32, 32);
    gpu_command_direct_blit(&recorder, 0, 1, 0, 0, 50, 80, 32, 32);
    gpu_command_present_texture(&recorder, 1, &present_completion, true);

    CommandList command_list = finish_commandlist_recorder(&recorder);
    submit_commandlist(command_list, &submit_completion);

    while (!submit_completion) {
    };
    while (!present_completion) {
    };

    debug_print("finished");
}

INTERRUPT_HANDLER void gpu_interrupt() {
    debug_print("gpu_interrupt()");
    Interrupt interrupt;
    if (get_pending_interrupt(Hart0, &interrupt)) {
        clear_interrupt(interrupt);
    }
}
