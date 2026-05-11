use std::sync::{Mutex, MutexGuard};

#[derive(Debug, Clone)]
pub enum NameValidation {
    Empty,
    Reserved,
    Used,
    IllegalChar(char),

    Valid(String),
}

pub trait LockClean<'a, T> {
    fn lock_mutex(&'a self) -> MutexGuard<'a, T>;
}

impl<'a, T> LockClean<'a, T> for Mutex<T> {
    fn lock_mutex(&'a self) -> MutexGuard<'a, T> {
        // i can get poisoning data (not complete)
        //for now, just returns the data even though she might be currepted
        self.lock().unwrap_or_else(|e| e.into_inner())
    }
}
