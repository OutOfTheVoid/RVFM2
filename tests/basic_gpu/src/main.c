#include "dbg.h"
#include "intrin.h"
#include "interrupt.h"
#include "gpu/gpu.h"
#include "command_list.h"

INTERRUPT_HANDLER void gpu_interrupt();

#define NULL 0

void INTERRUPT_HANDLER dummy_interrupt() {
}

DECLARE_VECTORED_INTERRUPT_TABLE(interrupt_table, dummy_interrupt, dummy_interrupt, gpu_interrupt);

u8 gpu_command_buffer_memory[0x100];

void main() {
    debug_print("begin\n");
    debug_flush();
    
    hart_set_vectored_interrupt_table(interrupt_table);

    set_interrupt_target(PresentInterrupt, Hart0);
    enable_interrupt(PresentInterrupt);
    set_interrupt_target(GpuInterrupt, Hart0);
    enable_interrupt(GpuInterrupt);

    hart_enable_external_interrupt();
    hart_enable_interrupts();

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
    gpu_command_present_texture(&recorder, 0, &present_completion, false);

    CommandList command_list = finish_commandlist_recorder(&recorder);
    gpu_submit_commandlist(command_list, &submit_completion);

    while (!poll_commandlist_submission(command_list)) {
    };

    debug_print("submission finished\n");

    while (present_completion == 0) {
    };

    debug_print("finished\n");

    while (1) { wfi(); }
}

INTERRUPT_HANDLER void gpu_interrupt() {
    debug_print("gpu_interrupt()\n");
    Interrupt interrupt;
    if (get_pending_interrupt(Hart0, &interrupt)) {
        clear_interrupt(interrupt);
    }
}
