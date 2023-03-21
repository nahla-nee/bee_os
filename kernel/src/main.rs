#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(abi_x86_interrupt)]
#![feature(once_cell)]
#![feature(panic_info_message)]

mod arch;
mod display;
mod serial;

use core::fmt::Write;
use core::panic::PanicInfo;

use uart_16550::SerialPort;
use bootloader_api::{entry_point, info::Optional, BootInfo};

use serial::DEBUG_SERIAL;
use display::TEXT_DISPLAY;
use display::{Color, TextDisplay};

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(mut lock) = TEXT_DISPLAY.try_lock() {
        if let Some(text_display) = lock.get_mut() {
            if let Some(args) = info.message() {
                text_display.set_clear_color(Color(0, 0, 0));
                text_display.set_text_color(Color(255, 0, 0));
                write!(text_display, "{}", args).unwrap();
            }
        }
    }
    if let Some(mut lock) = DEBUG_SERIAL.try_lock() {
        if let Some(debug_serial) = lock.get_mut() {
            if let Some(&args) = info.message() {
                let _ = debug_serial.write_fmt(args);
                let _ = debug_serial.write_char('\n');
            }
        }
    }

    loop {}
}

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let fb = match &mut boot_info.framebuffer {
        Optional::Some(fb) => fb,
        Optional::None => panic!(),
    };

    TEXT_DISPLAY
        .lock()
        .get_or_init(|| TextDisplay::new(fb, Color(0, 0, 0), Color(255, 255, 0)));
    DEBUG_SERIAL
        .lock()
        .get_or_init(|| {
            let mut serial = unsafe { SerialPort::new(0x3F8) };
            serial.init();
            serial
        });

    clearscrn!();
    log("Booting into BeeOS");

    arch::init();
    log("x86_64 initialized");

    loop {}
}

fn log(message: &str) {
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        if let Some(mut lock) = TEXT_DISPLAY.try_lock() {
            if let Some(text_display) = lock.get_mut() {
                write!(text_display, "{}\n", message).unwrap();
            }
        }
    });
    if let Some(mut lock) = DEBUG_SERIAL.try_lock() {
        if let Some(debug_serial) = lock.get_mut() {
            let _ = debug_serial.write_fmt(format_args!("{}\n", message));
        }
    }
}