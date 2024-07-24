#[derive(Debug)]
pub struct Completion(pub(crate) *mut u32);

impl<'a> Completion {
    pub fn new(location: *mut u32) -> Self {
        unsafe { (location as *mut u32).write_volatile(0); }
        Self(location)
    }

    pub fn reset(&mut self) {
        unsafe { self.0.write_volatile(0); }
    }

    pub fn poll(&mut self) -> bool {
        unsafe { self.0.read_volatile() != 0 }
    }
}
