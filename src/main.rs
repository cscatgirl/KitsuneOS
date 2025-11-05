#![no_std]
#![no_main]
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
static BLACK: u32 = 0x000000;
#[entry]
fn efi_main() -> Status {
    // Exit boot services immediately for maximum control
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
    let mut fb = FrameBuffer::new(fbinfo);
    let font = match psffont::parse(FONT_DATA) {
        Ok(f) => f,
        Err(_) => {
            fb.clear_screen(BLACK);
            loop {}
        }
    };
    fb.clear_screen(BLACK);
    fb.write_string("Is This Working ", 10, 10, 0x00FF00, &font);
    fb.write_string("KitsuneOS is booting...", 10, 30, 0x00FF00, &font);
    fb.write_string(
        "Wow the framebuffer actually has been written",
        10,
        50,
        0x00FF00,
        &font,
    );
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
