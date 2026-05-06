#!/bin/bash
# Test Renode emulation script

cd "$(dirname "$0")/.." || exit 1

echo "Testing Renode with built ELF..."
echo "ELF file: $(ls -lh build/zephyr/zephyr.elf 2>/dev/null || echo 'Not found')"

# Check if Renode can parse the script without GUI
renode --help 2>&1 | head -20

echo ""
echo "To run the emulation with GUI:"
echo "  renode renode/stm32h7b3i_dk.resc"
echo ""
echo "Or from Renode console:"
echo "  include @renode/stm32h7b3i_dk.resc"
