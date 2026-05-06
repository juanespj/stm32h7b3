# SPDX-License-Identifier: Apache-2.0

cmake_minimum_required(VERSION 3.20.0)

find_package(Zephyr REQUIRED HINTS $ENV{ZEPHYR_BASE})
project(stm32h7b3-display-demo)

target_sources(app PRIVATE src/main.c src/ui/ui.c src/led/led.c)
