use core::mem;

use spin::lock_api::Mutex;
const HEAP_START: usize = 0x_4444_4444_0000;
const HEAP_SIZE: usize = 4 * 1024 * 1024;
static ALLOCATOR: LinkedListAllocator = LinkedListAllocator;
fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
struct ListNode {
    size: usize,
    next: Option<&'static mut ListNode>,
}
impl ListNode {
    const fn new(size: usize) -> Self {
        ListNode { size, next: None }
    }

    fn start_addr(&self) -> usize {
        self as *const Self as usize
    }

    fn end_addr(&self) -> usize {
        self.start_addr() + self.size
    }
}
pub struct LinkedListAllocator {
    head: Mutex<Option<&'static mut ListNode>>,
}
impl LinkedListAllocator {
    pub fn new() -> Self {
        LinkedListAllocator {
            head: Mutex::new(None),
        }
    }
    pub fn init_heap(&mut self) {
        self.create_free_node(HEAP_START, HEAP_SIZE);
    }
    pub fn create_free_node(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());
        let mut node = ListNode::new(size);
        node.next = self.head.lock().take();
        let node_ptr = addr as *mut ListNode;
        unsafe {
            node_ptr.write(node);
            self.head = Mutex::new(Some(&mut *node_ptr));
        }
    }
}
