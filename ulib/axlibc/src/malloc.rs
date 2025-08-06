//! Provides the corresponding malloc(size_t) and free(size_t) when using the C user program.
//!
//! The normal malloc(size_t) and free(size_t) are provided by the library malloc.h, and
//! sys_brk is used internally to apply for memory from the kernel. But in a unikernel like
//! `ArceOS`, we noticed that the heap of the Rust user program is shared with the kernel. In
//! order to maintain consistency, C user programs also choose to share the kernel heap,
//! skipping the sys_brk step.

use crate::ctypes;
use arceos_posix_api as api;

use core::ffi::c_void;
use core::ptr::NonNull;

/// Allocate memory and return the memory address.
///
/// Returns 0 on failure (the current implementation does not trigger an exception)
#[unsafe(no_mangle)]
pub unsafe extern "C" fn malloc(size: ctypes::size_t) -> *mut c_void {
    api::sys_malloc(size).as_ptr() as *mut c_void
}

/// Deallocate memory.
///
/// (WARNING) If the address to be released does not match the allocated address, an error should
/// occur, but it will NOT be checked out. This is due to the global allocator `Buddy_system`
/// (currently used) does not check the validity of address to be released.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn free(ptr: *mut c_void) {
    if ptr.is_null() {
        return;
    }
    api::sys_free(NonNull::new(ptr as *mut u8).unwrap());
}

/// Allocate pages
#[unsafe(no_mangle)]
pub unsafe extern "C" fn page_alloc(size: ctypes::size_t) -> *mut c_void {
    api::sys_page_alloc(size).as_ptr() as *mut c_void
}

/// Free pages
#[unsafe(no_mangle)]
pub unsafe extern "C" fn page_free(ptr: *mut c_void, size: ctypes::size_t) {
    api::sys_page_free(NonNull::new(ptr as *mut u8).unwrap(), size);
}
