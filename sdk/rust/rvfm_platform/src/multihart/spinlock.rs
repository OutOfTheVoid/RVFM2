use core::cell::UnsafeCell;
use core::sync::atomic;
use core::convert::{AsRef, AsMut};

use lock_api::{GuardSend, Mutex, MutexGuard};

pub use lock_api::RawMutex;

pub struct RawSpinLock {
    pub(crate) lock: atomic::AtomicUsize,
}

impl RawSpinLock {
    pub const fn new() -> Self {
        Self {
            lock: atomic::AtomicUsize::new(0)
        }
    }
}

unsafe impl RawMutex for RawSpinLock {
    const INIT: RawSpinLock = RawSpinLock::new();

    type GuardMarker = GuardSend;

    fn lock(&self) {
        // Note: This isn't the best way of implementing a spinlock, but it
        // suffices for the sake of this example.
        while !self.try_lock() {}
    }

    fn try_lock(&self) -> bool {
        self.lock.swap(1, atomic::Ordering::AcqRel) == 0
    }

    unsafe fn unlock(&self) {
        self.lock.store(0, atomic::Ordering::Release);
    }
}

pub type SpinLock<T> = Mutex<RawSpinLock, T>;
pub type SpinLockGuard<'a, T> = MutexGuard<'a, RawSpinLock, T>;
