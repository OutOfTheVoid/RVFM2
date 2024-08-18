use core::{cell::UnsafeCell, mem::MaybeUninit};

use crate::hart::Hart;

pub struct UnsafeHartLocal<T> {
    per_core: UnsafeCell<[MaybeUninit<T>; 4]>,
}

impl<T> Default for UnsafeHartLocal<T> {
    fn default() -> Self {
        Self {
            per_core: UnsafeCell::new (
                [
                    MaybeUninit::uninit(),
                    MaybeUninit::uninit(),
                    MaybeUninit::uninit(),
                    MaybeUninit::uninit(),
                ]
            ),
        }
    }
}

impl<T> UnsafeHartLocal<T> {
    pub unsafe fn get(&self) -> &MaybeUninit<T> {
        let index = Hart::current().to_u32() as usize;
        &(&*self.per_core.get())[index]
    }
}
