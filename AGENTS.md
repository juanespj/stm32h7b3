<<<<<<< Updated upstream
# STM32H7B3 Bootloader Project

## Project Overview
Rust embedded project for STM32H7B3LIHxQ on STM32H7B3I Discovery Kit (MB1315).
Uses Embassy async framework with probe-rs for debugging/flashing.

## Directory Structure
```
stm32h7b3/
├── Cargo.toml              # Workspace root (members: app, bootloader)
├── AGENTS.md               # This file
├── Embed.toml              # probe-rs RTT config
├── app/                    # Application firmware
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08010000, 1984KB
│   └── src/main.rs         # LED blinky (PG13)
├── bootloader/             # UART bootloader
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08000000, 64KB
│   └── src/main.rs         # UART7 update + flash
└── scripts/
    └── flash-all.ps1       # Build, merge, and flash both
```

## MCU & Board
- **MCU**: STM32H7B3LIHxQ (Cortex-M7, 280MHz)
- **Flash**: 2MB (0x08000000 - 0x081FFFFF), sectors 0-7 at 128KB each
- **AXI SRAM**: 1MB (0x24000000 - 0x240FFFFF)
- **Board**: STM32H7B3I Discovery (MB1315)
- **LED**: PG13
- **Button**: PA0 (active low / pulled up)
- **Touch**: FT5336, I2C4 0x38, SCL=PD12, SDA=PD13
- **Debug**: probe-rs via SWD

## Memory Layout
| Region      | Start      | Size   | Contents        |
|-------------|------------|--------|-----------------|
| Bootloader  | 0x08000000 | 64KB   | UART bootloader |
| Application | 0x08010000 | 1984KB | User firmware   |

## Build System
- **Target**: thumbv7em-none-eabihf
- **Linker**: cortex-m-rt link.x + defmt.x
- **Profiles**: opt-level = "z" (size), fat LTO, defined in workspace root Cargo.toml

## Bootloader Protocol
UART7 at 115200 8N1, PA8=RX, PA15=TX.
Uses XMODEM-like packet format:
```
[SOF: 0x01] [LEN: 0xFF] [SEQ] [~SEQ] [256 bytes data] [CHKSUM]
```
- SOF (0x01) starts packet transfer
- ACK (0x06) / NACK (0x15) responses
- EOT (0x04) ends transfer, launches app
- CAN (0x18) aborts transfer
- Boot enters update mode if PA0 button held or no valid app found

## Build/Flash Commands

### Build both
```
cargo build -p bootloader --release
cargo build -p app --release
```

### Flash individually
```
cargo run -p bootloader --release
cargo run -p app --release
```

### Build, merge, and flash (single command)
```
.\scripts\flash-all.ps1
.\scripts\flash-all.ps1 -Action build    # Just build + merge
.\scripts\flash-all.ps1 -Action flash    # Just flash existing merge
.\scripts\flash-all.ps1 -Profile debug   # Debug build
```

## Flash Driver (Bootloader)
Direct register access to flash controller at 0x52002000.
- Unlock keys: 0x45670123, 0xCDEF89AB
- Program width: 32-bit (PSIZE=01)
- Erase: Sector Erase (SER), 128KB sectors
- Wait: Busy bit (BSY) in FLASH_SR

## Key Dependencies
- embassy-stm32 0.6.0
- embassy-executor 0.10.0
- embassy-time 0.5.1
- cortex-m 0.7.6
- defmt 1.0.1 + defmt-rtt 1.1.0
=======
# STM32H7B3 Bootloader Project

## Project Overview
Rust embedded project for STM32H7B3LIHxQ on STM32H7B3I Discovery Kit (MB1315).
Uses Embassy async framework with probe-rs for debugging/flashing.

## Directory Structure
```
stm32h7b3/
├── Cargo.toml              # Workspace root (members: app, bootloader)
├── AGENTS.md               # This file
├── Embed.toml              # probe-rs RTT config
├── app/                    # Application firmware
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08010000, 1984KB
│   └── src/main.rs         # LED blinky (PG13)
├── bootloader/             # UART bootloader
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08000000, 64KB
│   └── src/main.rs         # UART7 update + flash
└── scripts/
    └── flash-all.ps1       # Build, merge, and flash both
```

## MCU & Board
- **MCU**: STM32H7B3LIHxQ (Cortex-M7, 280MHz)
- **Flash**: 2MB (0x08000000 - 0x081FFFFF), sectors 0-7 at 128KB each
- **AXI SRAM**: 1MB (0x24000000 - 0x240FFFFF)
- **Board**: STM32H7B3I Discovery (MB1315)
- **LED**: PG13
- **Button**: PA0 (active low / pulled up)
- **Debug**: probe-rs via SWD

## Memory Layout
| Region      | Start      | Size   | Contents        |
|-------------|------------|--------|-----------------|
| Bootloader  | 0x08000000 | 64KB   | UART bootloader |
| Application | 0x08010000 | 1984KB | User firmware   |

## Build System
- **Target**: thumbv7em-none-eabihf
- **Linker**: cortex-m-rt link.x + defmt.x
- **Profiles**: opt-level = "z" (size), fat LTO, defined in workspace root Cargo.toml

## Bootloader Protocol
UART7 at 115200 8N1, PA8=RX, PA15=TX.
Uses XMODEM-like packet format:
```
[SOF: 0x01] [LEN: 0xFF] [SEQ] [~SEQ] [256 bytes data] [CHKSUM]
```
- SOF (0x01) starts packet transfer
- ACK (0x06) / NACK (0x15) responses
- EOT (0x04) ends transfer, launches app
- CAN (0x18) aborts transfer
- Boot enters update mode if PA0 button held or no valid app found

## Build/Flash Commands

### Build both
```
cargo build -p bootloader --release
cargo build -p app --release
```

### Flash individually
```
cargo run -p bootloader --release
cargo run -p app --release
```

### Build, merge, and flash (single command)
```
.\scripts\flash-all.ps1
.\scripts\flash-all.ps1 -Action build    # Just build + merge
.\scripts\flash-all.ps1 -Action flash    # Just flash existing merge
.\scripts\flash-all.ps1 -Profile debug   # Debug build
```

## Flash Driver (Bootloader)
Direct register access to flash controller at 0x52002000.
- Unlock keys: 0x45670123, 0xCDEF89AB
- Program width: 32-bit (PSIZE=01)
- Erase: Sector Erase (SER), 128KB sectors
- Wait: Busy bit (BSY) in FLASH_SR

## Key Dependencies
- embassy-stm32 0.6.0
- embassy-executor 0.10.0
- embassy-time 0.5.1
- cortex-m 0.7.6
- defmt 1.0.1 + defmt-rtt 1.1.0
>>>>>>> Stashed changes
