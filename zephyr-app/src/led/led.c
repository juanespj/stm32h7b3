#include "led.h"
#include <zephyr/drivers/gpio.h>

static const struct gpio_dt_spec led = GPIO_DT_SPEC_GET(DT_ALIAS(led0), gpios);

void led_init(void) {
    if (!gpio_is_ready_dt(&led)) {
        printk("LED not ready\n");
        return;
    }
    gpio_pin_configure_dt(&led, GPIO_OUTPUT_ACTIVE);
}

void led_toggle(void) {
    gpio_pin_toggle_dt(&led);
}