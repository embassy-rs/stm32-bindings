/**
 * Minimal bump allocator for NemaGFX HAL bring-up examples.
 *
 * Suitable for smoke tests only — `free()` is a no-op. Replace with a proper
 * heap when running a real application.
 */
#include <stddef.h>
#include <stdint.h>

#ifndef NEMA_BUMP_ALLOC_SIZE
#define NEMA_BUMP_ALLOC_SIZE (128 * 1024)
#endif

static uint8_t bump_pool[NEMA_BUMP_ALLOC_SIZE];
static size_t bump_offset;

void *malloc(size_t size)
{
    if (size == 0) {
        return 0;
    }

    size_t aligned = (size + 7U) & ~((size_t)7U);
    if (bump_offset + aligned > NEMA_BUMP_ALLOC_SIZE) {
        return 0;
    }

    void *ptr = &bump_pool[bump_offset];
    bump_offset += aligned;
    return ptr;
}

void free(void *ptr)
{
    (void)ptr;
}
