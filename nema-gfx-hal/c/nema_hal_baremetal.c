/**
 * NemaGFX bare-metal platform HAL (adapted from ST STM32N6570-DK example).
 *
 * Uses `hal_gpu2d_shim.h` instead of STM32Cube so the crate can link in CI.
 * On hardware, pair with a real GPU2D HAL implementation.
 */
#include "nema_core.h"
#include "nema_sys_defs.h"
#include "hal_gpu2d_shim.h"

#include <stdlib.h>

#define RING_SIZE 1024

static nema_ringbuffer_t ring_buffer_str = {0};
static volatile int last_cl_id = -1;

#if defined(USE_HAL_GPU2D_REGISTER_CALLBACKS) && (USE_HAL_GPU2D_REGISTER_CALLBACKS == 1)
static void GPU2D_CommandListCpltCallback(GPU2D_HandleTypeDef *handle, uint32_t cmd_list_id)
{
    UNUSED(handle);
    last_cl_id = (int)cmd_list_id;
}
#else
void HAL_GPU2D_CommandListCpltCallback(GPU2D_HandleTypeDef *handle, uint32_t cmd_list_id)
{
    UNUSED(handle);
    last_cl_id = (int)cmd_list_id;
}
#endif

int32_t nema_sys_init(void)
{
#if defined(USE_HAL_GPU2D_REGISTER_CALLBACKS) && (USE_HAL_GPU2D_REGISTER_CALLBACKS == 1)
    HAL_GPU2D_RegisterCommandListCpltCallback(&hgpu2d, GPU2D_CommandListCpltCallback);
#endif

    ring_buffer_str.bo = nema_buffer_create(RING_SIZE);
    (void)nema_buffer_map(&ring_buffer_str.bo);

    int ret = nema_rb_init(&ring_buffer_str, 1);
    if (ret < 0) {
        return ret;
    }

    last_cl_id = 0;
    return 0;
}

int nema_wait_irq(void)
{
    HAL_GPU2D_PollCompletion(&hgpu2d);
    return 0;
}

int nema_wait_irq_cl(int cl_id)
{
    while (last_cl_id < cl_id) {
        (void)nema_wait_irq();
    }
    return 0;
}

int nema_wait_irq_brk(int brk_id)
{
    UNUSED(brk_id);
    while (nema_reg_read(GPU2D_BREAKPOINT) == 0U) {
        (void)nema_wait_irq();
    }
    return 0;
}

uint32_t nema_reg_read(uint32_t reg)
{
    return HAL_GPU2D_ReadRegister(&hgpu2d, reg);
}

void nema_reg_write(uint32_t reg, uint32_t value)
{
    HAL_GPU2D_WriteRegister(&hgpu2d, reg, value);
}

nema_buffer_t nema_buffer_create(int size)
{
    nema_buffer_t bo;

    bo.base_virt = malloc((size_t)size);
    bo.base_phys = (uintptr_t)bo.base_virt;
    bo.size = size;
    bo.fd = 0;

    return bo;
}

nema_buffer_t nema_buffer_create_pool(int pool, int size)
{
    UNUSED(pool);
    return nema_buffer_create(size);
}

void *nema_buffer_map(nema_buffer_t *bo)
{
    return bo->base_virt;
}

void nema_buffer_unmap(nema_buffer_t *bo)
{
    UNUSED(bo);
}

void nema_buffer_destroy(nema_buffer_t *bo)
{
    free(bo->base_virt);
    bo->base_virt = 0;
    bo->base_phys = 0;
}

uintptr_t nema_buffer_phys(nema_buffer_t *bo)
{
    return bo->base_phys;
}

void nema_buffer_flush(nema_buffer_t *bo)
{
#if defined(NEMA_CACHED_MEMORY)
    extern void SCB_CleanInvalidateDCache_by_Addr(volatile void *addr, int32_t dsize);
    SCB_CleanInvalidateDCache_by_Addr((volatile void *)bo->base_virt, bo->size);
#else
    UNUSED(bo);
#endif
}

void nema_host_free(void *ptr)
{
    if (ptr) {
        free(ptr);
    }
}

void *nema_host_malloc(size_t size)
{
    return malloc(size);
}

int nema_mutex_lock(int mutex_id)
{
    UNUSED(mutex_id);
    return 0;
}

int nema_mutex_unlock(int mutex_id)
{
    UNUSED(mutex_id);
    return 0;
}
