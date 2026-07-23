# NemaGFX integration

Rust bindings for ST's [NemaGFX](https://github.com/STMicroelectronics/x-cube-image-processing/tree/main/Middleware/NemaGFX) middleware (NeoChrom / NeoChromVG GPU). Headers and prebuilt libraries are **not committed** to this repo — run `./d fetch-nema-gfx` to pull them from `x-cube-image-processing` into `stm32-bindings-gen/nema_gfx/` before generating or building (CI does this automatically as part of "Load sources"). This mirrors how the STM32CubeWBA/STM32CubeWB sources are handled, and avoids redistributing ST/Think Silicon's licensed SDK in our git history.

**Version:** see `stm32-bindings-gen/nema_gfx/VERSION` after fetching (currently x-cube-image-processing v1.0.0, NemaGFX v1.4.17).

## Enabling bindings

```toml
stm32-bindings = { version = "...", default-features = false, features = ["nema_gfx"] }
```

This exposes the `nema_gfx` module (alias `nemagfx`) with bindgen-generated FFI. It does **not** link a GPU library by itself.

## Linking a prebuilt library

NemaGFX ships one prebuilt archive per CPU core variant. Enable the module **and exactly one** `lib_*` feature matching your hardware:

| Feature | ST lib folder | Typical parts |
|---------|---------------|---------------|
| `lib_nemagfx_cortex_m33_revc_float_abi_hard` | `cortex_m33_revC` | STM32U5x7/x9 (NeoChrom) |
| `lib_nemagfx_cortex_m33_nemapvg_float_abi_hard` | `cortex_m33_NemaPVG` | STM32U5F9/U5G9 (NeoChromVG) |
| `lib_nemagfx_cortex_m7_float_abi_hard` | `cortex_m7` | STM32H7R7/H7S7 |
| `lib_nemagfx_cortex_m55_float_abi_hard` | `cortex_m55` | STM32N6xx |

Example for STM32N657:

```toml
stm32-bindings = { default-features = false, features = [
    "nema_gfx",
    "lib_nemagfx_cortex_m55_float_abi_hard",
] }
```

All shipped variants use the GCC **hard-float** ABI (`libnemagfx-float-abi-hard.a`), renamed on fetch to avoid filename collisions across cores.

### Presets

| Feature | Equivalent lib feature | Typical parts |
|---------|------------------------|---------------|
| `neochrom-m33-revc` | `lib_nemagfx_cortex_m33_revc_float_abi_hard` | STM32U5x7/x9 |
| `neochrom-m33-nemapvg` | `lib_nemagfx_cortex_m33_nemapvg_float_abi_hard` | STM32U5F9/U5G9 |
| `neochrom-m7` | `lib_nemagfx_cortex_m7_float_abi_hard` | STM32H7R7/H7S7 |
| `neochrom-m55` | `lib_nemagfx_cortex_m55_float_abi_hard` | STM32N6xx |

Enable **exactly one** NemaGFX library (via a preset or a single `lib_nemagfx_*` feature). `build.rs` rejects linking multiple core variants at once.

## Platform HAL (required)

NemaGFX expects a platform HAL (`nema_sys_init`, `nema_reg_read`/`write`, `nema_buffer_*`, `nema_wait_irq`, …). ST provides templates in the upstream `Middleware/NemaGFX/templates/` directory.

The companion crate **`nema-gfx-hal`** (in this repository) ships an adapted bare-metal HAL plus an optional GPU2D stub for link tests. On hardware, initialize GPU2D clocks, call `HAL_GPU2D_Init` from STM32Cube, disable the `stub` feature, and link the real Cube GPU2D driver.

### Smoke example

```bash
cargo run --release --bin stm32-bindings-gen -- --module nema_gfx
cd examples/nema-gfx-smoke && cargo build
```

`examples/nema-gfx-smoke` runs `nema_init`, binds a 64×64 framebuffer, issues `nema_clear` via a command list, and links against the M55 NemaGFX archive. With the default GPU2D stub it validates the API/link path in CI; flash to an STM32N6 board only after replacing the stub with Cube HAL init.

Refer to ST's STM32N6570-DK examples under `x-cube-image-processing/STM32N6570-DK/` for full clock, memory, and display setup.

## Fetching / updating vendored files

```bash
./d fetch-nema-gfx
```

Fetches headers and prebuilt libraries into `stm32-bindings-gen/nema_gfx/` (gitignored — not committed). Required before `cargo run --release --bin stm32-bindings-gen` or any build/check that touches the `nema_gfx` feature. Edit `XCUBE_IP_REV` in `d` when bumping the upstream package.

## License

NemaGFX is licensed under Think Silicon / ST terms. See `stm32-bindings-gen/nema_gfx/LICENSE.md` after fetching. Redistribution conditions differ from MIT/Apache-2.0 and restrict redistributing the SDK itself — that's why it's fetched at build time rather than committed; review the license before publishing applications or crates that bundle the `.a` files.
