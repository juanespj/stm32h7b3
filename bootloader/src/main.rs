#![no_std]
#![no_main]

use core::arch::asm;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::gpio::{Input, Level, Output, Pull, Speed};
use embassy_stm32::usart::{Config, Uart};
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};

const APP_START: u32 = 0x08010000;
const APP_END: u32 = 0x08200000;

const FLASH_BASE: u32 = 0x52002000;
const FLASH_CR: u32 = FLASH_BASE + 0x0C;
const FLASH_SR: u32 = FLASH_BASE + 0x10;
const FLASH_KEYR: u32 = FLASH_BASE + 0x04;

const FLASH_KEY1: u32 = 0x45670123;
const FLASH_KEY2: u32 = 0xCDEF89AB;

const FLASH_CR_PG: u32 = 1 << 1;
const FLASH_CR_SER: u32 = 1 << 2;
const FLASH_CR_PSIZE_32: u32 = 1 << 4;
const FLASH_CR_START: u32 = 1 << 5;
const FLASH_CR_LOCK: u32 = 1 << 0;

const FLASH_SR_BSY: u32 = 1 << 0;
const FLASH_SR_WRPERR: u32 = 1 << 17;
const FLASH_SR_PGAERR: u32 = 1 << 22;

const SECTOR_SIZE: usize = 128 * 1024;
const PACKET_LEN: usize = 256;
const SOF: u8 = 0x01;
const ACK: u8 = 0x06;
const NACK: u8 = 0x15;

#[embassy_executor::main]
async fn main(_spawner: Spawner) -> ! {
    info!("Bootloader v0.1");

    let mut config = embassy_stm32::Config::default();
    config.rcc.supply_config = embassy_stm32::rcc::SupplyConfig::LDO;
    let p = embassy_stm32::init(config);

    let mut led = Output::new(p.PG13, Level::Low, Speed::Low);
    let button = Input::new(p.PA0, Pull::Up);

    led.set_high();
    Timer::after_millis(100).await;

    let force_update = !button.is_high();

    if !force_update && is_app_valid(APP_START) {
        info!("Launching app at {:08x}", APP_START);
        unsafe { jump_to_app(APP_START); }
    }

    info!("Update mode - PA0 held or no valid app");
    blink_led(&mut led, 3).await;
    info!("Waiting for firmware via UART7 (PA8=RX, PA15=TX, 115200 8N1)");

    let mut uart_cfg = Config::default();
    uart_cfg.baudrate = 115200;
    let mut uart = Uart::new_blocking(p.UART7, p.PA8, p.PA15, uart_cfg).unwrap();

    loop {
        if receive_firmware(&mut uart, &mut led).await.is_ok() {
            blink_led(&mut led, 5).await;
            if is_app_valid(APP_START) {
                info!("Launch!");
                unsafe { jump_to_app(APP_START); }
            } else {
                error!("Invalid app after flash");
            }
        } else {
            error!("Transfer failed");
            blink_led(&mut led, 10).await;
        }
    }
}

fn is_app_valid(addr: u32) -> bool {
    let sp = unsafe { core::ptr::read_volatile(addr as *const u32) };
    let rv = unsafe { core::ptr::read_volatile((addr + 4) as *const u32) };
    (sp & 0xFF000000 == 0x20000000 || sp & 0xFF000000 == 0x24000000)
        && (rv & 1) == 1
        && rv >= APP_START
        && rv < APP_END
}

unsafe fn jump_to_app(addr: u32) -> ! {
    let sp = core::ptr::read_volatile(addr as *const u32);
    let rv = core::ptr::read_volatile((addr + 4) as *const u32) & !1;

    let scb = &*cortex_m::peripheral::SCB::PTR;
    scb.vtor.write(addr);

    asm!("msr msp, {0}", in(reg) sp);
    asm!("msr control, {0}", in(reg) 0, options(nomem, nostack));
    asm!("isb");
    asm!("bx {0}", in(reg) rv, options(noreturn));
}

fn flash_unlock() {
    unsafe {
        core::ptr::write_volatile(FLASH_KEYR as *mut u32, FLASH_KEY1);
        core::ptr::write_volatile(FLASH_KEYR as *mut u32, FLASH_KEY2);
    }
}

fn flash_lock() {
    unsafe {
        let v = core::ptr::read_volatile(FLASH_CR as *mut u32);
        core::ptr::write_volatile(FLASH_CR as *mut u32, v | FLASH_CR_LOCK);
    }
}

fn flash_wait() {
    while unsafe { core::ptr::read_volatile(FLASH_SR as *mut u32) } & FLASH_SR_BSY != 0 {}
}

fn erase_sector(n: usize) -> Result<(), &'static str> {
    if n > 7 {
        return Err("bad sector");
    }
    flash_unlock();
    unsafe {
        core::ptr::write_volatile(
            FLASH_CR as *mut u32,
            FLASH_CR_SER | FLASH_CR_PSIZE_32 | ((n as u32) << 8),
        );
        core::ptr::write_volatile(FLASH_CR as *mut u32, FLASH_CR_START);
    }
    flash_wait();
    unsafe {
        let sr = core::ptr::read_volatile(FLASH_SR as *mut u32);
        core::ptr::write_volatile(FLASH_CR as *mut u32, 0);
        if sr & FLASH_SR_WRPERR != 0 {
            flash_lock();
            return Err("wrperr");
        }
    }
    flash_lock();
    Ok(())
}

fn write_word(addr: u32, w: u32) -> Result<(), &'static str> {
    flash_unlock();
    unsafe {
        core::ptr::write_volatile(FLASH_CR as *mut u32, FLASH_CR_PG | FLASH_CR_PSIZE_32);
        core::ptr::write_volatile(addr as *mut u32, w);
    }
    flash_wait();
    unsafe {
        let sr = core::ptr::read_volatile(FLASH_SR as *mut u32);
        core::ptr::write_volatile(FLASH_CR as *mut u32, 0);
        if sr & (FLASH_SR_WRPERR | FLASH_SR_PGAERR) != 0 {
            flash_lock();
            return Err("progerr");
        }
    }
    flash_lock();
    Ok(())
}

async fn receive_firmware(
    uart: &mut Uart<'static, embassy_stm32::mode::Blocking>,
    led: &mut Output<'static>,
) -> Result<usize, &'static str> {
    uart.blocking_write(b"READY\n").map_err(|_| "write")?;

    let mut total = 0;
    let mut current_sector: Option<usize> = None;
    let mut pkt_buf = [0u8; PACKET_LEN];

    loop {
        let b = uart_read_byte(uart)?;

        if b == SOF {
            uart.blocking_write(&[ACK]).map_err(|_| "write")?;

            let len = uart_read_byte(uart)?;
            if len != PACKET_LEN as u8 {
                uart.blocking_write(&[NACK]).ok();
                return Err("bad len");
            }

            let seq = uart_read_byte(uart)?;
            let seq_inv = uart_read_byte(uart)?;
            if seq as u16 + seq_inv as u16 != 0xFF {
                uart.blocking_write(&[NACK]).ok();
                return Err("bad seq");
            }

            let mut chksum: u8 = 0;
            for i in 0..PACKET_LEN {
                pkt_buf[i] = uart_read_byte(uart)?;
                chksum = chksum.wrapping_add(pkt_buf[i]);
            }

            let expected = uart_read_byte(uart)?;
            if chksum != expected {
                uart.blocking_write(&[NACK]).ok();
                return Err("bad chk");
            }

            let sector = total / SECTOR_SIZE;
            if current_sector != Some(sector) {
                erase_sector(sector)?;
                current_sector = Some(sector);
                led.toggle();
            }

            let mut i = 0;
            while i + 4 <= PACKET_LEN {
                let w = u32::from_le_bytes([pkt_buf[i], pkt_buf[i + 1], pkt_buf[i + 2], pkt_buf[i + 3]]);
                let addr = APP_START + (total + i) as u32;
                if addr >= APP_END {
                    uart.blocking_write(&[NACK]).ok();
                    return Err("overflow");
                }
                write_word(addr, w)?;
                i += 4;
            }

            total += PACKET_LEN;
            uart.blocking_write(&[ACK]).map_err(|_| "write")?;
        } else if b == 0x04 {
            info!("Done: {} bytes", total);
            return Ok(total);
        } else if b == 0x18 {
            uart.blocking_write(&[ACK]).ok();
            return Err("abort");
        }
    }
}

fn uart_read_byte(uart: &mut Uart<'static, embassy_stm32::mode::Blocking>) -> Result<u8, &'static str> {
    let mut buf = [0u8; 1];
    uart.blocking_read(&mut buf).map_err(|_| "read")?;
    Ok(buf[0])
}

async fn blink_led(led: &mut Output<'static>, n: usize) {
    for _ in 0..n {
        led.set_high();
        Timer::after(Duration::from_millis(80)).await;
        led.set_low();
        Timer::after(Duration::from_millis(80)).await;
    }
}
