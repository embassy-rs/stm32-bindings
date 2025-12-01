#!/usr/bin/env bash

set -e
cd $(dirname $0)

CMD=$1
REV=8382ecbcc976ab5e91070b386700dbfd1e654275
shift

case "$CMD" in
    gen)
        cargo run --release stm32-bindings-gen -- --target=thumbv8m.main-none-eabihf
    ;;
    download-all)
        rm -rf ./sources/
        git clone https://github.com/STMicroelectronics/STM32CubeWBA.git ./sources/ -q
        cd ./sources/
        git checkout $REV
    ;;
    *)
        echo "unknown command"
    ;;
esac
