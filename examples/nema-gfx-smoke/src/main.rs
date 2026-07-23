//! Minimal NemaGFX smoke test: `nema_init` + `nema_clear` on a tiny framebuffer.
//!
//! Uses the in-tree GPU2D HAL stub — verifies linking and the basic command-list
//! flow. Flash to an STM32N6 board only after replacing the stub with STM32Cube
//! GPU2D initialization.

#![no_std]
#![no_main]

use core::ptr;

use cortex_m_rt::entry;
use stm32_bindings::nema_gfx::{
    nema_bind_dst_tex, nema_cl_bind, nema_cl_create, nema_cl_submit, nema_cl_wait, nema_clear,
    nema_init, NEMA_RGBA8888,
};

const FB_WIDTH: u32 = 64;
const FB_HEIGHT: u32 = 64;
const FB_STRIDE: i32 = -1;

static mut FRAMEBUFFER: [u32; (FB_WIDTH * FB_HEIGHT) as usize] =
    [0; (FB_WIDTH * FB_HEIGHT) as usize];

#[entry]
fn main() -> ! {
    if nema_gfx_hal::gpu2d_init_stub().is_err() {
        panic!("GPU2D stub init failed");
    }

    let status = unsafe { nema_init() };
    if status != 0 {
        panic!("nema_init failed");
    }

    let fb_addr = ptr::addr_of!(FRAMEBUFFER) as usize;
    unsafe {
        nema_bind_dst_tex(fb_addr, FB_WIDTH, FB_HEIGHT, NEMA_RGBA8888, FB_STRIDE);

        let mut cl = nema_cl_create();
        nema_cl_bind(&mut cl);
        nema_clear(0xFF0000FF);
        nema_cl_submit(&mut cl);
        let wait_status = nema_cl_wait(&mut cl);
        if wait_status != 0 {
            panic!("nema_cl_wait failed");
        }
    }

    // Stub GPU: framebuffer may stay zero, but the API path linked and ran.
    loop {
        cortex_m::asm::nop();
    }
}

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {
        cortex_m::asm::nop();
    }
}
