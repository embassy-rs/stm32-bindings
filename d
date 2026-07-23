#!/usr/bin/env bash

set -e
cd $(dirname $0)

CMD=$1
WBA_REV=v1.9.0
WB_REV=v1.24.0
XCUBE_IP_REV=v1.0.0
shift

fetch_nema_gfx() {
    local tmp
    tmp=$(mktemp -d)
    git clone --depth 1 https://github.com/STMicroelectronics/x-cube-image-processing.git "$tmp"
    cd "$tmp"
    git fetch origin "$XCUBE_IP_REV" 2>/dev/null || true
    git checkout FETCH_HEAD 2>/dev/null || git checkout "$XCUBE_IP_REV" 2>/dev/null || git checkout main
    cd - >/dev/null

    rm -rf ./stm32-bindings-gen/nema_gfx/include
    mkdir -p ./stm32-bindings-gen/nema_gfx/include ./stm32-bindings-gen/nema_gfx/lib
    cp -R "$tmp/Middleware/NemaGFX/include/." ./stm32-bindings-gen/nema_gfx/include/
    cp "$tmp/Middleware/NemaGFX/LICENSE.md" ./stm32-bindings-gen/nema_gfx/LICENSE.md
    cat > ./stm32-bindings-gen/nema_gfx/VERSION <<EOF
x-cube-image-processing ${XCUBE_IP_REV}
NemaGFX middleware v1.4.17
EOF

    local src dst
    for pair in \
        "cortex_m33_revC:cortex_m33_revc" \
        "cortex_m33_NemaPVG:cortex_m33_nemapvg" \
        "cortex_m7:cortex_m7" \
        "cortex_m55:cortex_m55"
    do
        src="${pair%%:*}"
        dst="${pair##*:}"
        cp "$tmp/Middleware/NemaGFX/lib/core/${src}/gcc/libnemagfx-float-abi-hard.a" \
            "./stm32-bindings-gen/nema_gfx/lib/libnemagfx_${dst}_float_abi_hard.a"
    done

    rm -rf "$tmp"
    echo "NemaGFX headers and libraries updated under stm32-bindings-gen/nema_gfx/"
}

case "$CMD" in
    gen)
        cargo run --release --bin stm32-bindings-gen
    ;;
    download-all)
        rm -rf ./sources
        git clone https://github.com/STMicroelectronics/STM32CubeWBA.git ./sources/STM32CubeWBA/ --depth 1
        git clone https://github.com/STMicroelectronics/STM32CubeWB.git ./sources/STM32CubeWB/ --depth 1
        cd ./sources/STM32CubeWBA/
        git fetch origin $WBA_REV
        git checkout FETCH_HEAD
        git submodule update --init --recursive
        cd ../..
        cd ./sources/STM32CubeWB/
        git fetch origin $WB_REV
        git checkout FETCH_HEAD
        git submodule update --init --recursive
        cd ../..
    ;;
    build-thread)
        mkdir -p build
        rm -rf build/thread 
        mkdir -p build/thread
        
        cp -r stm32-bindings-gen/thread build
        cd build/thread

        cmake -B build -G Ninja -DCMAKE_C_COMPILER=arm-none-eabi-gcc -DCMAKE_BUILD_TYPE=Release "-DCMAKE_TOOLCHAIN_FILE=arm-gcc-toolchain.cmake"
        cmake --build build

        cd ../..
    ;;
    fetch-nema-gfx)
        fetch_nema_gfx
    ;;
    *)
        echo "unknown command"
    ;;
esac
