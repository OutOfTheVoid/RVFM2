#pragma once

#include "../command_list.h"
#include "../completion.h"
#include "../mem_buffer.h"
#include "commands.h"

volatile u32 * const SPU_RUN_MODE         = (((volatile u32 *) 0x80040000));
volatile u32 * const SPU_SAMPLE_COUNTER   = (((volatile u32 *) 0x80040004));
volatile u32 * const SPU_SAMPLE_RATE      = (((volatile u32 *) 0x80040008));
volatile u32 * const SPU_SUBMISSION_ERROR = (((volatile u32 *) 0x8004000C));

typedef enum {
    SampleRate16000 = 0,
    SampleRate32000 = 1,
    SampleRate41000 = 2,
    SampleRate48000 = 3,
} SampleRate;

inline static void spu_set_sample_rate(SampleRate rate) {
    *SPU_SAMPLE_RATE = rate;
}

inline static void spu_run() {
    *SPU_RUN_MODE = 1;
}

inline static bool spu_running() {
    return *SPU_RUN_MODE != 0;
}

typedef struct {
    u32 completion;
    u32 running;
    u32 sample_index;
    u32 loop_count;
} SamplerStatus;


