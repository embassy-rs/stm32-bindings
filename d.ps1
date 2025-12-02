<# #>
param (
    [Parameter(Mandatory = $true)]
    [string]$CMD,

    [string]$peri
)

$REV = "v1.8.0"

Switch ($CMD) {
    "gen" {
        cargo run --release stm32-bindings-gen
    }
    "download-all" {
        rm -r -Force ./sources/ -ErrorAction SilentlyContinue
        git clone --branch $REV https://github.com/STMicroelectronics/STM32CubeWBA.git ./sources/STM32CubeWBA/ --depth 1 -q
        cd ..
    }
    default {
        echo "unknown command"
    }
}