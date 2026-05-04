#Requires -Version 5.1

param(
    [ValidateSet("build", "flash", "all")]
    [string]$Action = "all",
    
    [ValidateSet("debug", "release")]
    [string]$Profile = "release",
    
    [string]$Chip = "STM32H7B3LIHxQ"
)

$ProjectRoot = Split-Path $PSScriptRoot -Parent
$TargetDir = Join-Path $ProjectRoot "target\thumbv7em-none-eabihf\$Profile"
$BootElf = Join-Path $TargetDir "bootloader"
$AppElf = Join-Path $TargetDir "app"
$BootBin = Join-Path $TargetDir "bootloader.bin"
$AppBin = Join-Path $TargetDir "app.bin"
$MergedBin = Join-Path $TargetDir "firmware.bin"

$AppOffset = 0x10000  # 64KB bootloader region
$FlashSize = 0x200000 # 2MB total flash

function Log {
    param([string]$Message, [string]$Color = "White")
    Write-Host "[$(Get-Date -Format 'HH:mm:ss')] " -NoNewline -ForegroundColor Gray
    Write-Host $Message -ForegroundColor $Color
}

function Build-Crates {
    Log "Building bootloader..." "Cyan"
    cargo build -p bootloader --profile $Profile
    if ($LASTEXITCODE -ne 0) { throw "Bootloader build failed" }
    
    Log "Building application..." "Cyan"
    cargo build -p app --profile $Profile
    if ($LASTEXITCODE -ne 0) { throw "Application build failed" }
}

function Create-Binaries {
    Log "Extracting binaries..." "Yellow"
    
    $objcopy = Get-Command "arm-none-eabi-objcopy" -ErrorAction SilentlyContinue
    if (-not $objcopy) {
        $llvmObjcopy = Get-Command "llvm-objcopy" -ErrorAction SilentlyContinue
        if ($llvmObjcopy) {
            $objcopy = $llvmObjcopy
        } else {
            throw "Neither arm-none-eabi-objcopy nor llvm-objcopy found. Install llvm-tools-preview: rustup component add llvm-tools-preview"
        }
    }
    
    & $objcopy.Source -O binary $BootElf $BootBin
    & $objcopy.Source -O binary $AppElf $AppBin
}

function Merge-Binaries {
    Log "Merging binaries..." "Yellow"
    
    $bootSize = (Get-Item $BootBin).Length
    $appSize = (Get-Item $AppBin).Length
    
    Log "  Bootloader: $bootSize bytes ($([math]::Round($bootSize/1024, 1)) KB)" "Gray"
    Log "  Application: $appSize bytes ($([math]::Round($appSize/1024, 1)) KB) at 0x$("{0:X}" -f $AppOffset)" "Gray"
    
    if ($bootSize -gt $AppOffset) {
        throw "Bootloader ($bootSize bytes) exceeds reserved flash ($AppOffset bytes)!"
    }
    
    $merged = New-Object byte[] $FlashSize
    
    [Array]::Copy([System.IO.File]::ReadAllBytes($BootBin), 0, $merged, 0, $bootSize)
    [Array]::Copy([System.IO.File]::ReadAllBytes($AppBin), 0, $merged, $AppOffset, $appSize)
    
    [System.IO.File]::WriteAllBytes($MergedBin, $merged)
    
    Log "  Merged: $($merged.Length) bytes written to firmware.bin" "Green"
}

function Flash-Device {
    Log "Flashing to device..." "Magenta"
    probe-rs download $MergedBin --chip $Chip --verify
    if ($LASTEXITCODE -ne 0) { throw "Flash failed" }
    Log "Flash complete!" "Green"
}

try {
    Log "STM32H7B3 Firmware Builder" "White"
    Log "Profile: $Profile | Chip: $Chip" "Gray"
    
    if ($Action -eq "build" -or $Action -eq "all") {
        Build-Crates
        Create-Binaries
        Merge-Binaries
    }
    
    if ($Action -eq "flash" -or $Action -eq "all") {
        if (-not (Test-Path $MergedBin)) {
            Log "No merged binary found, building first..." "Yellow"
            Build-Crates
            Create-Binaries
            Merge-Binaries
        }
        Flash-Device
    }
    
    Log "Done!" "Green"
}
catch {
    Log "Error: $_" "Red"
    exit 1
}
