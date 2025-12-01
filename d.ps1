<# #>
param (
    [Parameter(Mandatory = $true)]
    [string]$CMD,

    [string]$peri
)

$REV = "8382ecbcc976ab5e91070b386700dbfd1e654275"

Switch ($CMD) {
    "gen" {
        cargo run --release stm32-bindings-gen -- --target=thumbv8m.main-none-eabihf
    }
    "download-all" {
        rm -r -Force ./sources/ -ErrorAction SilentlyContinue
        git clone https://github.com/STMicroelectronics/STM32CubeWBA.git ./sources/
        cd ./sources/
        git checkout $REV
        cd ..
    }
    default {
        echo "unknown command"
    }
}