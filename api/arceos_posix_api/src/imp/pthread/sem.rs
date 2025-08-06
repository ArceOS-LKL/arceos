use core::ffi::c_int;

use alloc::boxed::Box;
use axsync::Mutex;
use axtask::WaitQueue;

use crate::ctypes;

pub struct Semaphore {
    count: Mutex<isize>,
    wq: WaitQueue,
}

impl Semaphore {
    fn new(count: isize) -> Self {
        Self {
            count: Mutex::new(count),
            wq: WaitQueue::new(),
        }
    }

    fn destroy(&self) {
        self.wq.notify_all(false);
    }

    fn up(&self) {
        *self.count.lock() += 1;
        self.wq.notify_one(false);
    }

    fn down(&self) {
        loop {
            self.wq.wait_until(|| *(self.count.lock()) > 0);
            let mut count = self.count.lock();
            if *count > 0 {
                *count -= 1;
                break;
            }
            drop(count);
        }
    }
}

/// Allocate a semaphore.
pub fn sys_sem_alloc(count: c_int) -> *mut Semaphore {
    let sem = Box::new(Semaphore::new(count as isize));
    Box::into_raw(sem)
}

/// Destroy a semaphore.
pub fn sys_sem_destroy(sem: *mut ctypes::sem_t) -> c_int {
    unsafe {
        (*sem.cast::<Semaphore>()).destroy();
    }
    0
}

/// Post a semaphore.
pub fn sys_sem_post(sem: *mut ctypes::sem_t) -> c_int {
    unsafe {
        (*sem.cast::<Semaphore>()).up();
    }
    0
}

/// Wait a semaphore.
pub fn sys_sem_wait(sem: *mut ctypes::sem_t) -> c_int {
    unsafe {
        (*sem.cast::<Semaphore>()).down();
    }
    0
}
