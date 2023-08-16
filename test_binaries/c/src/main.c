#include "dbg.h"
#include "intrin.h"

void main() {
    debug_print("Hello, world!");
    while (1) {
        wfi();
    }
}
