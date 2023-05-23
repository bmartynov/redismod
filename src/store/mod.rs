use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use redis_module as rm;
use redis_module::{
    key::{RedisKey, RedisKeyWritable},
    raw,
    RedisString,
};
pub use io::{IOLoader, IOSaver, Loader, Saver};
pub use types::{DataTypes, RDBLoadSave, Type, TypeMethods};

use crate::Module;

mod io;
mod types;

pub trait Stores {
    fn register(&self, _ctx: &rm::Context) -> Result<(), &str>;
}

impl Stores for () {
    fn register(&self, _ctx: &rm::Context) -> Result<(), &str> {
        Ok(())
    }
}

pub type ModuleStores<M> = <<M as Module>::DataTypes as DataTypes>::Stores;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entity not found")]
    NotFound,
    #[error("redis error")]
    Redis(rm::RedisError),
}

pub struct Store<T> {
    marker: PhantomData<T>,
    redis_type: rm::native_types::RedisType,
}

unsafe impl<T: Type> Send for Store<T> {}

impl<T> Clone for Store<T> {
    fn clone(&self) -> Self {
        Self {
            marker: Default::default(),
            redis_type: self.redis_type.clone(),
        }
    }
}

impl<T: Type> Default for Store<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            redis_type: T::redis_type(),
        }
    }
}

pub struct Entry<'s, T> {
    entry: &'s T,
    _key: RedisKey,
}

impl<T> Deref for Entry<'_, T> {
    type Target = T;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.entry
    }
}

pub struct EntryMut<'s, T> {
    entry: &'s mut T,
    _key: RedisKeyWritable,
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

impl<T: Type + TypeMethods> Store<T> {
    pub fn new() -> Self {
        let redis_type = T::redis_type();

        Self {
            redis_type,
            marker: PhantomData,
        }
    }

    #[inline]
    pub fn load(&self, ctx: &rm::Context, id: &T::IDType) -> Result<Entry<T>, Error> {
        let raw_key = T::key(id);

        let key = ctx.open_key(&ctx.create_string(&raw_key));

        let value =
            unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(key.key_inner()).cast::<T>() };

        if value.is_null() {
            return Err(Error::NotFound);
        }

        Ok(Entry {
            _key: key,
            entry: unsafe { &*value },
        })
    }

    #[inline]
    pub fn load_key(&self, ctx: &rm::Context, key: &RedisString) -> Result<Entry<T>, Error> {
        let key = ctx.open_key(key);

        let value =
            unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(key.key_inner()).cast::<T>() };

        if value.is_null() {
            return Err(Error::NotFound);
        }

        Ok(Entry {
            _key: key,
            entry: unsafe { &*value },
        })
    }

    #[inline]
    pub fn load_mut(&self, ctx: &rm::Context, id: &T::IDType) -> Result<EntryMut<T>, Error> {
        let raw_key = T::key(id);

        let key = ctx.open_key_writable(&ctx.create_string(&raw_key));

        let value =
            unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(key.key_inner()).cast::<T>() };

        if value.is_null() {
            return Err(Error::NotFound);
        }

        Ok(EntryMut {
            _key: key,
            entry: unsafe { &mut *value },
        })
    }

    #[inline]
    pub fn load_mut_key(&self, ctx: &rm::Context, key: &RedisString) -> Result<EntryMut<T>, Error> {
        let key = ctx.open_key_writable(&key);

        let value =
            unsafe { raw::RedisModule_ModuleTypeGetValue.unwrap()(key.key_inner()).cast::<T>() };

        if value.is_null() {
            return Err(Error::NotFound);
        }

        Ok(EntryMut {
            _key: key,
            entry: unsafe { &mut *value },
        })
    }

    #[inline]
    pub fn store(&self, ctx: &rm::Context, id: &T::IDType, value: T) -> Result<(), Error> {
        let raw_key = T::key(id);

        let key = ctx.open_key_writable(&ctx.create_string(&raw_key));

        key.set_value(&self.redis_type, value)
            .map_err(Error::Redis)?;

        Ok(())
    }

    #[inline]
    pub fn store_key(&self, ctx: &rm::Context, key: &RedisString, value: T) -> Result<(), Error> {
        let key = ctx.open_key_writable(key);

        key.set_value(&self.redis_type, value)
            .map_err(Error::Redis)?;

        Ok(())
    }

    #[inline]
    pub fn exists(&self, ctx: &rm::Context, id: &T::IDType) -> Result<bool, Error> {
        let raw_key = T::key(id);
        let key = ctx.open_key(&ctx.create_string(&raw_key));

        Ok(!key.is_null())
    }

    pub fn register(&self, ctx: &rm::Context) -> Result<(), &str> {
        self.redis_type.create_data_type(ctx.ctx)?;

        Ok(())
    }
}
