use std::io::Result;

fn main() -> Result<()> {
    // read env variables that were set in build script
    let uefi_path = env!("UEFI_PATH");

    let mut cmd = std::process::Command::new("qemu-system-x86_64");
    cmd.arg("-bios").arg(ovmf_prebuilt::ovmf_pure_efi());
    cmd.arg("-drive")
        .arg(format!("format=raw,file={uefi_path}"));
    cmd.arg("-device")
        .arg("isa-debug-exit,iobase=0xf4,iosize=0x04");
    cmd.arg("-serial")
        .arg("stdio");

    let mut child = cmd.spawn()?;
    child.wait()?;
    Ok(())
}
