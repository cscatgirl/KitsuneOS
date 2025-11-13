#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]
mod apic;
mod console;
mod framebuffer;
pub mod gdt;
mod interupts;
mod keyboard;
mod memory;
mod psfparser;
use core::panic::PanicInfo;
use core::u64;
use framebuffer::{FrameBuffer, FrameBufferInfo};
use keyboard::Keyboard;
use psfparser::psffont;
use uefi::boot::*;
use uefi::mem::memory_map::{MemoryMap, MemoryType};
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
static FONT_DATA: &[u8] = include_bytes!("../fonts/Lat2-Terminus16.psfu");
use spin::Once;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};

use crate::apic::has_apic;
use crate::memory::{BitmapFrameAllocator, init_frame_allocator, with_frame_allocator};
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
pub fn find_largest_region(mmap: &uefi::mem::memory_map::MemoryMapOwned) -> Option<(u64, u64)> {
    let mut largest_start_addr = 0;
    let mut largest_size = 0;
    for desc in mmap.entries() {
        if desc.ty == MemoryType::CONVENTIONAL {
            let size = desc.page_count * 4096;
            if size > largest_size {
                largest_size = size;
                largest_start_addr = desc.phys_start;
            }
        }
    }
    if largest_size > 0 {
        Some((largest_start_addr, largest_size))
    } else {
        None
    }
}
pub fn handle_memory(mmap: &uefi::mem::memory_map::MemoryMapOwned) {
    let mut total_pages = 0;
    for desc in mmap.entries() {
        total_pages += desc.page_count
    }
    let bitmap_field_size = (total_pages + 63) / 64;
    let bitmap_size = bitmap_field_size * 8;

    let (region_start, region_size) = match find_largest_region(mmap) {
        Some(region) => region,
        None => {
            println!("ERROR: Failed to find suitable memory region for allocator");
            return;
        }
    };

    if region_size < bitmap_size {
        println!("ERROR: Largest memory region too small for bitmap allocator");
        return;
    }

    let bitmap_ptr = region_start as *mut u64;
    unsafe {
        for i in 0..bitmap_field_size {
            bitmap_ptr.add(i as usize).write(u64::MAX);
        }
    }

    let mut allocator =
        BitmapFrameAllocator::new(bitmap_ptr, bitmap_field_size as usize, total_pages as usize);

    for desc in mmap.entries() {
        if desc.ty == MemoryType::CONVENTIONAL
            || desc.ty == MemoryType::BOOT_SERVICES_CODE
            || desc.ty == MemoryType::BOOT_SERVICES_DATA
        {
            let start_frame = (desc.phys_start / 4096) as usize;
            let frame_count = desc.page_count as usize;
            allocator.mark_range_free(start_frame, frame_count);
        }
    }

    let bitmap_frames = (bitmap_size + 4095) / 4096;
    let bitmap_start_frame = (region_start / 4096) as usize;
    allocator.mark_range_used(bitmap_start_frame, bitmap_frames as usize);

    init_frame_allocator(allocator);
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
    handle_memory(&mmap);
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
