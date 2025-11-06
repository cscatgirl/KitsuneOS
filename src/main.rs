#![no_std]
#![no_main]
mod console;
mod framebuffer;
mod psfparser;
use core::panic::PanicInfo;
use framebuffer::{FrameBuffer, FrameBufferInfo};
use psfparser::psffont;
use uefi::boot::*;
use uefi::mem::memory_map::MemoryType;
use uefi::prelude::*;
use uefi::proto::console::gop::GraphicsOutput;
static FONT_DATA: &[u8] = include_bytes!("../fonts/Lat2-Terminus16.psfu");
use spin::Once;
static FONT: Once<psffont> = Once::new();
static BLACK: u32 = 0x000000;
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

fn kernel_main(_mmap: uefi::mem::memory_map::MemoryMapOwned, fbinfo: FrameBufferInfo) -> ! {
    let fb = FrameBuffer::new(fbinfo);

    let font = match psffont::parse(FONT_DATA) {
        Ok(f) => f,
        Err(_) => loop {},
    };
    FONT.call_once(|| font);
    let font_ref = FONT.get().unwrap();
    console::Console::init(fb, font_ref);
    println!("=== KitsuneOS Boot ===");
    println!();

    println!("[OK] PSF font loaded successfully");
    println!("[OK] Console initialized");
    println!();

    println!("Welcome to KitsuneOS!");
    println!("Running on UEFI with framebuffer graphics");
    println!();

    // Test formatting
    println!();

    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!();
    println!("!!! KERNEL PANIC !!!");
    println!("{}", info);
    loop {}
}
