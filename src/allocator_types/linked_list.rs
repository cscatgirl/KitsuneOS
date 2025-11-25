use core::alloc::{GlobalAlloc, Layout};
use core::{mem, ptr};
use spin::Mutex;

pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 4 * 1024 * 1024;
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
    fn find_region(&self, size: usize, align: usize) -> Option<(usize, usize)> {
        let alloc_start = align_up(self.start_addr(), align);
        let alloc_end = alloc_start.checked_add(size)?;
        if alloc_end > self.end_addr() {
            None
        } else {
            Some((alloc_start, alloc_end))
        }
    }
    fn can_split(&self, alloc_end: usize) -> bool {
        let size = self.end_addr() - alloc_end;
        size >= mem::size_of::<ListNode>()
    }
}
pub struct LinkedListAllocator {
    head: Mutex<Option<&'static mut ListNode>>,
}
impl LinkedListAllocator {
    pub const fn new() -> Self {
        LinkedListAllocator {
            head: Mutex::new(None),
        }
    }

    pub unsafe fn init(&self, heap_start: usize, heap_size: usize) {
        assert_eq!(
            align_up(heap_start, mem::align_of::<ListNode>()),
            heap_start
        );
        assert!(heap_size >= mem::size_of::<ListNode>());

        let mut initial_node = ListNode::new(heap_size);
        initial_node.next = None;

        let node_ptr = heap_start as *mut ListNode;
        unsafe {
            node_ptr.write(initial_node);

            *self.head.lock() = Some(&mut *node_ptr);
        }
    }

    fn alloc_from_list(&mut self, size: usize, align: usize) -> Option<*mut u8> {
        let mut list = self.head.lock();
        let mut current = &mut *list;

        loop {
            let node = current.take()?;

            if let Some((alloc_start, alloc_end)) = node.find_region(size, align) {
                let next = node.next.take();

                if node.can_split(alloc_end) {
                    let remainder_start = alloc_end;
                    let remainder_size = node.end_addr() - alloc_end;
                    let mut remainder_node = ListNode::new(remainder_size);
                    remainder_node.next = next;

                    let remainder_ptr = remainder_start as *mut ListNode;
                    unsafe {
                        remainder_ptr.write(remainder_node);
                        *current = Some(&mut *remainder_ptr);
                    }
                } else {
                    *current = next;
                }

                return Some(alloc_start as *mut u8);
            }

            *current = Some(node);
            current = &mut current.as_mut().unwrap().next;
        }
    }
    fn dealloc_to_list(&mut self, addr: usize, size: usize) {
        assert_eq!(align_up(addr, mem::align_of::<ListNode>()), addr);
        assert!(size >= mem::size_of::<ListNode>());

        let mut list = self.head.lock();
        let mut current = &mut *list;

        loop {
            match current.take() {
                None => {
                    let mut new_node = ListNode::new(size);
                    new_node.next = None;

                    let node_ptr = addr as *mut ListNode;
                    unsafe {
                        node_ptr.write(new_node);
                        *current = Some(&mut *node_ptr);
                    }
                    return;
                }
                Some(node) => {
                    let node_addr = node.start_addr();
                    let node_end = node.end_addr();

                    if addr < node_addr {
                        let mut new_node = ListNode::new(size);

                        if addr + size == node_addr {
                            new_node.size += node.size;
                            new_node.next = node.next.take();
                        } else {
                            new_node.next = Some(node);
                        }

                        let node_ptr = addr as *mut ListNode;
                        unsafe {
                            node_ptr.write(new_node);
                            *current = Some(&mut *node_ptr);
                        }
                        return;
                    } else if node_end == addr {
                        node.size += size;

                        if let Some(next) = node.next.as_ref() {
                            if node.end_addr() == next.start_addr() {
                                let next_node = node.next.take().unwrap();
                                node.size += next_node.size;
                                node.next = next_node.next.take();
                            }
                        }

                        *current = Some(node);
                        return;
                    } else {
                        *current = Some(node);
                        current = &mut current.as_mut().unwrap().next;
                    }
                }
            }
        }
    }
}
unsafe impl GlobalAlloc for LinkedListAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let size = layout.size().max(mem::size_of::<ListNode>());
        let align = layout.align();
        let result = unsafe {
            let allocator = self as *const Self as *mut LinkedListAllocator;
            (*allocator).alloc_from_list(size, align)
        };
        result.unwrap_or(ptr::null_mut())
    }
    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let size = layout.size().max(mem::size_of::<ListNode>());
        unsafe {
            let allocator = self as *const Self as *mut LinkedListAllocator;
            (*allocator).dealloc_to_list(ptr as usize, size);
        }
    }
}
