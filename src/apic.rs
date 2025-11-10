use core::arch::asm;
use pic8259::ChainedPics;
use x86_64::instructions::port::Port;
use x86_64::registers::model_specific::Msr;
pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });
pub static mut APIC_BASE: usize = 0;
fn cpuid(eax: u32) -> (u32, u32, u32) {
    let (mut eax_out, mut ecx, mut edx): (u32, u32, u32);
    unsafe {
        asm!(
            "cpuid",
            inlateout("eax") eax => eax_out,
            lateout("ecx") ecx,
            lateout("edx") edx,
        );
    }
    (eax_out, ecx, edx)
}
pub fn has_apic() -> bool {
    let (_, _, edx) = cpuid(1);
    (edx & (1 << 9)) != 0
}

fn init_pics() {
    unsafe {
        PICS.lock().initialize();
    }
}

fn disable_pics() {
    unsafe {
        PICS.lock().disable();
    }
}
fn enable_APIC() {
    let mut apic_base_msr = Msr::new(0x1B);
    unsafe {
        let value = apic_base_msr.read();
        apic_base_msr.write(value | (1 << 11));
        let apic_base = (value & 0xFFFF_FFFF_F000) as usize;
        APIC_BASE = apic_base;
        write_apic_register(apic_base, 0xF0, 0x1FF);
        write_apic_register(apic_base, 0x350, 1 << 16);
        write_apic_register(apic_base, 0x360, 0x400);
        write_apic_register(apic_base, 0x370, 0x33);
        write_apic_register(apic_base, 0x080, 0);
        timer_config();
    }
}
fn timer_config() {
    unsafe {
        write_apic_register(APIC_BASE, 0x3E0, 0xB);
        write_apic_register(APIC_BASE, 0x320, 0x20020);
        write_apic_register(APIC_BASE, 0x380, 0x100000);
    }
}
unsafe fn read_apic_register(apic_base: usize, offset: usize) -> u32 {
    let apic_base = (apic_base & 0xFFFF_FFFF_F000) as *const u32;
    unsafe { apic_base.add(offset / 4).read_volatile() }
}
pub unsafe fn write_apic_register(apic_base: usize, offset: usize, value: u32) {
    let apic_base = (apic_base & 0xFFFF_FFFF_F000) as *mut u32;

    unsafe {
        apic_base.add(offset / 4).write_volatile(value);
    }
}

pub fn init() {
    init_pics();
    disable_pics();
    enable_APIC();
}
