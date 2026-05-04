find pins without guessing
grep -n "UART7" target/thumbv7em-none-eabihf/release/build/embassy-stm32-*/out/_generated.rs

stm32h7b3/
├── Cargo.toml              # Workspace root
├── app/                    # Application (your blinky)
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08010000 (1984K)
│   └── src/main.rs
├── bootloader/             # UART bootloader
│   ├── .cargo/config.toml
│   ├── Cargo.toml
│   ├── memory.x            # Flash: 0x08000000 (64K)
│   └── src/main.rs
└── memory.x                # (original, can be removed)

Memory layout:

Region	Address	Size
Bootloader	0x08000000	64KB
Application	0x08010000	1984KB
Binary sizes (release):

Bootloader: 15.4 KB (fits easily in 64KB)
App: 12.8 KB
Bootloader behavior:

Checks PA0 button on boot - if held, enters update mode
If no button and valid app exists, jumps to app at 0x08010000
In update mode: uses UART7 (PA8=RX, PA15=TX, 115200 8N1) with XMODEM-like protocol (SOF/ACK/NACK)
Erases flash sectors, programs 32-bit words, then verifies and jumps
To flash:

# Flash bootloader first
cargo run -p bootloader --release

# Then flash app
cargo run -p app --release
To send firmware over UART: Use a tool that speaks XMODEM protocol or write a sender using the SOF(0x01) + length + seq + ~seq + data + checksum packet format.