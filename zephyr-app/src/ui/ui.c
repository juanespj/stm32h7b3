#include <zephyr/kernel.h>
#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/display.h>
#include <lvgl.h>

#include "ui.h"

#define UI_STACK_SIZE 4096
#define UI_PRIORITY K_PRIO_PREEMPT(5)

K_THREAD_STACK_DEFINE(ui_stack, UI_STACK_SIZE);
static struct k_thread ui_thread_data;
static lv_obj_t *counter_label;
static int32_t counter;

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

static void create_ui_screen(void)
{
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
}

static void ui_thread(void *a, void *b, void *c)
{
    ARG_UNUSED(a);
    ARG_UNUSED(b);
    ARG_UNUSED(c);

    while (1) {
        lv_task_handler();
        k_msleep(10);
    }
}

void ui_init(void)
{
    const struct device *display = DEVICE_DT_GET(DT_CHOSEN(zephyr_display));
    if (!device_is_ready(display)) {
        printk("Display not ready\n");
        return;
    }

    display_blanking_off(display);
    create_ui_screen();

    k_thread_create(&ui_thread_data, ui_stack, UI_STACK_SIZE, ui_thread, NULL, NULL, NULL,
                    UI_PRIORITY, 0, K_NO_WAIT);
}
