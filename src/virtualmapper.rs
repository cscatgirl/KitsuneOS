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

pub fn map_physical_to_virtual(
    mmap: &uefi::mem::memory_map::MemoryMapOwned,
    framebuffer_addr: u64,
    framebuffer_size: usize,
) {
    let mut frame_allocator = FrameAllocatorWrapper;
    let pm4_frame = frame_allocator.allocate_frame().expect("failed to alloc");
    let phys_addr = pm4_frame.start_address().as_u64();
    let pm4_frame_ptr = pm4_frame.start_address().as_u64() as *mut PageTable;
    unsafe {
        core::ptr::write_bytes(pm4_frame_ptr as *mut u8, 0, 4096);
    }
    let pm4 = unsafe { &mut *pm4_frame_ptr };
    let mut mem_map = unsafe { OffsetPageTable::new(pm4, VirtAddr::new(0)) };

    for desc in mmap.entries() {
        let start_addr = desc.phys_start;
        let end_addr = start_addr + (desc.page_count * 4096);
        let start_frame: PhysFrame = PhysFrame::containing_address(PhysAddr::new(start_addr));
        let end_frame = PhysFrame::containing_address(PhysAddr::new(end_addr - 1));

        let mut page_in_region = 0;
        for frame in PhysFrame::range_inclusive(start_frame, end_frame) {
            page_in_region += 1;

            let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
            let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
            unsafe {
                match mem_map.map_to(page, frame, flags, &mut frame_allocator) {
                    Ok(flush) => {
                        flush.flush();
                    }
                    Err(MapToError::PageAlreadyMapped(_)) => {}
                    Err(MapToError::ParentEntryHugePage) => {}
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

    let fb_start = framebuffer_addr;
    let fb_end = framebuffer_addr + framebuffer_size as u64;
    let fb_start_frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(fb_start));
    let fb_end_frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(fb_end - 1));

    for frame in PhysFrame::range_inclusive(fb_start_frame, fb_end_frame) {
        let page = Page::containing_address(VirtAddr::new(frame.start_address().as_u64()));
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

        unsafe {
            match mem_map.map_to(page, frame, flags, &mut frame_allocator) {
                Ok(flush) => {
                    flush.flush();
                }
                Err(MapToError::PageAlreadyMapped(_)) => {}
                Err(e) => {
                    println!(
                        "[ERROR] Failed to map framebuffer page 0x{:x}: {:?}",
                        page.start_address().as_u64(),
                        e
                    );
                    panic!("Framebuffer mapping failed");
                }
            }
        }
    }

    let lapic_base = 0xFEE00000u64;
    let lapic_frame: PhysFrame<Size4KiB> = PhysFrame::containing_address(PhysAddr::new(lapic_base));
    let lapic_page = Page::containing_address(VirtAddr::new(lapic_base));
    let lapic_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

    unsafe {
        match mem_map.map_to(lapic_page, lapic_frame, lapic_flags, &mut frame_allocator) {
            Ok(flush) => {
                flush.flush();
            }
            Err(MapToError::PageAlreadyMapped(_)) => {
                println!("[DEBUG] Local APIC already mapped");
            }
            Err(e) => {
                println!("[ERROR] Failed to map Local APIC: {:?}", e);
                panic!("Local APIC mapping failed");
            }
        }
    }

    let ioapic_base = 0xFEC00000u64;
    let ioapic_frame: PhysFrame<Size4KiB> =
        PhysFrame::containing_address(PhysAddr::new(ioapic_base));
    let ioapic_page = Page::containing_address(VirtAddr::new(ioapic_base));
    let ioapic_flags =
        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::NO_CACHE;

    unsafe {
        match mem_map.map_to(
            ioapic_page,
            ioapic_frame,
            ioapic_flags,
            &mut frame_allocator,
        ) {
            Ok(flush) => {
                flush.flush();
            }
            Err(MapToError::PageAlreadyMapped(_)) => {
                println!("[DEBUG] I/O APIC already mapped");
            }
            Err(e) => {
                println!("[ERROR] Failed to map I/O APIC: {:?}", e);
                panic!("I/O APIC mapping failed");
            }
        }
    }

    unsafe {
        x86_64::registers::control::Cr3::write(
            pm4_frame,
            x86_64::registers::control::Cr3Flags::empty(),
        );
    }
}

pub fn map_heap(heap_start: u64, heap_size: usize) {
    use crate::memory::FrameAllocatorWrapper;

    let (pm4_frame, _) = Cr3::read();
    let pm4_ptr = pm4_frame.start_address().as_u64() as *mut PageTable;
    let pm4 = unsafe { &mut *pm4_ptr };
    let mut mem_map = unsafe { OffsetPageTable::new(pm4, VirtAddr::new(0)) };

    let mut frame_allocator = FrameAllocatorWrapper;

    let page_count = (heap_size + 4095) / 4096;
    let heap_start_page = Page::<Size4KiB>::containing_address(VirtAddr::new(heap_start));

    for i in 0..page_count {
        let page = heap_start_page + i as u64;

        let frame = match frame_allocator.allocate_frame() {
            Some(f) => f,
            None => panic!(
                "[ERROR] Failed to allocate physical frame for heap page {}",
                i
            ),
        };

        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;

        unsafe {
            match mem_map.map_to(page, frame, flags, &mut frame_allocator) {
                Ok(flush) => {
                    flush.flush();
                }
                Err(e) => {
                    println!(
                        "[ERROR] Failed to map heap page 0x{:x}: {:?}",
                        page.start_address().as_u64(),
                        e
                    );
                    panic!("Heap mapping failed");
                }
            }
        }
    }
}
