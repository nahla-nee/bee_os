#[cfg(feature = "x86_64")]
mod x86_64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    #[cfg(feature = "x86_64")]
    x86_64::exit_qemu(exit_code);
}
