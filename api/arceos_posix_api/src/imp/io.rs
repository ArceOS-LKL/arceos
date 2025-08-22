use crate::ctypes;
#[cfg(feature = "fd")]
use crate::imp::fd_ops::get_file_like;
use axerrno::{LinuxError, LinuxResult};
#[cfg(not(feature = "fd"))]
use axio::prelude::*;

use core::ffi::{c_int, c_void};

fn read_impl(fd: c_int, buf: *const c_void, count: usize) -> LinuxResult<ctypes::ssize_t> {
    if buf.is_null() {
        return Err(LinuxError::EFAULT);
    }
    let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
    #[cfg(feature = "fd")]
    {
        Ok(get_file_like(fd)?.read(dst)? as ctypes::ssize_t)
    }
    #[cfg(not(feature = "fd"))]
    match fd {
        0 => Ok(super::stdio::stdin().read(dst)? as ctypes::ssize_t),
        1 | 2 => Err(LinuxError::EPERM),
        _ => Err(LinuxError::EBADF),
    }
}

fn write_impl(fd: c_int, buf: *const c_void, count: usize) -> LinuxResult<ctypes::ssize_t> {
    if buf.is_null() {
        return Err(LinuxError::EFAULT);
    }
    let src = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };
    #[cfg(feature = "fd")]
    {
        Ok(get_file_like(fd)?.write(src)? as ctypes::ssize_t)
    }
    #[cfg(not(feature = "fd"))]
    match fd {
        0 => Err(LinuxError::EPERM),
        1 | 2 => Ok(super::stdio::stdout().write(src)? as ctypes::ssize_t),
        _ => Err(LinuxError::EBADF),
    }
}

/// Read data from the file indicated by `fd`.
///
/// Return the read size if success.
pub fn sys_read(fd: c_int, buf: *mut c_void, count: usize) -> ctypes::ssize_t {
    debug!("sys_read <= {} {:#x} {}", fd, buf as usize, count);
    syscall_body!(sys_read, read_impl(fd, buf, count))
}

/// Write data to the file indicated by `fd`.
///
/// Return the written size if success.
pub fn sys_write(fd: c_int, buf: *const c_void, count: usize) -> ctypes::ssize_t {
    debug!("sys_write <= {} {:#x} {}", fd, buf as usize, count);
    syscall_body!(sys_write, write_impl(fd, buf, count))
}

pub unsafe fn rw_vector<F>(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    op: F,
) -> LinuxResult<ctypes::ssize_t>
where
    F: Fn(c_int, *const c_void, usize) -> LinuxResult<ctypes::ssize_t>,
{
    if !(0..=1024).contains(&iocnt) {
        return Err(LinuxError::EINVAL);
    }
    let iovs = unsafe { core::slice::from_raw_parts(iov, iocnt as usize) };
    let mut ret = 0;
    for iov in iovs.iter() {
        if iov.iov_len == 0 {
            continue;
        }
        let result = op(fd, iov.iov_base, iov.iov_len)?;
        if result < 0 {
            return Ok(result);
        }
        ret += result;
        if result < iov.iov_len as isize {
            break;
        }
    }
    Ok(ret)
}

/// Read from a file descriptor into multiple buffers
pub unsafe fn sys_readv(fd: c_int, iov: *const ctypes::iovec, iocnt: c_int) -> ctypes::ssize_t {
    debug!("sys_readv <= fd: {}", fd);
    syscall_body!(sys_readv, {
        unsafe { rw_vector(fd, iov, iocnt, read_impl) }
    })
}

/// Write to a file descriptor from multiple buffers
pub unsafe fn sys_writev(fd: c_int, iov: *const ctypes::iovec, iocnt: c_int) -> ctypes::ssize_t {
    debug!("sys_writev <= fd: {}", fd);
    syscall_body!(sys_writev, {
        unsafe { rw_vector(fd, iov, iocnt, write_impl) }
    })
}

#[cfg(feature = "fs")]
fn prw_impl<F>(fd: c_int, offset: ctypes::off_t, mut op: F) -> LinuxResult<ctypes::ssize_t>
where
    F: FnMut(&mut axsync::MutexGuard<axfs::fops::File>) -> axerrno::AxResult<usize>,
{
    use axio::SeekFrom;

    use crate::imp::fs::File;
    if offset < 0 {
        return Err(LinuxError::EINVAL);
    }

    let file = File::from_fd(fd)?;
    let mut file = file.inner().lock();
    let old_offset = file.seek(SeekFrom::Current(0))?;
    let result = file
        .seek(SeekFrom::Start(offset as u64))
        .and_then(|_| op(&mut file))?;
    file.seek(SeekFrom::Start(old_offset))?;
    Ok(result as ctypes::ssize_t)
}

/// Read from or write to a file descriptor at a given offset.
///
/// After the read or write operation, the file offset is not updated.
#[cfg(feature = "fs")]
pub fn sys_pread(
    fd: c_int,
    buf: *mut c_void,
    count: usize,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_pread <= {} {:#x} {} {}",
        fd, buf as usize, count, offset
    );
    syscall_body!(sys_pread, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let dst = unsafe { core::slice::from_raw_parts_mut(buf as *mut u8, count) };
        prw_impl(fd, offset, |file| file.read(dst))
    })
}

/// Write to a file descriptor at a given offset.
///
/// After the write operation, the file offset is not updated.
#[cfg(feature = "fs")]
pub fn sys_pwrite(
    fd: c_int,
    buf: *const c_void,
    count: usize,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_pwrite <= {} {:#x} {} {}",
        fd, buf as usize, count, offset
    );
    syscall_body!(sys_pwrite, {
        if buf.is_null() {
            return Err(LinuxError::EFAULT);
        }
        let src = unsafe { core::slice::from_raw_parts(buf as *const u8, count) };
        prw_impl(fd, offset, |file| file.write(src))
    })
}

/// Read data at a given offset into multiple buffers
#[cfg(feature = "fs")]
pub unsafe fn sys_preadv(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_preadv <= {} {:#x} {} {}",
        fd, iov as usize, iocnt, offset
    );
    syscall_body!(sys_preadv, {
        unsafe {
            rw_vector(fd, iov, iocnt, |fd, buf, len| {
                if buf.is_null() {
                    return Err(LinuxError::EFAULT);
                }
                let dst = core::slice::from_raw_parts_mut(buf as *mut u8, len);
                prw_impl(fd, offset, |file| file.read(dst))
            })
        }
    })
}

/// Write to a file descriptor at a given offset
#[cfg(feature = "fs")]
pub unsafe fn sys_pwritev(
    fd: c_int,
    iov: *const ctypes::iovec,
    iocnt: c_int,
    offset: ctypes::off_t,
) -> ctypes::ssize_t {
    debug!(
        "sys_pwritev <= {} {:#x} {} {}",
        fd, iov as usize, iocnt, offset
    );
    syscall_body!(sys_pwritev, {
        unsafe {
            rw_vector(fd, iov, iocnt, |fd, buf, len| {
                if buf.is_null() {
                    return Err(LinuxError::EFAULT);
                }
                let src = core::slice::from_raw_parts(buf as *const u8, len);
                prw_impl(fd, offset, |file| file.write(src))
            })
        }
    })
}
