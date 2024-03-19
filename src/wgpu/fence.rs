use std::{
    sync::{Condvar, Mutex},
    time::Duration,
};

use wgpu_hal::FenceValue;

#[derive(Debug)]
pub struct Fence {
    mutex: Mutex<FenceValue>,
    condvar: Condvar,
}
impl Fence {
    #[must_use]
    pub fn new() -> Self {
        Self {
            mutex: Mutex::new(0),
            condvar: Condvar::new(),
        }
    }

    /// # Panics
    /// Panics if the contained mutex has been poisoned
    pub fn signal(&self, value: FenceValue) {
        *self.mutex.lock().unwrap() = value;
        self.condvar.notify_all();
    }

    /// # Panics
    /// Panics if the contained mutex has been poisoned
    pub fn wait(&self, value: FenceValue, timeout: u32) -> bool {
        let lock = self.mutex.lock().unwrap();
        if *lock == value {
            return true;
        }

        let (_guard, result) = self
            .condvar
            .wait_timeout_while(lock, Duration::from_millis(timeout as u64), |val| {
                *val >= value
            })
            .unwrap();

        !result.timed_out()
    }

    /// # Panics
    /// Panics if the contained mutex has been poisoned
    pub fn get_value(&self) -> FenceValue {
        *self.mutex.lock().unwrap()
    }
}
impl Default for Fence {
    fn default() -> Self {
        Self::new()
    }
}
