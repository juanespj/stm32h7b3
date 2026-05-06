#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/display.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/input/input.h>
#include <lvgl.h>

static const struct gpio_dt_spec led = GPIO_DT_SPEC_GET(DT_ALIAS(led0), gpios);
static lv_obj_t *counter_label;
static int32_t counter = 0;

static void button_event_cb(lv_event_t *e)
{
    counter++;
    lv_label_set_text_fmt(counter_label, "Count: %d", counter);
}

static void slider_event_cb(lv_event_t *e)
{
    lv_obj_t *slider = lv_event_get_target(e);
    int32_t value = lv_slider_get_value(slider);
    lv_label_set_text_fmt(counter_label, "Slider: %d", value);
}

void main(void)
{
    printk("STM32H7B3I LVGL demo with touch\n");

    if (!gpio_is_ready_dt(&led)) {
        printk("LED not ready\n");
        return;
    }
    gpio_pin_configure_dt(&led, GPIO_OUTPUT_ACTIVE);

    const struct device *display = DEVICE_DT_GET(DT_CHOSEN(zephyr_display));
    if (!device_is_ready(display)) {
        printk("Display not ready\n");
        return;
    }

    /* Zephyr LVGL calls lv_init() in SYS_INIT; calling it again corrupts screens. */
    display_blanking_off(display);

    lv_obj_t *scr = lv_scr_act();
    if (scr == NULL) {
        printk("LVGL: no active screen\n");
        return;
    }

    lv_obj_t *btn = lv_btn_create(scr);
    lv_obj_set_pos(btn, 160, 100);
    lv_obj_set_size(btn, 160, 50);
    lv_obj_add_event_cb(btn, button_event_cb, LV_EVENT_CLICKED, NULL);

    lv_obj_t *btn_label = lv_label_create(btn);
    lv_label_set_text(btn_label, "Click Me");
    lv_obj_center(btn_label);

    lv_obj_t *slider = lv_slider_create(scr);
    lv_obj_set_pos(slider, 100, 180);
    lv_obj_set_size(slider, 280, 20);
    lv_obj_add_event_cb(slider, slider_event_cb, LV_EVENT_VALUE_CHANGED, NULL);

    counter_label = lv_label_create(scr);
    lv_label_set_text(counter_label, "Count: 0");
    lv_obj_set_pos(counter_label, 160, 40);

    lv_task_handler();

    while (1) {
        lv_task_handler();
        gpio_pin_toggle_dt(&led);
        k_msleep(10);
    }
}
