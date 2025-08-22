use core::ffi::{c_int, c_void};

use arceos_posix_api::{sys_read, sys_readv, sys_write, sys_writev};

#[cfg(feature = "fs")]
use arceos_posix_api::{sys_pread, sys_preadv, sys_pwrite, sys_pwritev};

use crate::{ctypes, utils::e};

/// Read data from the file indicated by `fd`.
///
/// Return the read size if success.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn read(fd: c_int, buf: *mut c_void, count: usize) -> ctypes::ssize_t {
    e(sys_read(fd, buf, count) as _) as _
}

/// Write data to the file indicated by `fd`.
///
/// Return the written size if success.
#[unsafe(no_mangle)]
#[cfg(not(test))]
pub unsafe extern "C" fn write(fd: c_int, buf: *const c_void, count: usize) -> ctypes::ssize_t {
    e(sys_write(fd, buf, count) as _) as _
}

/// Read a vector.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn readv(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
) -> ctypes::ssize_t {
    e(sys_readv(fd, iov, iocnt) as _) as _
}

/// Write a vector.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn writev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
) -> ctypes::ssize_t {
    e(sys_writev(fd, iov, iocnt) as _) as _
}

/// Read from or write to a file descriptor at a given offset
#[unsafe(no_mangle)]
#[cfg(feature = "fs")]
pub unsafe extern "C" fn pread(
    fd: c_int,
    buf: *mut c_void,
    count: usize,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    e(sys_pread(fd, buf, count, offset) as _) as _
}

/// Write to a file descriptor at a given offset
#[unsafe(no_mangle)]
#[cfg(feature = "fs")]
pub unsafe extern "C" fn pwrite(
    fd: c_int,
    buf: *const c_void,
    count: usize,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    e(sys_pwrite(fd, buf, count, offset) as _) as _
}

/// Read data at a given offset into multiple buffers
#[unsafe(no_mangle)]
#[cfg(feature = "fs")]
pub unsafe extern "C" fn preadv(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    e(sys_preadv(fd, iov, iocnt, offset) as _) as _
}

/// Write data at a given offset from multiple buffers
#[unsafe(no_mangle)]
#[cfg(feature = "fs")]
pub unsafe extern "C" fn pwritev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    e(sys_pwritev(fd, iov, iocnt, offset) as _) as _
}
