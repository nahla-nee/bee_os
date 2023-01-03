#![no_std]
#![no_main]
#![feature(let_chains)]
#![feature(abi_x86_interrupt)]
#![feature(once_cell)]
#![feature(panic_info_message)]

mod arch;
mod display;

use core::cell::OnceCell;
use core::fmt::Write;
use core::panic::PanicInfo;

use bootloader_api::{entry_point, info::Optional, BootInfo};
use display::TEXT_DISPLAY;
use display::{Color, TextDisplay};

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(mut lock) = TEXT_DISPLAY.try_lock() {
        if let Some(text_display) = lock.get_mut() {
            text_display.move_cursor(display::Point(0, 0));
            if let Some(args) = info.message() {
                text_display.set_clear_color(Color(0, 0, 0));
                text_display.clear();
                text_display.set_text_color(Color(255, 0, 0));
                write!(text_display, "{}", args).unwrap();
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

    clearscrn!();
    println!("Booting into BeeOS.");

    arch::interrupts::init();
    println!("Interrupts initialized.");

    loop {}
}
