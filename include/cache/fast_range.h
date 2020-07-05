#ifndef ZORRO_FAST_RANGE_H
#define ZORRO_FAST_RANGE_H

#include <stdint.h>
#include <stdlib.h>

size_t
fast_range_32(uint32_t x, size_t range);

size_t
fast_range_64(uint64_t x, size_t range);

#endif
