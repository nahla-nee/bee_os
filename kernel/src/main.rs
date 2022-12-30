#![no_std]
#![no_main]

#![feature(abi_x86_interrupt)]
#![feature(once_cell)]
#![feature(panic_info_message)]

mod arch;
mod display;

use core::panic::PanicInfo;
use core::fmt::Write;
use core::cell::OnceCell;

use bootloader_api::{entry_point, info::Optional, BootInfo};
use display::{TextDisplay, Color};
use display::TEXT_DISPLAY;

/// This function is called on panic.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    if let Some(mut lock) = TEXT_DISPLAY.try_lock() {
        if let Some(text_display) = lock.get_mut() {
            text_display.move_cursor(display::Point(0, 0));
            if let Some(args) = info.message() {
                text_display.set_clear_color(Color(0, 0, 0));
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
    TEXT_DISPLAY.lock().get_or_init(|| {TextDisplay::new(fb, Color(0, 0, 0), Color(255, 255, 0))});
    TEXT_DISPLAY.lock().get_mut().unwrap().clear();
    println!("hello world!");
    loop {
    }
}
