#pragma once

#include <types.h>
#include <string.h>

static volatile const u8** const debug_message_ptr   = (volatile const u8** const) 0x80000000;
static volatile       u32* const debug_length        = (volatile       u32* const) 0x80000004;
static volatile       u32* const debug_status        = (volatile       u32* const) 0x80000008;
static volatile       u32* const debug_print_trigger = (volatile       u32* const) 0x8000000C;
static volatile       u32* const debug_flush_trigger = (volatile       u32* const) 0x80000010;

typedef u32 debug_result_t;

static inline debug_result_t debug_write_message(const char * chars, u32 length) {
    *debug_message_ptr = (const u8 *) chars;
    *debug_length = length;
    *debug_print_trigger = 1;
    return *debug_status;
}

static inline debug_result_t debug_write_string(const char * string) {
    u32 length = strlen(string);
    return debug_write_message(string, length);
}

static inline debug_result_t debug_flush() {
    *debug_flush_trigger = 1;
    return *debug_status;
}

static inline debug_result_t debug_print(const char * string) {
    debug_write_string(string);
    debug_flush();
}
