use std::sync::{Mutex, MutexGuard};

pub trait MutexLockSExt<T: ?Sized> {
    fn lock_s(&self) -> Result<MutexGuard<'_, T>, String>;
}

impl<T: ?Sized> MutexLockSExt<T> for Mutex<T> {
    /// Return String error instead of PoisonError.
    ///
    /// # Examples
    ///
    /// ```
    /// use lazy_crafter::utils::sync_ext::MutexLockSExt;
    /// use std::sync::Mutex;
    ///
    /// let mutex = Mutex::new(0);
    /// let guarded_value = mutex.lock_s()?;
    /// assert_eq!(*guarded_value, 0);
    /// # Ok::<(), String>(())
    /// ```
    fn lock_s(&self) -> Result<MutexGuard<'_, T>, String> {
        self.lock().map_err(|x| x.to_string())
    }
}
