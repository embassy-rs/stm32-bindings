# nema-gfx-hal

Bare-metal platform HAL for ST NemaGFX, adapted from the [STM32N6570-DK](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/STM32N6570-DK/Common/Src/nema_hal_baremetal.c) reference.

By default the `stub` feature links a minimal `HAL_GPU2D_*` implementation and bump allocator so examples and CI can link without STM32Cube.

For hardware:

1. Enable GPU2D clocks and call `HAL_GPU2D_Init(&hgpu2d)` from STM32Cube.
2. Disable `stub` on this crate and link the Cube GPU2D HAL object file.
3. Replace `bump_alloc.c` with a proper heap if needed.
