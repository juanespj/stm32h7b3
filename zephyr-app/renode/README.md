# Renode Emulation for STM32H7B3I-DK

## Prerequisites

Install Renode:
```bash
brew install --cask renode
# or download from https://renode.io/download
```

## Build Zephyr Project

```bash
source ~/zephyrproject/.venv/bin/activate
cd zephyr-app
west build -b stm32h7b3i_dk
```

## Run Emulation

### Method 1: Using Renode GUI
```bash
renode zephyr-app/renode/stm32h7b3i_dk.resc
```

### Method 2: Command Line
Start Renode and run:
```
include @zephyr-app/renode/stm32h7b3i_dk.resc
```

## After Build - Update Script

After building, edit `stm32h7b3i_dk.resc` and uncomment the LoadELF line:
```
sysbus LoadELF @build/zephyr/zephyr.elf
```

## What Works in Emulation

- ✅ UART1 console (USART1 at 115200 baud)
- ✅ GPIO LEDs (PG11 - LD1)
- ❌ Display (LTDC) - not emulated in Renode
- ❌ Touch screen - not emulated

## Debugging

Open UART analyzer in Renode to see console output:
```
showAnalyzer uart1
```
