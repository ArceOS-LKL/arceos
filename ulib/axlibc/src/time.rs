use arceos_posix_api::{sys_clock, sys_clock_gettime, sys_nanosleep};

#[cfg(all(feature = "multitask", feature = "irq"))]
use arceos_posix_api::{sys_timer_create, sys_timer_delete, sys_timer_settime};

use core::ffi::c_int;

use crate::{ctypes, utils::e};

/// Get clock time since booting
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock_gettime(clk: ctypes::clockid_t, ts: *mut ctypes::timespec) -> c_int {
    e(sys_clock_gettime(clk, ts))
}

/// Sleep some nanoseconds
///
/// TODO: should be woken by signals, and set errno
#[unsafe(no_mangle)]
pub unsafe extern "C" fn nanosleep(
    req: *const ctypes::timespec,
    rem: *mut ctypes::timespec,
) -> c_int {
    e(sys_nanosleep(req, rem))
}

/// Get the clock tick
#[unsafe(no_mangle)]
pub unsafe extern "C" fn clock() -> ctypes::clock_t {
    sys_clock()
}

/// Create a timer.
#[cfg(all(feature = "multitask", feature = "irq"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_create(
    clk: ctypes::clockid_t,
    se: *const ctypes::sigevent,
    timer: *mut ctypes::timer_t,
) -> c_int {
    e(sys_timer_create(clk, se, timer))
}

/// Delete a timer.
#[cfg(all(feature = "multitask", feature = "irq"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_delete(timer: ctypes::timer_t) -> c_int {
    e(sys_timer_delete(timer))
}

/// Set the time for a timer.
#[cfg(all(feature = "multitask", feature = "irq"))]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn timer_settime(
    timer: ctypes::timer_t,
    flags: c_int,
    value: *const ctypes::itimerspec,
    ovalue: *mut ctypes::itimerspec,
) -> c_int {
    e(sys_timer_settime(timer, flags, value, ovalue))
}
