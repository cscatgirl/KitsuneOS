use lazy_static::lazy_static;
use spin::Mutex;
use x86_64::PhysAddr;
use x86_64::structures::paging::{FrameAllocator, PhysFrame, Size4KiB};
pub struct FrameAllocatorWrapper;

lazy_static! {
    static ref FRAME_ALLOCATOR: Mutex<Option<BitmapFrameAllocator>> = Mutex::new(None);
}

pub fn init_frame_allocator(allocator: BitmapFrameAllocator) {
    *FRAME_ALLOCATOR.lock() = Some(allocator);
}
pub fn with_frame_allocator<F, R>(f: F) -> Option<R>
where
    F: FnOnce(&mut BitmapFrameAllocator) -> R,
{
    let mut guard = FRAME_ALLOCATOR.lock();
    if let Some(allocator) = guard.as_mut() {
        Some(f(allocator))
    } else {
        None
    }
}
unsafe impl FrameAllocator<Size4KiB> for FrameAllocatorWrapper {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        with_frame_allocator(|alloc| alloc.allocate_frame()).flatten()
    }
}
pub struct BitmapFrameAllocator {
    pub bitmap_start: *mut u64,
    bitmap_len: usize,
    total_frames: usize,
    used_frames: usize,
    next_free: usize,
}

// SAFETY: We guarantee that the bitmap will only be accessed from one thread at a time
// via the Mutex wrapper, making it safe to send between threads and share.
unsafe impl Send for BitmapFrameAllocator {}
unsafe impl Sync for BitmapFrameAllocator {}

impl BitmapFrameAllocator {
    pub fn new(start_addr: *mut u64, size_of_mem: usize, total_frame: usize) -> Self {
        BitmapFrameAllocator {
            bitmap_start: start_addr,
            bitmap_len: size_of_mem,
            total_frames: total_frame,
            used_frames: total_frame,
            next_free: 0,
        }
    }
    fn mark_freed(&mut self, frame: usize) {
        if frame >= self.total_frames {
            return;
        }
        let index = frame / 64;
        let bit = frame % 64;
        unsafe {
            let entry = self.bitmap_start.add(index);
            let current = entry.read_volatile();
            if (current & (1 << bit)) != 0 {
                entry.write_volatile(current & !(1 << bit));
                self.used_frames -= 1;
            }
        }
    }
    fn mark_used(&mut self, frame: usize) {
        if frame >= self.total_frames {
            return;
        }
        let index = frame / 64;
        let bit = frame % 64;
        unsafe {
            let entry = self.bitmap_start.add(index);
            let current = entry.read_volatile();
            if (current & (1 << bit)) == 0 {
                entry.write_volatile(current | (1 << bit));
                self.used_frames += 1;
            }
        }
    }
    pub fn mark_range_free(&mut self, start_frame: usize, count: usize) {
        for frame in start_frame..(start_frame + count) {
            self.mark_freed(frame);
        }
    }
    pub fn mark_range_used(&mut self, start_frame: usize, count: usize) {
        for frame in start_frame..(start_frame + count) {
            self.mark_used(frame);
        }
    }
}
unsafe impl FrameAllocator<Size4KiB> for BitmapFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        unsafe {
            // Search from next_free to end
            for i in self.next_free..self.bitmap_len {
                let entry = self.bitmap_start.add(i).read_volatile();
                if entry != u64::MAX {
                    // Found a u64 with at least one free bit
                    for bit in 0..64 {
                        if (entry & (1 << bit)) == 0 {
                            let frame_no = i * 64 + bit;
                            if frame_no >= self.total_frames {
                                continue; // Skip invalid frames, keep searching
                            }
                            self.mark_used(frame_no);
                            self.next_free = i;
                            let address = frame_no * 4096;
                            let phys_addr = PhysAddr::new(address as u64);
                            let frame: PhysFrame<Size4KiB> =
                                PhysFrame::from_start_address(phys_addr).unwrap();
                            return Some(frame);
                        }
                    }
                }
            }

            // Wraparound: search from beginning to next_free
            for i in 0..self.next_free {
                let entry = self.bitmap_start.add(i).read_volatile();
                if entry != u64::MAX {
                    for bit in 0..64 {
                        if (entry & (1 << bit)) == 0 {
                            let frame_no = i * 64 + bit;
                            if frame_no >= self.total_frames {
                                continue;
                            }
                            self.mark_used(frame_no);
                            self.next_free = i;
                            let address = frame_no * 4096;
                            let phys_addr = PhysAddr::new(address as u64);
                            let frame: PhysFrame<Size4KiB> =
                                PhysFrame::from_start_address(phys_addr).unwrap();
                            return Some(frame);
                        }
                    }
                }
            }

            None
        }
    }
}
