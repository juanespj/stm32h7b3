#include <zephyr/device.h>
#include <zephyr/devicetree.h>
#include <zephyr/drivers/display.h>
#include <zephyr/drivers/gpio.h>
#include <zephyr/kernel.h>
#include <zephyr/sys/printk.h>

static const struct gpio_dt_spec led = GPIO_DT_SPEC_GET(DT_ALIAS(led0), gpios);
static const struct device *display = DEVICE_DT_GET(DT_CHOSEN(zephyr_display));

int main(void)
{
    int ret;

    printk("STM32H7B3I display demo\n");

    if (!gpio_is_ready_dt(&led)) {
        printk("LED not ready\n");
        return -1;
    }

    gpio_pin_configure_dt(&led, GPIO_OUTPUT_ACTIVE);

    if (!device_is_ready(display)) {
        printk("Display device not ready\n");
    } else {
        struct display_capabilities caps;
        struct display_buffer_descriptor desc;
        static uint16_t line_buf[480];  /* Keep on heap, not stack */
        uint16_t x;

        display_get_capabilities(display, &caps);
        printk("Display: %ux%u, supported formats=0x%08x\n",
               caps.x_resolution, caps.y_resolution,
               caps.supported_pixel_formats);

        if (display_set_pixel_format(display, PIXEL_FORMAT_RGB_565) == 0) {
            printk("Display pixel format set to RGB565\n");
        }

        desc.buf_size = sizeof(line_buf);
        desc.width = caps.x_resolution;
        desc.height = 1;
        desc.pitch = caps.x_resolution;
        desc.frame_incomplete = false;

        for (x = 0; x < caps.x_resolution; x++) {
            line_buf[x] = 0xF800U; /* solid red */
        }

        ret = display_blanking_off(display);
        if (ret != 0) {
            printk("display_blanking_off failed: %d\n", ret);
        }

        for (uint16_t y = 0; y < caps.y_resolution; y++) {
            ret = display_write(display, 0, y, &desc, line_buf);
            if (ret != 0) {
                printk("display_write failed at row %u: %d\n", y, ret);
                break;
            }
        }

        if (ret == 0) {
            printk("Screen filled red\n");
        }
    }

    while (1) {
        gpio_pin_toggle_dt(&led);
        k_msleep(1000);
    }

    return 0;
}
