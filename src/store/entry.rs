use std::ffi::c_void;
use std::fmt;
use std::ops::{Deref, DerefMut};

use redis_module::key::{RedisKey, RedisKeyWritable};

pub struct Entry<'s, T> {
    pub(crate) entry: &'s T,
    pub(crate) _key: RedisKey,
}

impl<'s, T> Entry<'s, T> {
    pub(crate) unsafe fn from_ptr(key: RedisKey, ptr: *mut c_void) -> Self {
        // cast pointer back
        let value = unsafe { &*ptr.cast::<T>() };

        Self {
            _key: key,
            entry: value,
        }
    }
}

impl<T> Deref for Entry<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.entry
    }
}

impl<T: fmt::Debug> fmt::Debug for Entry<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.entry.fmt(f)
    }
}

pub struct EntryMut<'s, T> {
    pub(crate) entry: &'s mut T,
    pub(crate) key: RedisKeyWritable,
}

impl<'s, T> EntryMut<'s, T> {
    pub(crate) unsafe fn from_ptr(key: RedisKeyWritable, ptr: *mut c_void) -> Self {
        // cast pointer back
        let value = unsafe { &mut *ptr.cast::<T>() };

        Self { key: key, entry: value }
    }
}

impl<T> Deref for EntryMut<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.entry
    }
}

impl<T> DerefMut for EntryMut<'_, T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.entry
    }
}

impl<T> EntryMut<'_, T> {
    pub fn delete(self) -> Result<(), super::Error> {
        self.key.delete().map(|_| ())
    }
}
