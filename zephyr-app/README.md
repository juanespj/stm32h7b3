# STM32H7B3I-DK Zephyr Display Demo

Before building, set up the Zephyr environment:

```bash
export ZEPHYR_BASE=~/zephyrproject/zephyr
source "$ZEPHYR_BASE/zephyr-env.sh"
source ~/zephyrproject/.venv/bin/activate
```

## Build Instructions

```bash
# Activate west virtual environment
source ~/zephyrproject/.venv/bin/activate

# Build the project
cd zephyr-app
west build -b stm32h7b3i_dk

# Flash to board
west flash
```

## Project Structure

- `CMakeLists.txt` - CMake build configuration
- `prj.conf` - Zephyr configuration options
- `src/main.c` - Application source with display demo

## Features

- LTDC display driver initialized
- Red LED blink on PG11
- Fills screen with red color
- Console output via USART1

## Local Ollama Chat

This workspace includes a VS Code settings file for the Ollama Copilot extension.

To use the local Ollama chat agent:

1. Make sure Ollama is installed and running locally.
2. Install a code-capable Ollama model such as `codellama`.
3. Start the Ollama server if needed, or run the extension command `Ollama Copilot: Open Chat`.
4. Open the chat panel from the Ollama icon in the Activity Bar.

The workspace config is in `.vscode/settings.json` and is set to use `codellama:latest` for both completion and chat.
