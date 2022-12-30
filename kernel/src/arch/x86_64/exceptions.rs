use spinning_top::Spinlock;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

use core::cell::LazyCell;

static IDT: Spinlock<LazyCell<InterruptDescriptorTable>> = Spinlock::new(LazyCell::new(|| {
    let mut idt = InterruptDescriptorTable::new();
    idt.breakpoint.set_handler_fn(breakpoint_handler);
    idt
}));

pub fn init_idt() {
}

extern "x86-interrupt" fn breakpoint_handler(
    stack_frame: InterruptStackFrame) {
}