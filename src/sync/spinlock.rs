use core::cell::UnsafeCell;
use core::ops::{Deref, DerefMut};
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    /// `UnsafeCell<T>` enables mutation of data even through an immutable
    /// reference (`&T`), normally prohibited in Rust to maintain safety.
    /// This feature allows `&UnsafeCell<T>` to point to data that may be
    /// mutated concurrently. The `SpinLock` leverages this to safely control
    /// access to the contained data across multiple threads without violating
    /// Rust's borrowing rules.
    ///
    /// Note that only the immutability guarantee for shared references is
    /// affected by UnsafeCell. The uniqueness guarantee for mutable
    /// references is unaffected. There is no legal way to obtain aliasing
    /// &mut, not even with UnsafeCell<T>.
    data: UnsafeCell<T>,
    locked: AtomicBool,
}

impl<T> SpinLock<T> {
    pub const fn new(data: T) -> Self {
        Self {
            data: UnsafeCell::new(data),
            locked: AtomicBool::new(false),
        }
    }

    pub fn try_acquire(&self) -> bool {
        self.locked
            .compare_exchange(false, true, Ordering::Acquire, Ordering::Relaxed)
            .is_ok()
    }

    pub fn acquire(&self) -> SpinLockGuard<T> {
        while !self.try_acquire() {
            core::hint::spin_loop();
        }

        SpinLockGuard { lock: self }
    }

    pub fn release(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct SpinLockGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> Drop for SpinLockGuard<'_, T> {
    fn drop(&mut self) {
        self.lock.release();
    }
}

impl<T> Deref for SpinLockGuard<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*self.lock.data.get() }
    }
}

impl<T> DerefMut for SpinLockGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *self.lock.data.get() }
    }
}

/// A type is considered `Send` if it can be safely transferred to another 
/// thread. Similarly, a type is deemed `Sync` if it can be safely shared
/// among threads. In this context, by utilizing `AtomicBool` for managing
/// access to the spinlock, we are enabled to implement both `Sync` and 
/// `Send` for `SpinLock<T>`.
unsafe impl<T> Sync for SpinLock<T> {}
unsafe impl<T> Send for SpinLock<T> {}
