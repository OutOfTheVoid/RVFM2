#pragma once

#include "types.h"

typedef enum {
    InputIdUp     = 0,
    InputIdDown   = 1,
    InputIdLeft   = 2,
    InputIdRight  = 3,
    InputIdA      = 4,
    InputIdB      = 5,
    InputIdX      = 6,
    InputIdY      = 7,
    InputIdStart  = 8,
    InputIdSelect = 9,
} InputId;

inline static bool get_input(InputId input) {
    u32 address = 0x80050000 + (((u32) input) << 2);
    u32 value = *((volatile u32 *) address);
    return value != 0;
}
