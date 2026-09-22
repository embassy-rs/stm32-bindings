#!/usr/bin/env bash

set -e
cd $(dirname $0)

CMD=$1
WBA_REV=v1.10.0
WB_REV=v1.24.0
shift

case "$CMD" in
    gen)
        cargo run --release stm32-bindings-gen
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
    *)
        echo "unknown command"
    ;;
esac
