pub mod linked_list;

use linked_list::{HEAP_SIZE, HEAP_START, LinkedListAllocator};

#[global_allocator]
static ALLOCATOR: LinkedListAllocator = LinkedListAllocator::new();

pub unsafe fn init() {
    unsafe {
        ALLOCATOR.init(HEAP_START, HEAP_SIZE);
    }
}
