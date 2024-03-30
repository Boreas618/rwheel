use atomic_wait::{wait, wake_one};
use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicU32, Ordering};

pub struct Mutex<T> {
    locked: AtomicU32,
    value: UnsafeCell<T>,
}

unsafe impl<T> Sync for Mutex<T> {}
unsafe impl<T> Send for Mutex<T> {}

pub struct MutexGuard<'a, T> {
    mutex: &'a Mutex<T>,
}

impl<T> Deref for MutexGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &T {
        unsafe { &*self.mutex.value.get() }
    }
}

impl<T> DerefMut for MutexGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        unsafe { &mut *self.mutex.value.get() }
    }
}

impl<T> Mutex<T> {
    pub const fn new(value: T) -> Self {
        Self {
            locked: AtomicU32::new(0),
            value: UnsafeCell::new(value),
        }
    }

    pub fn lock(&self) -> MutexGuard<T> {
        while self.locked.swap(1, Ordering::Acquire) == 1 {
            wait(&self.locked, 1);
        }

        MutexGuard { mutex: self }
    }

    pub fn unlock(&self) {
        wake_one(&self.locked);
    }
}

impl<T> Drop for MutexGuard<'_, T> {
    fn drop(&mut self) {
        self.mutex.locked.store(0, Ordering::Release);

        // In fact we can wake multiple threads up, but it's nonsense in that
        // only one of them would finally get the mutex while others simply
        // waste the CPU cycles.
        wake_one(&self.mutex.locked);
    }
}
