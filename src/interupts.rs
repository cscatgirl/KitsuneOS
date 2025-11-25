use crate::apic::APIC_BASE;
use crate::apic::write_apic_register;
use crate::gdt;
use crate::hlt_loop;
use crate::keyboard::handle_scancode;
use crate::{print, println};
use lazy_static::lazy_static;
use x86_64::registers::control::Cr2;
use x86_64::structures::idt::PageFaultErrorCode;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};
lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(bkpoint_handler);
        idt.divide_error.set_handler_fn(divbyzero);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }

        idt[0xFF].set_handler_fn(spurious_interrupt_handler);
        idt[0x33].set_handler_fn(levt_error_handler);
        idt[32].set_handler_fn(timer_interup_handler);
        idt[33].set_handler_fn(keyboard_handler);
        idt.page_fault.set_handler_fn(page_fault_handler);
        idt
    };
}
pub fn init_idt() {
    IDT.load();
}
extern "x86-interrupt" fn bkpoint_handler(stackframe: InterruptStackFrame) {
    println!("Invoked Breakpoint {:#?}", stackframe);
}
extern "x86-interrupt" fn divbyzero(stackframe: InterruptStackFrame) {
    println!("You fucked up{:#?}", stackframe);
}
extern "x86-interrupt" fn double_fault(stackframe: InterruptStackFrame, eror_code: u64) -> ! {
    panic!("You fucked up and caused a doubled fault{:#?}", stackframe);
}
extern "x86-interrupt" fn spurious_interrupt_handler(stack: InterruptStackFrame) {
    println!("Triggered spurious_interrupt_handler{:#?}", stack);
}
extern "x86-interrupt" fn levt_error_handler(stackframe: InterruptStackFrame) {
    println!("Triggered levt_error_handler{:#?}", stackframe);
    unsafe {
        write_apic_register(APIC_BASE, 0x0B0, 0);
    }
}
extern "x86-interrupt" fn timer_interup_handler(stackframe: InterruptStackFrame) {
    //print!("Tick");
    unsafe {
        write_apic_register(APIC_BASE, 0x0B0, 0);
    }
}
extern "x86-interrupt" fn keyboard_handler(stackframe: InterruptStackFrame) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut data_port = Port::<u8>::new(0x60);
        let scancode = data_port.read();
        handle_scancode(scancode);
        // Send EOI to APIC
        write_apic_register(APIC_BASE, 0x0B0, 0);
    }
}
extern "x86-interrupt" fn page_fault_handler(
    stackframe: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    let fault_addr = Cr2::read();
    println!("Page Fault at {:#x}", fault_addr.unwrap().as_u64());
    println!("Error code {:?}", error_code);
    println!("Stact Frame {:#?}", stackframe);
    hlt_loop();
}
