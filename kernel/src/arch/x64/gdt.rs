
use spin::Lazy;
use x86_64::VirtAddr;
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{CS, Segment, DS, SS, ES};
use x86_64::structures::gdt::{GlobalDescriptorTable, Descriptor, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;

struct Selectors {
    code_selector: SegmentSelector,
    tss_selector: SegmentSelector,
}

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static TSS: Lazy<TaskStateSegment> = Lazy::new(|| {
    let mut tss = TaskStateSegment::new();
    tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
        const STACK_SIZE: usize = 4096 * 5;
        static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

        let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
        let stack_end = stack_start + STACK_SIZE;
        stack_end
    };
    tss
});

static GDT: Lazy<(GlobalDescriptorTable, Selectors)> = Lazy::new(|| {
    let mut gdt = GlobalDescriptorTable::new();
    let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
    let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
    (gdt, Selectors { code_selector, tss_selector })
});

pub fn init_gdt() {
    GDT.0.load();
    unsafe {
        DS::set_reg(x86_64::registers::segmentation::SegmentSelector(0));
        SS::set_reg(x86_64::registers::segmentation::SegmentSelector(0));
        ES::set_reg(x86_64::registers::segmentation::SegmentSelector(0));
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }
}