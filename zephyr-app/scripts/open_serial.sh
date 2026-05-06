#!/bin/bash
# Open serial port for STM32H7B3I-DK UART console
# Default: 115200 baud, 8N1

BAUD_RATE=${1:-115200}

# On macOS, find ST-LINK serial port
if [[ "$OSTYPE" == "darwin"* ]]; then
    # Look for the ST-LINK probe device (usbserial, usbmodem, etc.)
    SERIAL_PORT=$(ls /dev/cu.usbmodem* 2>/dev/null | head -1)
    
    if [ -z "$SERIAL_PORT" ]; then
        SERIAL_PORT=$(ls /dev/cu.usbserial* 2>/dev/null | head -1)
    fi
    
    if [ -z "$SERIAL_PORT" ]; then
        SERIAL_PORT=$(ls /dev/cu.usbacm* 2>/dev/null | head -1)
    fi
    
    if [ -z "$SERIAL_PORT" ]; then
        SERIAL_PORT=$(ls /dev/tty.usbmodem* 2>/dev/null | head -1)
    fi
    
    if [ -z "$SERIAL_PORT" ]; then
        echo "ERROR: No serial port found."
        echo "Available ports:"
        ls -la /dev/cu.* 2>/dev/null | grep -E 'usb|USB' || echo "  (no USB devices found)"
        echo "Make sure ST-LINK is connected and drivers are installed."
        exit 1
    fi
    
    echo "Found serial port: $SERIAL_PORT"
    echo "Opening at $BAUD_RATE baud..."
    
    # Try screen (comes with macOS)
    if command -v screen &> /dev/null; then
        screen "$SERIAL_PORT" "$BAUD_RATE"
    elif command -v minicom &> /dev/null; then
        minicom -D "$SERIAL_PORT" -b "$BAUD_RATE"
    elif command -v picocom &> /dev/null; then
        picocom -b "$BAUD_RATE" "$SERIAL_PORT"
    else
        echo "ERROR: No terminal program found (screen, minicom, picocom)"
        echo "Install one of: brew install minicom picocom"
        exit 1
    fi

elif [[ "$OSTYPE" == "linux-gnu"* ]]; then
    # On Linux, look for /dev/ttyACM* or /dev/ttyUSB*
    SERIAL_PORT=$(ls /dev/ttyACM* 2>/dev/null | head -1)
    
    if [ -z "$SERIAL_PORT" ]; then
        SERIAL_PORT=$(ls /dev/ttyUSB* 2>/dev/null | head -1)
    fi
    
    if [ -z "$SERIAL_PORT" ]; then
        echo "ERROR: No serial port found."
        echo "Check with: dmesg | grep tty"
        exit 1
    fi
    
    echo "Found serial port: $SERIAL_PORT"
    echo "Opening at $BAUD_RATE baud..."
    
    if command -v minicom &> /dev/null; then
        minicom -D "$SERIAL_PORT" -b "$BAUD_RATE"
    elif command -v screen &> /dev/null; then
        screen "$SERIAL_PORT" "$BAUD_RATE"
    elif command -v picocom &> /dev/null; then
        picocom -b "$BAUD_RATE" "$SERIAL_PORT"
    else
        echo "ERROR: No terminal program found (minicom, screen, picocom)"
        echo "Install one of: apt install minicom screen picocom"
        exit 1
    fi

else
    echo "Unsupported OS: $OSTYPE"
    exit 1
fi
