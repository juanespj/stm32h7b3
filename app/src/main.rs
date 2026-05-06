#![no_std]
#![no_main]
#![macro_use]
#![allow(static_mut_refs)]

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embassy_stm32::ltdc::{self, Ltdc, LtdcConfiguration, LtdcLayer, LtdcLayerConfig, PolarityActive, PolarityEdge};
use embassy_stm32::{bind_interrupts, peripherals};
use embassy_time::{Duration, Timer};
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::prelude::*;
use embedded_graphics::primitives::{PrimitiveStyleBuilder, Rectangle};
use embedded_graphics::primitives::StyledDrawable;
use {defmt_rtt as _, panic_probe as _};

const DISPLAY_WIDTH: usize = 480;
const DISPLAY_HEIGHT: usize = 272;
const MY_TASK_POOL_SIZE: usize = 2;

pub static mut FB1: [TargetPixelType; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];
pub static mut FB2: [TargetPixelType; DISPLAY_WIDTH * DISPLAY_HEIGHT] = [0; DISPLAY_WIDTH * DISPLAY_HEIGHT];

bind_interrupts!(struct Irqs {
    LTDC => ltdc::InterruptHandler<peripherals::LTDC>;
});

const NUM_COLORS: usize = 256;

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
        p.LTDC, Irqs,
        p.PI14, p.PI12, p.PI13, p.PK7,
        p.PJ12, p.PJ13, p.PJ14, p.PJ15, p.PK3,
        p.PK4, p.PK5, p.PK6,
        p.PJ7, p.PJ8, p.PJ9, p.PJ10, p.PJ11,
        p.PK0, p.PK1, p.PK2,
        p.PI15, p.PJ0, p.PJ1, p.PJ2, p.PJ3,
        p.PJ4, p.PJ5, p.PJ6,
    );
    ltdc.init(&ltdc_config);

    info!("enable bottom layer");
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

    let mut double_buffer = DoubleBuffer::new(
        unsafe { FB1.as_mut() },
        unsafe { FB2.as_mut() },
        layer_config,
    );

    info!("starting display loop");

    let mut x = 0i32;
    let mut y = 0i32;
    let mut dx = 2i32;
    let mut dy = 2i32;
    let box_w = 80i32;
    let box_h = 60i32;
    let mut hue = 0u8;

    loop {
        double_buffer.clear();

        let rect = Rectangle::new(Point::new(x, y), Size::new(box_w as u32, box_h as u32));
        let color = hsv_to_rgb(hue);
        let style = PrimitiveStyleBuilder::new()
            .fill_color(color)
            .build();
        rect.draw_styled(&style, &mut double_buffer).unwrap();

        x += dx;
        y += dy;
        if x <= 0 || x + box_w >= DISPLAY_WIDTH as i32 { dx = -dx; hue = hue.wrapping_add(30); }
        if y <= 0 || y + box_h >= DISPLAY_HEIGHT as i32 { dy = -dy; hue = hue.wrapping_add(30); }

        unwrap!(double_buffer.swap(&mut ltdc).await);
        Timer::after_millis(16).await;
    }
}

fn build_default_clut() -> [ltdc::RgbColor; NUM_COLORS] {
    let mut clut = [ltdc::RgbColor::default(); NUM_COLORS];
    for i in 0..NUM_COLORS {
        let v = i as u8;
        clut[i] = ltdc::RgbColor { red: v, green: v, blue: v };
    }
    clut[0] = ltdc::RgbColor { red: 0, green: 0, blue: 0 };
    clut[1] = ltdc::RgbColor { red: 255, green: 0, blue: 0 };
    clut[2] = ltdc::RgbColor { red: 0, green: 255, blue: 0 };
    clut[3] = ltdc::RgbColor { red: 0, green: 0, blue: 255 };
    clut[4] = ltdc::RgbColor { red: 255, green: 255, blue: 0 };
    clut[5] = ltdc::RgbColor { red: 255, green: 0, blue: 255 };
    clut[6] = ltdc::RgbColor { red: 0, green: 255, blue: 255 };
    clut[7] = ltdc::RgbColor { red: 255, green: 255, blue: 255 };
    clut
}

fn hsv_to_rgb(h: u8) -> Rgb888 {
    let h = h as u16 * 6;
    let s = 200u16;
    let v = 200u16;
    let region = h / 60;
    let remainder = (h % 60) * 255 / 60;
    let p = v * (255 - s) / 255;
    let q = v * (255 - s * remainder / 255) / 255;
    let t = v * (255 - s * (255 - remainder) / 255) / 255;
    let (r, g, b) = match region {
        0 => (v, t, p),
        1 => (q, v, p),
        2 => (p, v, t),
        3 => (p, q, v),
        4 => (t, p, v),
        _ => (v, p, q),
    };
    Rgb888::new(r as u8, g as u8, b as u8)
}

#[embassy_executor::task(pool_size = MY_TASK_POOL_SIZE)]
async fn led_task(mut led: Output<'static>) {
    let mut counter = 0;
    loop {
        info!("blink: {}", counter);
        counter += 1;
        led.set_low();
        Timer::after(Duration::from_millis(50)).await;
        led.set_high();
        Timer::after(Duration::from_millis(450)).await;
    }
}

pub type TargetPixelType = u8;

pub struct DoubleBuffer {
    buf0: &'static mut [TargetPixelType],
    buf1: &'static mut [TargetPixelType],
    is_buf0: bool,
    layer_config: LtdcLayerConfig,
}

impl DoubleBuffer {
    pub fn new(
        buf0: &'static mut [TargetPixelType],
        buf1: &'static mut [TargetPixelType],
        layer_config: LtdcLayerConfig,
    ) -> Self {
        Self {
            buf0,
            buf1,
            is_buf0: true,
            layer_config,
        }
    }

    pub fn current(&mut self) -> &mut [TargetPixelType] {
        if self.is_buf0 { self.buf0 } else { self.buf1 }
    }

    pub async fn swap<T: ltdc::Instance>(&mut self, ltdc: &mut Ltdc<'_, T>) -> Result<(), ltdc::Error> {
        let buf = self.current();
        let frame_buffer = buf.as_ptr();
        self.is_buf0 = !self.is_buf0;
        ltdc.set_buffer(self.layer_config.layer, frame_buffer as *const _).await
    }

    pub fn clear(&mut self) {
        let buf = self.current();
        for a in buf.iter_mut() {
            *a = 0;
        }
    }
}

impl DrawTarget for DoubleBuffer {
    type Color = Rgb888;
    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        let size = self.size();
        let width = size.width as i32;
        let height = size.height as i32;
        let buf = self.current();

        for pixel in pixels {
            let Pixel(point, color) = pixel;
            if point.x >= 0 && point.y >= 0 && point.x < width && point.y < height {
                let index = (point.y * width + point.x) as usize;
                let r = color.r();
                let g = color.g();
                let b = color.b();
                buf[index] = closest_palette_index(r, g, b);
            }
        }
        Ok(())
    }
}

impl OriginDimensions for DoubleBuffer {
    fn size(&self) -> Size {
        Size::new(
            (self.layer_config.window_x1 - self.layer_config.window_x0) as _,
            (self.layer_config.window_y1 - self.layer_config.window_y0) as _,
        )
    }
}

fn closest_palette_index(r: u8, g: u8, b: u8) -> u8 {
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
