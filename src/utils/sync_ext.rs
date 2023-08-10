use std::sync::{Mutex, MutexGuard};

pub trait MutexLockSExt<T: ?Sized> {
    fn lock_s(&self) -> Result<MutexGuard<T>, String>;
}

impl<T: ?Sized> MutexLockSExt<T> for Mutex<T> {
    /// Return String error instead of PoisonError.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_crafter::utils::sync_ext::MutexLockSExt;
    ///
    /// let mutex = Mutex::new(0);
    /// let guarded_value = mutex.lock_s()?;
    /// ```
    fn lock_s(&self) -> Result<MutexGuard<T>, String> {
        self.lock().map_err(|x| x.to_string())
    }
}
