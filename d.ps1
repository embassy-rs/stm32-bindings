<# #>
param (
    [Parameter(Mandatory = $true)]
    [string]$CMD,

    [string]$peri
)

$WBA_REV = ((Select-String -Path ".\d" -Pattern "^WBA_REV=") -split "=")[1]
$WB_REV = ((Select-String -Path ".\d" -Pattern "^WB_REV=") -split "=")[1]
$XCUBE_IP_REV = ((Select-String -Path ".\d" -Pattern "^XCUBE_IP_REV=") -split "=")[1]

function Fetch-NemaGfx {
    $tmp = New-Item -ItemType Directory -Path ([System.IO.Path]::GetTempPath() + [System.Guid]::NewGuid()) -Force
    git clone https://github.com/STMicroelectronics/x-cube-image-processing.git "$tmp" --depth 1
    Push-Location $tmp
    git fetch origin $XCUBE_IP_REV 2>$null
    git checkout FETCH_HEAD 2>$null
    if ($LASTEXITCODE -ne 0) { git checkout $XCUBE_IP_REV 2>$null }
    if ($LASTEXITCODE -ne 0) { git checkout main }
    Pop-Location

    Remove-Item -Recurse -Force ./stm32-bindings-gen/nema_gfx/include -ErrorAction SilentlyContinue
    New-Item -ItemType Directory -Force -Path ./stm32-bindings-gen/nema_gfx/include, ./stm32-bindings-gen/nema_gfx/lib | Out-Null
    Copy-Item -Recurse "$tmp/Middleware/NemaGFX/include/*" ./stm32-bindings-gen/nema_gfx/include/
    Copy-Item "$tmp/Middleware/NemaGFX/LICENSE.md" ./stm32-bindings-gen/nema_gfx/LICENSE.md
    @(
        "x-cube-image-processing $XCUBE_IP_REV",
        "NemaGFX middleware v1.4.17"
    ) | Set-Content ./stm32-bindings-gen/nema_gfx/VERSION

    $pairs = @{
        "cortex_m33_revC" = "cortex_m33_revc"
        "cortex_m33_NemaPVG" = "cortex_m33_nemapvg"
        "cortex_m7" = "cortex_m7"
        "cortex_m55" = "cortex_m55"
    }
    foreach ($entry in $pairs.GetEnumerator()) {
        Copy-Item "$tmp/Middleware/NemaGFX/lib/core/$($entry.Key)/gcc/libnemagfx-float-abi-hard.a" `
            "./stm32-bindings-gen/nema_gfx/lib/libnemagfx_$($entry.Value)_float_abi_hard.a"
    }

    Remove-Item -Recurse -Force $tmp
    Write-Host "NemaGFX headers and libraries updated under stm32-bindings-gen/nema_gfx/"
}

Switch ($CMD) {
    "gen" {
        cargo run --release --bin stm32-bindings-gen
    }
    "download-all" {
        while (Test-Path "sources") {
            rm -r -Force sources -ErrorAction SilentlyContinue
        }

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
    }
    "build-thread" {
        mkdir build 2>&1 1> $null
        rm -r -Force build/thread 2>&1 1> $null
        mkdir build/thread 2>&1 1> $null

        cp -r stm32-bindings-gen/thread build -ErrorAction SilentlyContinue
        cd build/thread

        cmake -B build -G Ninja -DCMAKE_C_COMPILER=arm-none-eabi-gcc -DCMAKE_BUILD_TYPE=Release "-DCMAKE_TOOLCHAIN_FILE=arm-gcc-toolchain.cmake"
        cmake --build build

        cd ../..
    }
    "fetch-nema-gfx" {
        Fetch-NemaGfx
    }
    default {
        echo "unknown command"
    }
}