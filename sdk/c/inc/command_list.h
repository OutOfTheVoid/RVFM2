#pragma once

#include "types.h"
#include "mem_buffer.h"

typedef struct {
    usize          length;
    volatile u32 * completion_flag;
} CommandListHeader;

typedef struct {
    usize                        capacity;
    usize                        length;
    volatile CommandListHeader * header;
    volatile u8                * data;
} CommandListRecorder;

typedef volatile CommandListHeader * CommandList;

static inline void init_commandlist_recorder(CommandListRecorder * recorder, MemBuffer buffer) {
    volatile u32 * capacity_ptr = (volatile u32 *) buffer.buffer;
    *capacity_ptr = recorder->capacity = buffer.length - 12;
    recorder->header = (volatile CommandListHeader *) &buffer.buffer[4];
    recorder->data = &buffer.buffer[12];
    recorder->length = 0;
}

static inline CommandList finish_commandlist_recorder(CommandListRecorder * recorder) {
    recorder->header->length = recorder->length;
    return recorder->header;
}

static inline bool poll_commandlist_submission(CommandList command_list) {
    return *command_list->completion_flag != 0;
}

static inline MemBuffer reset_commandlist(CommandList command_list) {
    MemBuffer mem_buffer;
    volatile u8 * buffer = &((volatile u8 *) command_list)[-4];
    usize * capacity_ptr = (u32 *) buffer;
    usize capacity = *capacity_ptr;
    mem_buffer.buffer = buffer;
    mem_buffer.length = capacity;
    return mem_buffer;
}

static inline usize remaining_space(CommandListRecorder * recorder) {
    return recorder->capacity - recorder->length;
}

static inline bool push_command(CommandListRecorder * recorder, u8 * command, usize length) {
    if (remaining_space(recorder) < length) {
        return false;
    }
    volatile u8 * data = &recorder->data[recorder->length];
    for (usize i = 0; i < length; i ++) {
        data[i] = command[i];
    }
    recorder->length += length;
    return true;
}

