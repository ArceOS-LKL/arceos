use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use axerrno::LinuxError;
use axhal::time::TimeValue;
use axsync::Mutex;
use axtask::{AxTaskRef, WaitQueue};
use core::{
    ffi::{c_int, c_void},
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::{
    ctypes::{self, itimerspec, sigval, timespec},
    imp::pthread::{Pthread, TID_TO_PTHREAD},
};

#[derive(Clone)]
struct TimerCallback {
    callback: *mut c_void,
    argument: sigval,
}

struct SigTimer {
    callback: TimerCallback,
    _clk: ctypes::clockid_t,
    exited: bool,
    /// The next deadline of the timer.
    deadline: u64,
    /// The interval of the timer(nanoseconds).
    ///
    /// If the interval is 0, the timer is a one-shot timer.
    interval: u64,
    wq: Arc<WaitQueue>,
    task: AxTaskRef,
}

unsafe impl Send for SigTimer {}
unsafe impl Sync for SigTimer {}

static TIMER_MAP: Mutex<BTreeMap<usize, SigTimer>> = Mutex::new(BTreeMap::new());

fn create_timer_thread(callback_fn: *mut c_void, argument: sigval) -> AxTaskRef {
    let callback_fn = unsafe { core::mem::transmute::<*mut c_void, fn(sigval)>(callback_fn) };
    let arg_addr = &argument as *const _ as usize;
    axtask::spawn(move || {
        let arg = unsafe { core::ptr::read_unaligned(arg_addr as *const sigval) };
        callback_fn(arg);
        axtask::exit(0);
    })
}

/// Create a timer.
pub unsafe fn sys_timer_create(
    clk: ctypes::clockid_t,
    se: *const ctypes::sigevent,
    timer: *mut ctypes::timer_t,
) -> c_int {
    static TIMER_ID: AtomicUsize = AtomicUsize::new(1);
    syscall_body!(sys_timer_create, {
        if clk != ctypes::CLOCK_REALTIME as c_int {
            return Err(LinuxError::ENOSYS);
        }
        const SIGEV_THREAD: c_int = 2;
        let se = unsafe { &*se };
        if se.sigev_notify != SIGEV_THREAD {
            return Err(LinuxError::ENOSYS);
        }

        let id = TIMER_ID.fetch_add(1, Ordering::Relaxed);
        let wq = Arc::new(WaitQueue::new());
        let wq_ptr = Arc::clone(&wq);
        let task = axtask::spawn(move || {
            fn deadline(timer_id: usize) -> u64 {
                TIMER_MAP.lock().get(&timer_id).unwrap().deadline
            }
            if TIMER_MAP.lock().get(&id).is_none() {
                axtask::WaitQueue::wait(&wq_ptr);
            }
            loop {
                if TIMER_MAP.lock().get(&id).unwrap().exited {
                    break;
                }
                loop {
                    let curr_ddl = deadline(id);
                    let dur = curr_ddl - axhal::time::wall_time_nanos();
                    if curr_ddl == 0 || dur as i64 <= 0 {
                        break;
                    }
                    axtask::WaitQueue::wait_timeout(&wq_ptr, TimeValue::from_nanos(dur));
                }

                let curr_ddl = deadline(id);
                if curr_ddl != 0 && axhal::time::wall_time_nanos() >= curr_ddl {
                    // The timer has been expired, so handle the callback.
                    let handler = TIMER_MAP.lock().get_mut(&id).unwrap().callback.clone();
                    let task = create_timer_thread(handler.callback, handler.argument);
                    let tid = task.id().as_u64();
                    let pthread = Pthread::from_axtask(task);

                    TID_TO_PTHREAD.write().insert(tid, Arc::new(pthread));
                    Pthread::join(tid as _).unwrap();

                    // TODO: apply the interval to the next deadline.
                }
            }

            axtask::exit(0);
        });

        TIMER_MAP.lock().insert(
            id,
            SigTimer {
                callback: TimerCallback {
                    callback: unsafe {
                        se.__sev_fields.__sev_thread.sigev_notify_function.unwrap() as *mut c_void
                    },
                    argument: se.sigev_value,
                },
                _clk: clk,
                exited: false,
                deadline: 0,
                interval: 0,
                wq: Arc::clone(&wq),
                task,
            },
        );

        unsafe {
            let timer = &mut *timer;
            *timer = id as ctypes::timer_t;
        }
        Ok(0)
    })
}

/// Set the time for a timer.
pub unsafe fn sys_timer_settime(
    timer: ctypes::timer_t,
    flags: c_int,
    new_value: *const itimerspec,
    old_value: *mut itimerspec,
) -> c_int {
    fn to_nanos(ts: timespec) -> u64 {
        ts.tv_nsec as u64 + ts.tv_sec as u64 * axhal::time::NANOS_PER_SEC
    }

    syscall_body!(sys_timer_settime, {
        if flags != 0 || !old_value.is_null() {
            warn!("Not implemented: timer_settime with flags != 0 or old_value != NULL");
            return Err(LinuxError::EINVAL);
        }
        if new_value.is_null() {
            return Err(LinuxError::EINVAL);
        }
        let mut timer_map = TIMER_MAP.lock();
        let timer = timer_map
            .get_mut(&(timer as usize))
            .ok_or(LinuxError::EINVAL)?;
        let new_value = unsafe { &*new_value };
        if new_value.it_interval.tv_sec != 0 || new_value.it_interval.tv_nsec != 0 {
            warn!("Not implemented: timer_settime with it_interval != 0");
            return Err(LinuxError::EINVAL);
        }
        // The value is relative to the current time.
        timer.deadline = axhal::time::wall_time_nanos() + to_nanos(new_value.it_value);
        timer.interval = to_nanos(new_value.it_interval);
        timer.wq.notify_one(false);
        Ok(0)
    })
}

/// Delete a timer.
pub unsafe fn sys_timer_delete(timer: ctypes::timer_t) -> c_int {
    syscall_body!(sys_timer_delete, {
        let task_to_join = {
            let mut timer_map = TIMER_MAP.lock();
            timer_map.get_mut(&(timer as usize)).map(|timer_data| {
                timer_data.deadline = 0;
                timer_data.exited = true;
                timer_data.wq.notify_one(false);
                timer_data.task.clone()
            })
        };

        if let Some(task) = task_to_join {
            task.join().unwrap();
        }

        TIMER_MAP.lock().remove(&(timer as usize));

        Ok(0)
    })
}
