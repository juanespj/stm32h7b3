#include <zephyr/kernel.h>
#include "led/led.h"
#include "ui/ui.h"

void main(void)
{
    printk("STM32H7B3I LVGL demo with touch\n");

    led_init();
    ui_init();

    while (1) {
        led_toggle();
        k_msleep(500);
    }
}
