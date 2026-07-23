//! Platform HAL for [NemaGFX](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/Middleware/NemaGFX).
//!
//! This crate compiles ST's bare-metal `nema_hal` glue plus, by default, a GPU2D
//! HAL stub so you can link against `stm32-bindings` in CI or early bring-up.
//!
//! # Hardware
//!
//! Before calling [`stm32_bindings::nema_gfx::nema_init`], initialize the GPU2D
//! peripheral clocks and call `HAL_GPU2D_Init(&mut hgpu2d)` from STM32Cube HAL.
//! Then disable the `stub` feature and link your Cube HAL GPU2D driver instead of
//! [`gpu2d_hal_stub.c`](c/gpu2d_hal_stub.c).
//!
//! # Example
//!
//! See `examples/nema-gfx-smoke` for a minimal `nema_init` + `nema_clear` flow.

#![no_std]

/// Initialize the GPU2D handle using the stub HAL.
///
/// On hardware, replace this with STM32Cube `HAL_GPU2D_Init` after enabling clocks.
#[cfg(feature = "stub")]
pub fn gpu2d_init_stub() -> Result<(), ()> {
    unsafe extern "C" {
        fn nema_gfx_hal_gpu2d_init() -> i32;
    }

    unsafe {
        if nema_gfx_hal_gpu2d_init() == 0 {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[cfg(not(feature = "stub"))]
pub fn gpu2d_init_stub() -> Result<(), ()> {
    Err(())
}
