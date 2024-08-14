use super::super::intrin::*;
use super::spinlock::*;

use critical_section::RawRestoreState;

static CRITICAL_SECTION_LOCK: RawSpinLock = RawSpinLock::new();

struct RvfmCriticalSection;
critical_section::set_impl!(RvfmCriticalSection);

unsafe impl critical_section::Impl for RvfmCriticalSection {
    unsafe fn acquire() -> RawRestoreState {
        let rs = hart_disable_interrupts();
        CRITICAL_SECTION_LOCK.lock();
        rs
    }

    unsafe fn release(token: RawRestoreState) {
        unsafe { CRITICAL_SECTION_LOCK.unlock(); }
        if token {
            hart_enable_interrupts();
        }
    }
}