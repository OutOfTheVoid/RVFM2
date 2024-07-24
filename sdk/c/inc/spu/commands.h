#pragma once

#include "../command_list.h"

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

const u32 SPU_COMMANDLIST_SUBMISSION_PORT_BASE = 0x80040010;

typedef enum {
    WaveformSquare   = 0,
    WaveformTriangle = 1,
    WaveformSin      = 2,
    WaveformSuperSaw = 3,
    WaveformNoise    = 4,
} Waveform;

typedef enum {
    FilterModeAllPass = 0,
    LowPass6          = 1,
    LowPass12         = 2,
    LowPass18         = 3,
    LowPass24         = 4,

    HighPass6         = 5,
    HighPass12        = 6,
    HighPass18        = 7,
    HighPass24        = 8,

    BandPass6         = 9,
    BandPass12        = 10,
    BandPass18        = 11,
    BandPass24        = 12,
} FilterMode;

typedef enum {
    PitchModeConstant            = 0,
    PitchModePortamentoQuadratic = 1,
} PitchMode;

typedef enum {
    ChannelCountMono = 0,
    ChannelCountStereo = 1,
} ChannelCount;

typedef enum {
    SpuQueue0 = 0,
    SpuQueue1 = 1,
    SpuQueue2 = 2,
    SpuQueue3 = 3,
} SpuQueue;

inline static bool spu_command_reset_sample_counter(CommandListRecorder * recorder, u32 reset_value) {
    u8 data[] = {
        0x00,
        COMMAND_ENCODED_U32(reset_value)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_wait_for_sample_counter(CommandListRecorder * recorder, u32 sample_count) {
    u8 data[] = {
        0x01,
        COMMAND_ENCODED_U32(sample_count)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_write_flag(CommandListRecorder * recorder, bool interrupt, volatile u32 * flag, u32 value) {
    u8 data[] = {
        0x02,
        interrupt ? 0x01 : 0x00,
        COMMAND_ENCODED_U32(flag),
        COMMAND_ENCODED_U32(value),
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_stop(CommandListRecorder * recorder) {
    u8 data[] = {
        0x04
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_mute(CommandListRecorder * recorder, u8 envelope) {
    u8 data[] = {
        0x05,
        envelope,
        0x00
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_off(CommandListRecorder * recorder, u8 envelope) {
    u8 data[] = {
        0x05,
        envelope,
        0x01
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_on(CommandListRecorder * recorder, u8 envelope) {
    u8 data[] = {
        0x05,
        envelope,
        0x02
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_set_attack(CommandListRecorder * recorder, u8 envelope, u32 attack) {
    u8 data[] = {
        0x06,
        envelope,
        0x00,
        COMMAND_ENCODED_U32(attack)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_set_decay(CommandListRecorder * recorder, u8 envelope, u32 decay) {
    u8 data[] = {
        0x06,
        envelope,
        0x01,
        COMMAND_ENCODED_U32(decay)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_set_release(CommandListRecorder * recorder, u8 envelope, u32 release) {
    u8 data[] = {
        0x06,
        envelope,
        0x02,
        COMMAND_ENCODED_U32(release)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_envelope_set_sustain(CommandListRecorder * recorder, u8 envelope, i16 sustain) {
    u8 data[] = {
        0x06,
        envelope,
        0x03,
        COMMAND_ENCODED_U16((u16) sustain)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_oscillator_reset(CommandListRecorder * recorder, u8 oscillator) {
     u8 data[] = {
        0x07,
        oscillator,
        0x00,
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_oscillator_set_parameter(CommandListRecorder * recorder, u8 oscillator, u8 parameter, i16 value) {
     u8 data[] = {
        0x08,
        oscillator,
        0x01,
        parameter,
        COMMAND_ENCODED_U16(value)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_oscillator_set_phase(CommandListRecorder * recorder, u8 oscillator, u8 index, u16 value) {
     u8 data[] = {
        0x08,
        oscillator,
        0x01,
        index,
        COMMAND_ENCODED_U16(value)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_oscillator_set_waveform(CommandListRecorder * recorder, u8 oscillator, Waveform waveform) {
     u8 data[] = {
        0x08,
        oscillator,
        0x02,
        waveform
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_filter_reset(CommandListRecorder * recorder, u8 filter) {
    u8 data[] = {
        0x09,
        filter,
        0x00
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_filter_set_mode(CommandListRecorder * recorder, u8 filter, FilterMode mode) {
    u8 data[] = {
        0x0A,
        filter,
        0x00,
        mode
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_filter_set_resonance(CommandListRecorder * recorder, u8 filter, u16 resonance) {
    u8 data[] = {
        0x0A,
        filter,
        0x01,
        COMMAND_ENCODED_U16(resonance)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_pitch_finish(CommandListRecorder * recorder, u8 pitch) {
    u8 data[] = {
        0x0B,
        pitch,
        0x00
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_pitch_set_target(CommandListRecorder * recorder, u8 pitch, u16 target) {
    u8 data[] = {
        0x0C,
        pitch,
        0x00,
        COMMAND_ENCODED_U16(target)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_pitch_set_speed(CommandListRecorder * recorder, u8 pitch, u16 speed) {
    u8 data[] = {
        0x0C,
        pitch,
        0x01,
        COMMAND_ENCODED_U16(speed)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_pitch_set_mode(CommandListRecorder * recorder, u8 pitch, PitchMode mode) {
    u8 data[] = {
        0x0C,
        pitch,
        0x02,
        mode
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_set_mix(CommandListRecorder * recorder, u8 voice, bool right, i16 mix) {
    u16 channel = ((u16) voice) << 1;
    if (right) {
        channel |= 1;
    }
    u8 data[] = {
        0x0D,
        channel,
        COMMAND_ENCODED_U16((u16) mix)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_note_on(CommandListRecorder * recorder, u8 voice, u16 frequency) {
    u8 data[] = {
        0x0E,
        voice,
        COMMAND_ENCODED_U16(frequency)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_relative_wait(CommandListRecorder * recorder, u32 count) {
    u8 data[] = {
        0x0F,
        COMMAND_ENCODED_U32(count)
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_setup(CommandListRecorder * recorder, u8 sampler, ChannelCount channel_count, u32 sample_count, u32 start_address) {
    u8 data[] = {
        0x10,
        0x00,
        sampler,
        (u8) channel_count,
        COMMAND_ENCODED_U32(sample_count),
        COMMAND_ENCODED_U32(start_address),
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_set_loopmode_infinite(CommandListRecorder * recorder, u8 sampler) {
    u8 data[] = {
        0x10,
        0x01,
        sampler,
        COMMAND_ENCODED_U32(0xFFFFFFFF),
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_set_loopmode_finite(CommandListRecorder * recorder, u8 sampler, u32 loop_count) {
    u8 data[] = {
        0x10,
        0x01,
        sampler,
        COMMAND_ENCODED_U32(loop_count),
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_start(CommandListRecorder * recorder, u8 sampler) {
    u8 data[] = {
        0x11,
        0x00,
        sampler,
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_continue(CommandListRecorder * recorder, u8 sampler) {
    u8 data[] = {
        0x11,
        0x01,
        sampler,
    };
    return PUSH_COMMAND;
}

inline static bool spu_command_sampler_pause(CommandListRecorder * recorder, u8 sampler) {
    u8 data[] = {
        0x11,
        0x02,
        sampler,
    };
    return PUSH_COMMAND;
}

static inline void spu_submit_commandlist(SpuQueue queue, CommandList command_list, volatile u32 * completion_flag) {
    *completion_flag = 0;
    command_list->completion_flag = completion_flag;
    *(volatile u32 *)(SPU_COMMANDLIST_SUBMISSION_PORT_BASE + (((u32) queue) << 2)) = (usize) command_list;
}

