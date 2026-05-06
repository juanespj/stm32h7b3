#![no_std]
#![no_main]
#![macro_use]
#![allow(static_mut_refs)]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::i2c::{Config as I2cConfig, I2c, Master};
use embassy_stm32::ltdc::{
    self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge,
};
use embassy_stm32::mode::Blocking;
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::Timer;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::mono_font::{ascii::FONT_10X20, MonoTextStyle};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle, StyledDrawable};
use embedded_graphics::text::{Baseline, Text};
use static_cell::StaticCell;
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 480;
const DISPLAY_HEIGHT: usize = 272;

pub static mut FB1: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [u8; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

struct LtdcDisplay {
    ltdc: Ltdc<'static, peripherals::LTDC>,
    layer_config: LtdcLayerConfig,
    buf0: *mut u8,
    buf1: *mut u8,
    len: usize,
    is_buf0: bool,
}

impl LtdcDisplay {
    fn new(
        ltdc: Ltdc<'static, peripherals::LTDC>,
        layer_config: LtdcLayerConfig,
        buf0: &'static mut [u8],
        buf1: &'static mut [u8],
    ) -> Self {
        Self {
            ltdc,
            layer_config,
            buf0: buf0.as_mut_ptr(),
            buf1: buf1.as_mut_ptr(),
            len: buf0.len(),
            is_buf0: true,
        }
    }

    async fn swap(&mut self) -> Result<(), ltdc::Error> {
        let buf = if self.is_buf0 { self.buf0 } else { self.buf1 };
        self.is_buf0 = !self.is_buf0;
        self.ltdc
            .set_buffer(self.layer_config.layer, buf as *const _)
            .await
    }

    fn current_buf(&mut self) -> &mut [u8] {
        let ptr = if self.is_buf0 { self.buf0 } else { self.buf1 };
        unsafe { core::slice::from_raw_parts_mut(ptr, self.len) }
    }

    fn clear(&mut self) {
        let buf = self.current_buf();
        for pixel in buf.iter_mut() {
            *pixel = 0;
        }
    }
}

impl DrawTarget for LtdcDisplay {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        let buf = self.current_buf();

        for pixel in pixels {
            let Pixel(point, color) = pixel;
            if point.x >= 0 && point.y >= 0 && point.x < width && point.y < height {
                let index = (point.y * width + point.x) as usize;
                if index < buf.len() {
                    buf[index] = rgb888_to_l8(color.r(), color.g(), color.b());
                }
            }
        }
        Ok(())
    }
}

impl OriginDimensions for LtdcDisplay {
    fn size(&self) -> Size {
        Size::new(
            (self.layer_config.window_x1 - self.layer_config.window_x0) as _,
            (self.layer_config.window_y1 - self.layer_config.window_y0) as _,
        )
    }
}

fn rgb888_to_l8(r: u8, g: u8, b: u8) -> u8 {
    let palette: [(u8, u8, u8); 8] = [
        (0, 0, 0),
        (255, 0, 0),
        (0, 255, 0),
        (0, 0, 255),
        (255, 255, 0),
        (255, 0, 255),
        (0, 255, 255),
        (255, 255, 255),
    ];
    let mut best = 0u8;
    let mut best_dist = u32::MAX;
    for (i, &(pr, pg, pb)) in palette.iter().enumerate() {
        let dr = (r as i32 - pr as i32).pow(2);
        let dg = (g as i32 - pg as i32).pow(2);
        let db = (b as i32 - pb as i32).pow(2);
        let dist = (dr + dg + db) as u32;
        if dist < best_dist {
            best_dist = dist;
            best = i as u8;
        }
    }
    best
}

struct Ft5336<'d> {
    i2c: I2c<'d, Blocking, Master>,
}

impl<'d> Ft5336<'d> {
    const ADDR: u8 = 0x38;

    fn new(i2c: I2c<'d, Blocking, Master>) -> Self {
        Self { i2c }
    }

    fn init(&mut self) {
        info!("Initializing touch controller at 0x{:02X}", Self::ADDR);
        let mut buf = [0u8; 1];
        match self.i2c.blocking_write_read(Self::ADDR, &[0x00], &mut buf) {
            Ok(_) => info!("Touch IC ID: {}", buf[0]),
            Err(e) => info!("Failed to init touch IC: {:?}", e),
        }
    }

    fn read_touch(&mut self) -> Option<(i32, i32)> {
        let mut buf = [0u8; 8];
        match self.i2c.blocking_write_read(Self::ADDR, &[0x02], &mut buf) {
            Ok(_) => {
                let num_touches = buf[0] & 0x0F;
                if num_touches > 0 {
                    let x = ((buf[1] as u16 & 0x0F) << 8) | buf[2] as u16;
                    let y = ((buf[3] as u16 & 0x0F) << 8) | buf[4] as u16;
                    let x = x.min(479) as i32;
                    let y = y.min(271) as i32;
                    return Some((x, y));
                }
                None
            }
            Err(e) => {
                info!("Touch read error: {:?}", e);
                None
            }
        }
    }
}

struct Button {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    label: &'static str,
    color: Rgb888,
    pressed: bool,
}

impl Button {
    fn new(x: i32, y: i32, width: i32, height: i32, label: &'static str, color: Rgb888) -> Self {
        Self {
            x,
            y,
            width,
            height,
            label,
            color,
            pressed: false,
        }
    }

    fn draw<D: DrawTarget<Color = Rgb888>>(&self, display: &mut D) -> Result<(), D::Error> {
        let rect = Rectangle::new(
            Point::new(self.x, self.y),
            Size::new(self.width as u32, self.height as u32),
        );
        let style = PrimitiveStyleBuilder::new().fill_color(self.color).build();
        rect.draw_styled(&style, display)?;

        let text_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));
        let text_width = (self.label.len() as i32) * 10;
        let text_x = self.x + (self.width - text_width) / 2;
        let text_y = self.y + self.height / 2 + 7;
        Text::with_baseline(
            self.label,
            Point::new(text_x, text_y),
            text_style,
            Baseline::Alphabetic,
        )
        .draw(display)?;

        Ok(())
    }

    fn contains(&self, x: i32, y: i32) -> bool {
        x >= self.x && x < self.x + self.width && y >= self.y && y < self.y + self.height
    }

    fn set_pressed(&mut self, pressed: bool) {
        self.pressed = pressed;
    }

    fn is_pressed(&self) -> bool {
        self.pressed
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = rcc_setup::stm32h7b3i_init();

    let led = Output::new(p.PG13, Level::High, Speed::Low);
    spawner.spawn(unwrap!(led_task(led)));

    let mut lcd_disp = Output::new(p.PA2, Level::Low, Speed::Low);
    let _lcd_bl = Output::new(p.PA1, Level::High, Speed::Low);
    Timer::after_millis(10).await;
    lcd_disp.set_high();

    const RK043FN48H_HSYNC: u16 = 41;
    const RK043FN48H_HBP: u16 = 13;
    const RK043FN48H_HFP: u16 = 32;
    const RK043FN48H_VSYNC: u16 = 10;
    const RK043FN48H_VBP: u16 = 2;
    const RK043FN48H_VFP: u16 = 2;

    let ltdc_config = LtdcConfiguration {
        active_width: DISPLAY_WIDTH as _,
        active_height: DISPLAY_HEIGHT as _,
        h_back_porch: RK043FN48H_HBP - 11,
        h_front_porch: RK043FN48H_HFP,
        v_back_porch: RK043FN48H_VBP,
        v_front_porch: RK043FN48H_VFP,
        h_sync: RK043FN48H_HSYNC,
        v_sync: RK043FN48H_VSYNC,
        h_sync_polarity: PolarityActive::ActiveLow,
        v_sync_polarity: PolarityActive::ActiveLow,
        data_enable_polarity: PolarityActive::ActiveLow,
        pixel_clock_polarity: PolarityEdge::FallingEdge,
    };

    info!("init ltdc");
    let mut ltdc = Ltdc::<_, ltdc::Rgb888>::new_with_pins(
        p.LTDC, Irqs, p.PI14, p.PI12, p.PI13, p.PK7, p.PJ12, p.PJ13, p.PJ14, p.PJ15, p.PK3, p.PK4,
        p.PK5, p.PK6, p.PJ7, p.PJ8, p.PJ9, p.PJ10, p.PJ11, p.PK0, p.PK1, p.PK2, p.PI15, p.PJ0,
        p.PJ1, p.PJ2, p.PJ3, p.PJ4, p.PJ5, p.PJ6,
    );
    ltdc.init(&ltdc_config);

    let layer_config = LtdcLayerConfig {
        pixel_format: ltdc::PixelFormat::L8,
        layer: LtdcLayer::Layer1,
        window_x0: 0,
        window_x1: DISPLAY_WIDTH as _,
        window_y0: 0,
        window_y1: DISPLAY_HEIGHT as _,
    };

    let clut = build_default_clut();
    ltdc.init_layer(&layer_config, Some(&clut));

    static DISPLAY: StaticCell<LtdcDisplay> = StaticCell::new();
    let display = DISPLAY.init(LtdcDisplay::new(
        ltdc,
        layer_config,
        unsafe { FB1.as_mut() },
        unsafe { FB2.as_mut() },
    ));

    info!("init touch (I2C4 - PD12=SCL, PD13=SDA)");
    let mut i2c_config = I2cConfig::default();
    i2c_config.scl_pullup = true;
    i2c_config.sda_pullup = true;

    let i2c = I2c::new_blocking(
        p.I2C4, p.PD12, // SCL for I2C4
        p.PD13, // SDA for I2C4
        i2c_config,
    );

    let mut touch = Ft5336::new(i2c);
    touch.init();

    info!("creating UI");
    let mut btn1 = Button::new(100, 100, 120, 50, "Click Me!", Rgb888::new(200, 0, 0));
    let mut btn2 = Button::new(260, 100, 120, 50, "Reset", Rgb888::new(0, 200, 0));
    let mut counter = 0i32;

    info!("starting display loop - touch the buttons to interact");

    loop {
        display.clear();

        let title_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(255, 255, 255));
        Text::with_baseline(
            "STM32H7B3 Touch UI",
            Point::new(80, 30),
            title_style,
            Baseline::Alphabetic,
        )
        .draw(display)
        .unwrap();

        let counter_style = MonoTextStyle::new(&FONT_10X20, Rgb888::new(200, 200, 200));
        let mut counter_text = heapless::String::<32>::new();
        use core::fmt::Write;
        write!(&mut counter_text, "Count: {}", counter).unwrap();
        Text::with_baseline(
            counter_text.as_str(),
            Point::new(150, 200),
            counter_style,
            Baseline::Alphabetic,
        )
        .draw(display)
        .unwrap();

        btn1.draw(display).unwrap();
        btn2.draw(display).unwrap();

        match display.swap().await {
            Ok(_) => {}
            Err(e) => info!("Display swap error: {:?}", e),
        }

        // Read touch - only process on actual touch events
        if let Some((tx, ty)) = touch.read_touch() {
            info!("Touch at: x={}, y={}", tx, ty);

            // Check button 1
            if btn1.contains(tx, ty) {
                if !btn1.is_pressed() {
                    btn1.set_pressed(true);
                    counter += 1;
                    info!("Button 1 pressed! Count: {}", counter);
                }
            } else {
                btn1.set_pressed(false);
            }

            // Check button 2
            if btn2.contains(tx, ty) {
                if !btn2.is_pressed() {
                    btn2.set_pressed(true);
                    counter = 0;
                    info!("Button 2 pressed! Counter reset");
                }
            } else {
                btn2.set_pressed(false);
            }
        } else {
            btn1.set_pressed(false);
            btn2.set_pressed(false);
        }

        Timer::after_millis(16).await;
    }
}

fn build_default_clut() -> [ltdc::RgbColor; 256] {
    let mut clut = [ltdc::RgbColor::default(); 256];
    for i in 0..256 {
        let v = i as u8;
        clut[i] = ltdc::RgbColor {
            red: v,
            green: v,
            blue: v,
        };
    }
    clut[0] = ltdc::RgbColor {
        red: 0,
        green: 0,
        blue: 0,
    };
    clut[1] = ltdc::RgbColor {
        red: 255,
        green: 0,
        blue: 0,
    };
    clut[2] = ltdc::RgbColor {
        red: 0,
        green: 255,
        blue: 0,
    };
    clut[3] = ltdc::RgbColor {
        red: 0,
        green: 0,
        blue: 255,
    };
    clut[4] = ltdc::RgbColor {
        red: 255,
        green: 255,
        blue: 0,
    };
    clut[5] = ltdc::RgbColor {
        red: 255,
        green: 0,
        blue: 255,
    };
    clut[6] = ltdc::RgbColor {
        red: 0,
        green: 255,
        blue: 255,
    };
    clut[7] = ltdc::RgbColor {
        red: 255,
        green: 255,
        blue: 255,
    };
    clut
}

#[embassy_executor::task]
async fn led_task(mut led: Output<'static>) {
    loop {
        led.set_low();
        Timer::after_millis(50).await;
        led.set_high();
        Timer::after_millis(450).await;
    }
}

mod rcc_setup {
    use embassy_stm32::rcc::{Hse, HseMode, *};
    use embassy_stm32::time::Hertz;
    use embassy_stm32::{Config, Peripherals};

    pub fn stm32h7b3i_init() -> Peripherals {
        let mut config = Config::default();
        config.rcc.supply_config = SupplyConfig::LDO;
        config.rcc.hse = Some(Hse {
            freq: Hertz::mhz(25),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll1 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul56,
            divp: Some(PllDiv::Div2),
            divq: Some(PllDiv::Div4),
            divr: Some(PllDiv::Div2),
        });
        config.rcc.pll3 = Some(Pll {
            source: PllSource::Hse,
            prediv: PllPreDiv::Div5,
            mul: PllMul::Mul80,
            divp: Some(PllDiv::Div10),
            divq: Some(PllDiv::Div10),
            divr: Some(PllDiv::Div83),
        });
        config.rcc.voltage_scale = VoltageScale::Scale0;
        config.rcc.sys = Sysclk::Pll1P;
        config.rcc.ahb_pre = AHBPrescaler::Div2;
        config.rcc.apb1_pre = APBPrescaler::Div2;
        config.rcc.apb2_pre = APBPrescaler::Div2;
        config.rcc.apb3_pre = APBPrescaler::Div2;
        config.rcc.apb4_pre = APBPrescaler::Div2;
        embassy_stm32::init(config)
    }
}
