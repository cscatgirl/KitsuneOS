use core::ptr::null_mut;

use crate::allocator_types::linked_list;
use alloc::alloc::GlobalAlloc;
const HEAP_START: usize = 0x_4444_4444_0000;
const HEAP_SIZE: usize = 4 * 1024 * 1024;
pub struct allocator;
#[global_allocator]
static ALLOCATOR: allocator = allocator;
unsafe impl GlobalAlloc for allocator {
    unsafe fn alloc(&self, layout: core::alloc::Layout) -> *mut u8 {
        null_mut()
    }
    unsafe fn dealloc(&self, _ptr: *mut u8, layout: core::alloc::Layout) {
        panic!("I should not be called");
    }
}
