#include "dbg.h"
#include "intrin.h"
#include "input.h"

void main() {
    if (get_input(InputIdUp)) {
        debug_print("up: DOWN");
    }

    while(1) {}
}
