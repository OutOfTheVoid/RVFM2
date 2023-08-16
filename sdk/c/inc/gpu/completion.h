#pragma once

#include "../types.h"
#include "../interrupt.h"
#include "../intrin.h"

void spinwait_completion(volatile u32 * completion_flag) {
    while (! *completion_flag) {}
}

bool poll_completion(volatile u32 * completion) {
    
}
