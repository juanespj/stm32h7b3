---
name: west-env
description: "Use this skill when running Zephyr west tasks in this repository. Activate the Zephyr environment before any west command."
applyTo:
  - "**/*"
---

# Zephyr west environment skill

This repository requires Zephyr environment setup before using `west`.

## Always run before any west task

```bash
export ZEPHYR_BASE=~/zephyrproject/zephyr
source $ZEPHYR_BASE/zephyr-env.sh
source ~/zephyrproject/.venv/bin/activate
```

If `west` is still not found after these steps, open a fresh shell or source your shell startup file.

## Recommended workflow

1. Open a terminal in this workspace.
2. Run the commands above.
3. Then run `west build`, `west flash`, or any `west` subcommand.

## Why this matters

Without the Zephyr environment active, `west` may not be on PATH and Zephyr toolchain configuration may be missing. This is the expected setup for this workspace.
