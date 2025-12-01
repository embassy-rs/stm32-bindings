#!/usr/bin/env bash

set -e
cd $(dirname $0)

CMD=$1
REV=v1.8.0
shift

case "$CMD" in
    gen)
        cargo run --release stm32-bindings-gen
    ;;
    download-all)
        rm -rf ./sources/
        git clone --branch $REV https://github.com/STMicroelectronics/STM32CubeWBA.git ./sources/STM32CubeWBA/ --depth 1 -q
        cd ./sources/
    ;;
    *)
        echo "unknown command"
    ;;
esac
