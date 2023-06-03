use std::ffi::c_void;
use std::marker::PhantomData;
use std::time::Duration;

pub use entry::{Entry, EntryMut};
pub use io::{IOLoader, IOSaver, Loader, Saver};
pub use rdb::RDBLoadSave;
use redis_module::key::{RedisKey, RedisKeyWritable};
use redis_module::{native_types, raw, Context, RedisError, RedisString};
pub use types::{AsKey, DataTypes, Type, TypeMethods};

use crate::Module;

mod entry;
mod io;
mod rdb;
mod types;

pub trait Stores {
    fn register(&self, _ctx: &Context) -> Result<(), &str>;
}

impl Stores for () {
    fn register(&self, _ctx: &Context) -> Result<(), &str> {
        Ok(())
    }
}

pub type ModuleStores<M> = <<M as Module>::DataTypes as DataTypes>::Stores;

pub type Error = RedisError;

pub struct Store<T> {
    marker: PhantomData<T>,
    pub redis_type: native_types::RedisType,
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

impl<T: Type> Store<T> {
    pub fn new() -> Self {
        Self::default()
    }
}

/// get* methods
impl<T: Type> Store<T> {
    /// get: returns value by key.
    ///
    /// key: T::ID/RedisString
    ///
    pub fn get(&self, ctx: &Context, key: impl AsKey) -> Result<Option<Entry<T>>, Error> {
        let key = key.as_key(ctx);
        let key = ctx.open_key(key.as_ref());

        self.get_value(key)
    }

    /// get: returns mut value by key.
    ///
    /// key: T::ID/RedisString
    ///
    pub fn get_mut(&self, ctx: &Context, key: impl AsKey) -> Result<Option<EntryMut<T>>, Error> {
        let key = key.as_key(ctx);
        let key = ctx.open_key_writable(key.as_ref());

        self.get_value_mut(key)
    }
}

/// exists* methods
impl<T: Type> Store<T> {
    pub fn exists(&self, ctx: &Context, key: impl AsKey) -> Result<bool, Error> {
        let key = key.as_key(ctx);

        self.exists_by_key(ctx, &key)
    }

    pub fn exists_by_key(&self, ctx: &Context, key: &RedisString) -> Result<bool, Error> {
        let value = unsafe { raw::RedisModule_KeyExists.unwrap()(ctx.ctx, key.inner) };

        Ok(value != 0)
    }
}

/// store* methods
impl<T: Type> Store<T> {
    pub fn store(&self, ctx: &Context, value: T, ttl: Option<Duration>) -> Result<(), Error> {
        let key = value.id();
        let key = key.as_key(ctx);
        let key = ctx.open_key_writable(key.as_ref());

        key.set_value(&self.redis_type, value)?;

        if let Some(ttl) = ttl {
            key.set_expire(ttl)?;
        }

        Ok(())
    }
}

/// create* methods
impl<T: Type> Store<T> {
    pub fn create(&self, ctx: &Context, value: T, ttl: Option<Duration>) -> Result<EntryMut<T>, Error> {
        let key = value.id();
        let key = key.as_key(ctx);
        let key = ctx.open_key_writable(key.as_ref());

        if !key.is_empty() {
            return Err(RedisError::Str("already exists"));
        }

        // move value to heap, cast to `c_void`
        let ptr = Box::into_raw(Box::new(value)).cast::<c_void>();

        // store value into redis
        unsafe { self.raw_set_value(&key, ptr) }?;

        // set expire time
        if let Some(ttl) = ttl {
            key.set_expire(ttl)?;
        }

        Ok(unsafe { EntryMut::from_ptr(key, ptr) })
    }
}

/// get_or_create
impl<T: Type> Store<T> {
    pub fn get_or_create(
        &self,
        ctx: &Context,
        id: &T::ID,
        f: impl FnOnce() -> T,
        ttl: Option<Duration>,
    ) -> Result<(EntryMut<T>, bool), Error> {
        let key = id.as_key(ctx);
        let key = ctx.open_key_writable(key.as_ref());

        match unsafe { self.raw_get_value(key.key_inner()) }? {
            // value at key already exists
            Some(ptr) => Ok((unsafe { EntryMut::from_ptr(key, ptr) }, false)),
            // value at key does not exists
            None => {
                // move value to heap, cast to `c_void`
                let ptr = Box::into_raw(Box::new(f())).cast::<c_void>();

                // store value into redis
                unsafe { self.raw_set_value(&key, ptr) }?;

                // set expire time
                if let Some(ttl) = ttl {
                    key.set_expire(ttl)?;
                }

                let entry = unsafe { EntryMut::from_ptr(key, ptr) };

                Ok((entry, true))
            }
        }
    }
}

impl<T: Type> Store<T> {
    pub(crate) fn register(&self, ctx: &Context) -> Result<(), &str> {
        self.redis_type.create_data_type(ctx.ctx)?;

        Ok(())
    }

    fn get_value(&self, key: RedisKey) -> Result<Option<Entry<T>>, Error> {
        match unsafe { self.raw_get_value(key.key_inner()) }? {
            None => Ok(None),
            Some(ptr) => Ok(Some(unsafe { Entry::from_ptr(key, ptr) })),
        }
    }

    fn get_value_mut(&self, key: RedisKeyWritable) -> Result<Option<EntryMut<T>>, Error> {
        match unsafe { self.raw_get_value(key.key_inner()) }? {
            None => Ok(None),
            Some(ptr) => Ok(Some(unsafe { EntryMut::from_ptr(key, ptr) })),
        }
    }

    /// verify_type: checks key type
    ///
    /// # Safety: user should be shure that key is not empty
    unsafe fn verify_type(&self, key: *mut raw::RedisModuleKey) -> Result<(), Error> {
        let expected_type = self
            .redis_type
            .raw_type
            .get()
            .ok_or_else(|| RedisError::Str("type not initialized"))?;

        // The key exists; check its type
        let raw_type = raw::RedisModule_ModuleTypeGetType.unwrap()(key);

        if raw_type != *expected_type {
            return Err(RedisError::Str("Existing key has wrong Redis type"));
        }

        Ok(())
    }

    /// raw_get_value: gets value without checking key type
    ///
    /// # Safety: user should be shure that key is empty or key type correct
    unsafe fn raw_get_value<'k>(&self, key: *mut raw::RedisModuleKey) -> Result<Option<*mut c_void>, Error> {
        let value = raw::RedisModule_ModuleTypeGetValue.unwrap()(key);

        if value.is_null() {
            return Ok(None);
        }

        self.verify_type(key)?;

        Ok(Some(value))
    }

    /// raw_set_value: sets value without checking key type
    ///
    /// # Safety: user should be shure that key is empty or key type correct
    unsafe fn raw_set_value(&self, key: &RedisKeyWritable, ptr: *mut c_void) -> Result<(), Error> {
        let key_inner = key.key_inner();

        let raw_type = self
            .redis_type
            .raw_type
            .get()
            .ok_or(RedisError::Str("type not initialized"))?;

        let status: raw::Status = raw::RedisModule_ModuleTypeSetValue.unwrap()(key_inner, *raw_type, ptr).into();

        status.into()
    }
}
