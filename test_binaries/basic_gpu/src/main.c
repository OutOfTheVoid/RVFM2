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

void main() {
    MemBuffer command_buffer_membuff;
    command_buffer_membuff.buffer = (volatile u8 *) gpu_command_buffer_memory;
    command_buffer_membuff.length = sizeof(gpu_command_buffer_memory);

    CommandListRecorder recorder;
    init_commandlist_recorder(&recorder, command_buffer_membuff);

    const u8 clear_color[] = {0x00, 0xFF, 0xFF, 0xFF};

    volatile u32 present_completion = 0;
    volatile u32 submit_completion = 0;

    gpu_command_set_video_mode(&recorder, VideoResolution256x192, false, false, false);
    gpu_command_set_constant_sampler_rgba_unorm8(&recorder, 0, clear_color);
    gpu_command_configure_texture(&recorder, 0, PixelDataLayoutD8x4, ImageDataLayoutContiguous, 256, 192);
    gpu_command_clear_texture(&recorder, 0, 0);
    gpu_command_present_texture(&recorder, 0, &present_completion, true);

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
