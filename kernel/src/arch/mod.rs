#[cfg(feature = "x64")]
mod x64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    #[cfg(feature = "x64")]
    x64::exit_qemu(exit_code);
}

#[cfg(feature = "x64")]
pub fn init() {
    use crate::log;

    log("Initializing GDT");
    x64::gdt::init_gdt();
    log("GDT initialized");

    log("Initializing IDT");
    x64::interrupts::init_idt();
    log("IDT initialized");

    log("Initializing PIC8529");
    x64::interrupts::init_pic8529();
    log("PIC8529 initialized");

    log("Enabling interrupts");
    x86_64::instructions::interrupts::enable();
    log("Interrupts enabled");
}