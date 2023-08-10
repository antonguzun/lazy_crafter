use std::sync::{Mutex, MutexGuard};

pub trait MutexLockSExt<T: ?Sized> {
    fn lock_s(&self) -> Result<MutexGuard<T>, String>;
}

impl<T: ?Sized> MutexLockSExt<T> for Mutex<T> {
    fn lock_s(&self) -> Result<MutexGuard<T>, String> {
        self.lock().map_err(|x| x.to_string())
    }
}
