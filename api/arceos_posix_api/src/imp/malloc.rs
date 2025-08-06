use axhal::mem::PAGE_SIZE_4K;

use crate::ctypes;
use core::{alloc::Layout, ptr::NonNull};

struct MemoryControlBlock {
    size: usize,
}

const CTRL_BLK_SIZE: usize = core::mem::size_of::<MemoryControlBlock>();

/// Allocate memory from the heap according to the size.
///
/// # Arguments
///
/// * `size` - The size of the memory to allocate.
///
/// Return the pointer to the allocated memory.
pub fn sys_malloc(size: ctypes::size_t) -> NonNull<u8> {
    // Allocate `(actual length) + 8`. The lowest 8 Bytes are stored in the actual allocated space size.
    // This is because free(uintptr_t) has only one parameter representing the address,
    // So we need to save in advance to know the size of the memory space that needs to be released
    let layout = Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
    unsafe {
        let ptr = axalloc::global_allocator().alloc(layout).unwrap();
        ptr.as_ptr()
            .cast::<MemoryControlBlock>()
            .write(MemoryControlBlock { size });
        axlog::debug!("alloc ptr: {:x}", ptr.add(CTRL_BLK_SIZE).as_ptr() as usize);
        ptr.add(CTRL_BLK_SIZE)
    }
}

/// Free memory from the heap according to the pointer.
///
/// # Arguments
///
/// * `ptr` - The pointer to the memory to free.
pub fn sys_free(ptr: NonNull<u8>) {
    unsafe {
        axlog::debug!("free ptr: {:x}", ptr.as_ptr() as usize);
        let ptr = ptr.sub(CTRL_BLK_SIZE).cast::<MemoryControlBlock>();
        let size = ptr.read().size;
        let layout: Layout = Layout::from_size_align(size + CTRL_BLK_SIZE, 8).unwrap();
        axalloc::global_allocator().dealloc(ptr.cast(), layout);
    }
}

/// Allocate pages from the heap.
///
/// # Arguments
///
/// * `size` - The size of the memory to allocate, which will be aligned to the page size.
pub fn sys_page_alloc(size: ctypes::size_t) -> NonNull<u8> {
    let layout = Layout::from_size_align(size, PAGE_SIZE_4K).unwrap();
    axalloc::global_allocator().alloc(layout).unwrap()
}

/// Free pages from the heap.
///
/// # Arguments
///
/// * `ptr` - The pointer to the memory to free.
/// * `size` - The size of the memory to free, which will be aligned to the page size.
pub fn sys_page_free(ptr: NonNull<u8>, size: ctypes::size_t) {
    let layout = Layout::from_size_align(size, PAGE_SIZE_4K).unwrap();
    axalloc::global_allocator().dealloc(ptr, layout);
}
