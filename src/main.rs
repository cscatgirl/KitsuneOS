#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
mod apic;
mod console;
mod framebuffer;
pub mod gdt;
mod interupts;
mod keyboard;
mod psfparser;
use core::panic::PanicInfo;
use framebuffer::{FrameBuffer, FrameBufferInfo};
use keyboard::Keyboard;
use psfparser::psffont;
use uefi::boot::*;
use uefi::mem::memory_map::{MemoryMap, MemoryType};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
static FONT_DATA: &[u8] = include_bytes!("../fonts/Lat2-Terminus16.psfu");
use spin::Once;

use crate::apic::has_apic;
static FONT: Once<psffont> = Once::new();
static BLACK: u32 = 0x000000;
pub fn init() {
    gdt::init();
    interupts::init_idt();
    apic::init();
    let mut keyboard = Keyboard::new();
    keyboard.init();
    x86_64::instructions::interrupts::enable();
}
pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}
#[entry]
fn efi_main() -> Status {
    let gop_handle = boot::get_handle_for_protocol::<GraphicsOutput>().expect("Cannot load");
    let mut gop =
        boot::open_protocol_exclusive::<GraphicsOutput>(gop_handle).expect("Cannot get gop");
    let mode_info = gop.current_mode_info();
    let mut framebuff_raw = gop.frame_buffer();
    let frame_info = FrameBufferInfo {
        addr: framebuff_raw.as_mut_ptr() as usize,
        size: framebuff_raw.size(),
        width: mode_info.resolution().0,
        height: mode_info.resolution().1,
        stride: mode_info.stride(),
        pixel_format: mode_info.pixel_format(),
    };
    let mmap = unsafe { exit_boot_services(Some(MemoryType::LOADER_DATA)) };

    kernel_main(mmap, frame_info);
}

fn kernel_main(mmap: uefi::mem::memory_map::MemoryMapOwned, fbinfo: FrameBufferInfo) -> ! {
    let fb = FrameBuffer::new(fbinfo);
    let font = match psffont::parse(FONT_DATA) {
        Ok(f) => f,
        Err(_) => loop {},
    };
    FONT.call_once(|| font);
    let font_ref = FONT.get().unwrap();
    console::Console::init(fb, font_ref);
    let mut usable_ram = 0;
    let mut reserved_ram = 0;
    for desc in mmap.entries() {
        let mem = desc.page_count * 4096;
        match desc.ty {
            MemoryType::CONVENTIONAL => usable_ram += mem,
            MemoryType::BOOT_SERVICES_CODE | MemoryType::BOOT_SERVICES_DATA => usable_ram += mem,
            MemoryType::RESERVED => reserved_ram += mem,
            _ => {}
        }
    }

    println!("total usable ram: {}", usable_ram / (1024 * 1024));
    println!("total reserved ram: {}", reserved_ram / (1024 * 1024));
    println!("total ram: {}", (reserved_ram + usable_ram) / (1024 * 1024));
    println!("=== KitsuneOS Boot ===");
    println!();
    println!();
    println!();
    println!("[OK] PSF font loaded successfully");
    println!("[OK] Console initialized");
    init();
    println!();

    println!("===Welcome to KitsuneOS!===");

    hlt_loop();
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);
    hlt_loop();
}
