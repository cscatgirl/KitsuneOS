use uefi::mem::memory_map::MemoryMap;
use uefi::mem::memory_map::MemoryType;
use x86_64::PhysAddr;
use x86_64::instructions::tlb::flush;
use x86_64::registers::control::Cr3;
use x86_64::structures::paging::Mapper;
use x86_64::structures::paging::mapper::MapToError;
use x86_64::structures::paging::{
    FrameAllocator, Page, PageTable, PageTableFlags, PhysFrame, Size4KiB,
};
use x86_64::{VirtAddr, structures::paging::OffsetPageTable};

use crate::memory::FrameAllocatorWrapper;
use crate::println;

pub unsafe fn init(frame_offset: VirtAddr) {}

pub fn map_physical_to_virtual(mmap: &uefi::mem::memory_map::MemoryMapOwned) {
    println!("[DEBUG] Starting map_physical_to_virtual");
    let mem_offset = VirtAddr::new(0);
    println!("[DEBUG] Creating OffsetPageTable");
    let mut mem_map = unsafe { init(mem_offset) };
    println!("[DEBUG] OffsetPageTable created");
    let mut frame_allocator = FrameAllocatorWrapper;

    let mut mapped_count = 0;
    let mut skipped_huge = 0;
    let mut skipped_already = 0;
    let mut region_count = 0;
    let mut frame_allocator = FrameAllocatorWrapper;
    let pm4_frame = frame_allocator.allocate_frame().expect("failed to alloc");
    let phys_addr = pm4_frame.start_address().as_u64();
    println!("[DEBUG] Allocated PML4 at physical: 0x{:x}", phys_addr);
    let pm4_frame_ptr = pm4_frame.start_address().as_u64() as *mut PageTable;
    unsafe {
        core::ptr::write_bytes(pm4_frame_ptr as *mut u8, 0, 4096);
    }
    let pm4 = unsafe { &mut *pm4_frame_ptr };
    let mut mem_map = unsafe { OffsetPageTable::new(pm4, VirtAddr::new(0)) };

    for desc in mmap.entries() {
        region_count += 1;
        println!(
            "[DEBUG] Processing region {} at 0x{:x}, {} pages",
            region_count, desc.phys_start, desc.page_count
        );

        let start_addr = desc.phys_start;
        let end_addr = start_addr + (desc.page_count * 4096);
        let start_frame: PhysFrame = PhysFrame::containing_address(PhysAddr::new(start_addr));
        let end_frame = PhysFrame::containing_address(PhysAddr::new(end_addr - 1));

        let mut page_in_region = 0;
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            page_in_region += 1;
            if page_in_region % 1000 == 0 {
                println!(
                    "[DEBUG] ... processed {} pages in region {}",
                    page_in_region, region_count
                );
            }

            let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                match mem_map.map_to(page, frame, flags, &mut frame_allocator) {
                    Ok(flush) => {
                        flush.flush();
                        mapped_count += 1;
                    }
                    Err(MapToError::PageAlreadyMapped(_)) => {
                        skipped_already += 1;
                    }
                    Err(MapToError::ParentEntryHugePage) => {
                        skipped_huge += 1;
                    }
                    Err(e) => {
                        println!(
                            "[ERROR] Mapping failed at page 0x{:x}: {:?}",
                            page.start_address().as_u64(),
                            e
                        );
                        panic!("Unknown mapping error {:?}", e);
                    }
                }
            }
        }
    }

    println!(
        "[DEBUG] Mapping complete: {} mapped, {} huge pages skipped, {} already mapped",
        mapped_count, skipped_huge, skipped_already
    );
    unsafe {
        x86_64::registers::control::Cr3::write(
            pm4_frame,
            x86_64::registers::control::Cr3Flags::empty(),
        );
    }
}
