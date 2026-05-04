# LTDC Display Driver Progress - STM32H7B3I-DK

## Goal
Implement LTDC display driver for the 4.3" RK043FN48H RGB LCD on STM32H7B3I-DK using Rust + Embassy.
- Framebuffer in AXI SRAM at `0x24070000` (RGB565, 480x272)
- SDRAM path blocked (FMC inaccessible)

## Current State (as of 2026-05-04)

### What Works
- GPIO configuration via direct register access (AF14 for LTDC pins on PI/PJ/PK)
- Embassy initialization with PLL1 (207.36 MHz SYSCLK from HSE 24MHz)
- RCC register writes succeed (APB3ENR LTDC clock enable)
- **LTDC direct register initialization works without crashes**
- Framebuffer fills successfully with test colors
- Program continues running after LTDC enable (GPIO blink loop verified)

### What's Still Needed
- Verify actual display output (blue screen test)
- Implement animation/color cycling to prove framebuffer updates
- SDRAM/FMC initialization remains blocked (separate issue)

### Direct Register Approach
Using raw MMIO writes to LTDC at `0x50001000`:
1. Configure GPIOs (PI12-15, PJ0-15, PK0-7) as AF14 with very high speed
2. Enable LTDC clock via `RCC.APB3ENR` bit 3
3. Configure timing (HSYNC=41, HBP=13, VSYNC=10, VBP=3, etc.)
4. Configure Layer 1: RGB565, framebuffer address, pitch, dimensions
5. Fill framebuffer with test color
6. Enable layer, reload, enable LTDC controller

### Key Success: Framebuffer Placement
- **Original crash cause**: Framebuffer at `0x24000000` overlapped with `defmt_rtt::BUFFER` at `0x240ffe1e`
- **Solution**: Framebuffer at `0x24070000` (448KB into AXI SRAM, away from RTT buffer and stack)
- Framebuffer size: 480 × 272 × 2 bytes = 261,120 bytes (~255KB)

### Memory Map
| Region | Address | Size | Notes |
|---|---|---|---|
| LTDC registers | 0x50001000 | 4KB | Display controller |
| Framebuffer | 0x24070000 | 255KB | AXI SRAM, RGB565 |
| defmt_rtt buffer | ~0x240ffe1e | ~1KB | RTT logging |
| Stack | 0x24100000 down | varies | Cortex-M stack |

### RK043FN48H Timing
```
Horizontal: HSYNC=41, HBP=13, Active=480, HFP=32, Total=566
Vertical:   VSYNC=10, VBP=3,  Active=272, VFP=13, Total=298
```

### Build/Flash
```powershell
cargo build -p app --release
probe-rs run --chip STM32H7B3LIHxQ --protocol swd target/thumbv7em-none-eabihf/release/app
```

### Reference Files in REF/
- `um2569-discovery-kit-with-stm32h7b3li-mcu-stmicroelectronics.pdf` - Board manual
- `stm32h7b3i_discovery_sdram.c` - ST SDRAM init example
- `stm32h7b3ri.pdf` - Reference manual (closest available)
- `system_stm32h7xx.c` - System clock init reference
- `is42s16800j.c` - SDRAM timing configuration

### SDRAM/FMC Status
- FMC base: `0x52002000`
- AHB3ENR bit 0 (FMCEN) can be set, reads back as set
- Any access to FMC registers triggers BusFault
- Likely `stm32-metapac` metadata issue for `stm32h7b3li`
- Workaround: Use AXI SRAM for framebuffer (proven working)
