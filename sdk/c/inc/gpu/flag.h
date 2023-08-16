#pragma once

#include "../types.h"

inline static bool gpu_poll_flag(volatile u32 * flag, u32 value) {
    return *flag == value;
}

inline static void gpu_spinwait_flag(volatile u32 * flag, u32 value) {
    while (!gpu_poll_flag(flag, value)) {}
}


