#pragma once

#include "intrin.h"

typedef enum {
    Hart0 = 0,
    Hart1 = 1,
    Hart2 = 2,
    Hart3 = 3,
} Hart;

inline static Hart this_hart() {
    return (Hart) hart_id();
}
