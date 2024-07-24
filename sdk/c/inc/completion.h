#pragma once

#include "types.h"
#include "interrupt.h"
#include "intrin.h"

void spinwait_completion(volatile u32 * completion_flag, u32 value) {
    while (*completion_flag != value) {}
}

bool poll_completion(volatile u32 * completion, u32 value) {
    return *completion == value;
}
