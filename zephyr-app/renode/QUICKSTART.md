# Quick Start - Renode Emulation

## Option 1: Run directly from terminal (Recommended)

```bash
cd /Users/Juan.EstebanPaz/Documents/GitHub/stm32h7b3
renode zephyr-app/renode/stm32h7b3i_dk.resc
```

This will open Renode GUI and automatically load the platform and ELF file.

## Option 2: From Renode monitor

```bash
cd /Users/Juan.EstebanPaz/Documents/GitHub/stm32h7b3
renode
```

Then in Renode monitor:
```
include @zephyr-app/renode/stm32h7b3i_dk.resc
```

## What to expect

1. Renode window opens
2. UART1 analyzer window shows console output
3. LED LD1 (red) should blink on virtual GPIO port

## Troubleshooting

If you see "Could not find file" errors:
- Make sure you're in the project root directory (`/Users/Juan.EstebanPaz/Documents/GitHub/stm32h7b3`)
- Verify the ELF file exists: `ls -la zephyr-app/build/zephyr/zephyr.elf`
- If missing, rebuild: `cd zephyr-app && west build -b stm32h7b3i_dk`
