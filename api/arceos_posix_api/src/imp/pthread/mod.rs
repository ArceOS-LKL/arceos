use alloc::{collections::BTreeMap, sync::Arc};
use core::cell::UnsafeCell;
use core::ffi::{c_int, c_void};

use axerrno::{LinuxError, LinuxResult};
use axtask::AxTaskRef;
use spin::RwLock;

use crate::ctypes;

pub mod mutex;
pub mod sem;
lazy_static::lazy_static! {
    static ref TID_TO_PTHREAD: RwLock<BTreeMap<u64, Arc<Pthread>>> = {
        let mut map = BTreeMap::new();
        let main_task = axtask::current();
        let main_tid = main_task.id().as_u64();
        let main_thread = Pthread {
            inner: main_task.as_task_ref().clone(),
            retval: Arc::new(Packet {
                result: UnsafeCell::new(core::ptr::null_mut()),
            }),
        };
        map.insert(main_tid, Arc::new(main_thread));
        RwLock::new(map)
    };
}

struct Packet<T> {
    result: UnsafeCell<T>,
}

unsafe impl<T> Send for Packet<T> {}
unsafe impl<T> Sync for Packet<T> {}

pub struct Pthread {
    inner: AxTaskRef,
    retval: Arc<Packet<*mut c_void>>,
}

impl Pthread {
    fn create(
        _attr: *const ctypes::pthread_attr_t,
        start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
        arg: *mut c_void,
    ) -> LinuxResult<ctypes::pthread_t> {
        let arg_wrapper = ForceSendSync(arg);

        let my_packet: Arc<Packet<*mut c_void>> = Arc::new(Packet {
            result: UnsafeCell::new(core::ptr::null_mut()),
        });
        let their_packet = my_packet.clone();

        let main = move || {
            let arg = arg_wrapper;
            let ret = start_routine(arg.0);
            unsafe { *their_packet.result.get() = ret };
            drop(their_packet);
        };

        let task_inner = axtask::spawn(main);
        let tid = task_inner.id().as_u64();
        let thread = Pthread {
            inner: task_inner,
            retval: my_packet,
        };
        TID_TO_PTHREAD.write().insert(tid, Arc::new(thread));
        Ok(tid as _)
    }

    fn current() -> Option<Arc<Pthread>> {
        let tid = axtask::current().id().as_u64();
        match TID_TO_PTHREAD.read().get(&tid) {
            None => None,
            Some(ptr) => Some(Arc::clone(ptr)),
        }
    }

    fn exit_current(retval: *mut c_void) -> ! {
        let thread = Self::current().expect("fail to get current thread");
        unsafe { *thread.retval.result.get() = retval };
        axtask::exit(0);
    }

    fn join(tid: ctypes::pthread_t) -> LinuxResult<*mut c_void> {
        if core::ptr::eq(tid, axtask::current().id().as_u64() as _) {
            return Err(LinuxError::EDEADLK);
        }

        let thread = match Self::pthread_ref(tid as u64) {
            None => return Err(LinuxError::ESRCH),
            Some(ptr) => ptr,
        };
        thread.inner.join();
        let tid = thread.inner.id().as_u64();
        let retval = unsafe { *thread.retval.result.get() };
        TID_TO_PTHREAD.write().remove(&tid);
        drop(thread);
        Ok(retval)
    }

    fn pthread_ref(tid: u64) -> Option<Arc<Pthread>> {
        match TID_TO_PTHREAD.read().get(&tid) {
            None => None,
            Some(ptr) => Some(Arc::clone(ptr)),
        }
    }
}

/// Returns the `pthread` struct of current thread.
pub fn sys_pthread_self() -> ctypes::pthread_t {
    Pthread::current()
        .expect("fail to get current thread")
        .inner
        .id()
        .as_u64() as _
}

/// Check if two threads are equal.
pub unsafe fn sys_pthread_equal(a: ctypes::pthread_t, b: ctypes::pthread_t) -> c_int {
    if a == b { 1 } else { 0 }
}

/// Create a new thread with the given entry point and argument.
///
/// If successful, it stores the tid of the new thread in `res` and returns 0.
pub unsafe fn sys_pthread_create(
    res: *mut ctypes::pthread_t,
    attr: *const ctypes::pthread_attr_t,
    start_routine: extern "C" fn(arg: *mut c_void) -> *mut c_void,
    arg: *mut c_void,
) -> c_int {
    debug!(
        "sys_pthread_create <= {:#x}, {:#x}",
        start_routine as usize, arg as usize
    );
    syscall_body!(sys_pthread_create, {
        let tid = Pthread::create(attr, start_routine, arg)?;
        unsafe { core::ptr::write(res, tid) };
        crate::sys_sched_yield();
        Ok(0)
    })
}

/// Exits the current thread. The value `retval` will be returned to the joiner.
pub fn sys_pthread_exit(retval: *mut c_void) -> ! {
    debug!("sys_pthread_exit <= {:#x}", retval as usize);
    Pthread::exit_current(retval);
}

/// Waits for the given thread to exit, and stores the return value in `retval`.
pub unsafe fn sys_pthread_join(thread: ctypes::pthread_t, retval: *mut *mut c_void) -> c_int {
    debug!("sys_pthread_join <= {:#x}", retval as usize);
    syscall_body!(sys_pthread_join, {
        let ret = Pthread::join(thread)?;
        if !retval.is_null() {
            unsafe { core::ptr::write(retval, ret) };
        }
        Ok(0)
    })
}

/// Detach the current thread.
pub unsafe fn sys_pthread_detach(thread: ctypes::pthread_t) -> c_int {
    debug!("sys_pthread_detach <= {:#x}", thread as usize);
    warn!("sys_pthread_detach is not implemented");
    0
}

#[derive(Clone, Copy)]
struct ForceSendSync<T>(T);

unsafe impl<T> Send for ForceSendSync<T> {}
unsafe impl<T> Sync for ForceSendSync<T> {}
