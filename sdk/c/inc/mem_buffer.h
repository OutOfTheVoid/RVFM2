#pragma once

#include "types.h"

typedef struct {
    volatile u8 * buffer;
    usize         length;
} MemBuffer;
