#pragma once

#include "types.h"
#include "hart.h"

inline static volatile u32 * const hart_start_triggers = (volatile u32 * const) 0x80020000;

inline static void start_hart(Hart hart, u32 start_address) {
    hart_start_triggers[(u32) hart] = start_address;
}
