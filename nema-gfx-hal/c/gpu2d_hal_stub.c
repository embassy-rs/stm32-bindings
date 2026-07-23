/**
 * Stub GPU2D HAL for link tests and bring-up without STM32Cube.
 *
 * On hardware, provide a real HAL_GPU2D implementation and disable the
 * `stub` feature on `nema-gfx-hal`.
 */
#include "hal_gpu2d_shim.h"

GPU2D_HandleTypeDef hgpu2d = {0};

HAL_StatusTypeDef HAL_GPU2D_Init(GPU2D_HandleTypeDef *handle)
{
    if (handle == 0) {
        return HAL_ERROR;
    }
    if (handle->Instance == 0) {
        handle->Instance = (void *)0x50000000UL;
    }
    return HAL_OK;
}

uint32_t HAL_GPU2D_ReadRegister(GPU2D_HandleTypeDef *handle, uint32_t reg)
{
    UNUSED(handle);
    if (reg == GPU2D_BREAKPOINT) {
        return 1U;
    }
    return 0U;
}

void HAL_GPU2D_WriteRegister(GPU2D_HandleTypeDef *handle, uint32_t reg, uint32_t value)
{
    UNUSED(handle);
    UNUSED(reg);
    UNUSED(value);
}

void HAL_GPU2D_PollCompletion(GPU2D_HandleTypeDef *handle)
{
    /* Stub never completes anything; nothing to poll. */
    UNUSED(handle);
}

int nema_gfx_hal_gpu2d_init(void)
{
    return HAL_GPU2D_Init(&hgpu2d) == HAL_OK ? 0 : -1;
}

#if defined(USE_HAL_GPU2D_REGISTER_CALLBACKS) && (USE_HAL_GPU2D_REGISTER_CALLBACKS == 1)
HAL_StatusTypeDef HAL_GPU2D_RegisterCommandListCpltCallback(
    GPU2D_HandleTypeDef *handle, GPU2D_CommandListCpltCallbackTypeDef cb)
{
    UNUSED(handle);
    UNUSED(cb);
    return HAL_OK;
}
#endif
