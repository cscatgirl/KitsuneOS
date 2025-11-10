use lazy_static::lazy_static;
use x86_64::VirtAddr;
use x86_64::instructions::segmentation::{CS, Segment};
use x86_64::instructions::tables::load_tss;
use x86_64::registers::segmentation::{DS, ES, SS};
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 * 5;
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            let stack_start = VirtAddr::from_ptr(&raw const STACK);
            let stack_end = stack_start + STACK_SIZE as u64;
            stack_end
        };
        tss
    };
}

lazy_static! {
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_select = gdt.append(Descriptor::kernel_code_segment());
        let data_selector = gdt.append(Descriptor::kernel_data_segment());
        let tss_select = gdt.append(Descriptor::tss_segment(&TSS));
        (
            gdt,
            Selectors {
                code_select,
                data_selector,
                tss_select,
            },
        )
    };
}
struct Selectors {
    code_select: SegmentSelector,
    data_selector: SegmentSelector,
    tss_select: SegmentSelector,
}
pub fn init() {
    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code_select);
        SS::set_reg(GDT.1.data_selector);
        DS::set_reg(GDT.1.data_selector);
        ES::set_reg(GDT.1.data_selector);
        load_tss(GDT.1.tss_select);
    }
}
