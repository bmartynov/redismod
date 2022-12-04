mod types;

use std::marker::PhantomData;

use redis_module as rm;
use redis_module::Context;

pub use types::{Type, TypeMethods, Types};

use crate::Module;

pub trait Stores {
    fn register(&self, _ctx: &rm::Context) -> Result<(), &str>;
}

impl Stores for () {
    fn register(&self, _ctx: &Context) -> Result<(), &str> {
        Ok(())
    }
}

pub type ModuleStores<M> = <<M as Module>::DataTypes as Types>::Stores;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("entity not found")]
    NotFound,
    #[error("redis error: {0}")]
    Redis(rm::RedisError),
}

pub struct Store<T> {
    marker: PhantomData<T>,
    redis_type: rm::native_types::RedisType,
}

unsafe impl<T: Type> Send for Store<T> {}

impl<T: Type> Default for Store<T> {
    fn default() -> Self {
        Self {
            marker: PhantomData,
            redis_type: T::redis_type(),
        }
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

    pub fn exists(&self, ctx: &rm::Context, id: &T::IDType) -> Result<bool, Error> {
        self.get(ctx, id).exists()
    }

    pub fn get(&self, ctx: &rm::Context, id: &T::IDType) -> Entry<T> {
        let raw_key = format!("{}:{}:{}", T::NAME, T::PREFIX, id);

        let key = ctx.open_key(&ctx.create_string(&raw_key));

        Entry {
            key,
            marker: PhantomData,
            redis_type: &self.redis_type,
        }
    }

    pub fn get_mut(&self, ctx: &rm::Context, id: &T::IDType) -> EntryMut<T> {
        let raw_key = format!("{}:{}:{}", T::NAME, T::PREFIX, id);

        let key = ctx.open_key_writable(&ctx.create_string(&raw_key));

        EntryMut {
            key,
            marker: PhantomData,
            redis_type: &self.redis_type,
        }
    }

    pub fn register(&self, ctx: &rm::Context) -> Result<(), &str> {
        self.redis_type.create_data_type(ctx.ctx)?;

        Ok(())
    }
}

pub struct Entry<'s, T: Type> {
    marker: PhantomData<&'s T>,
    key: rm::key::RedisKey,
    redis_type: &'s rm::native_types::RedisType,
}

impl<'s, T: Type> Entry<'s, T> {
    pub fn exists(&self) -> Result<bool, Error> {
        Ok(self.key.key_type() != rm::KeyType::Empty)
    }

    pub fn load(&self) -> Result<&T, Error> {
        self.key
            .get_value::<T>(self.redis_type)
            .map_err(Error::Redis)?
            .ok_or(Error::NotFound)
    }
}

pub struct EntryMut<'s, T: Type> {
    marker: PhantomData<&'s T>,
    key: rm::key::RedisKeyWritable,
    redis_type: &'s rm::native_types::RedisType,
}

impl<'s, T: Type> EntryMut<'s, T> {
    pub fn exists(&self) -> Result<bool, Error> {
        Ok(self.key.key_type() != rm::KeyType::Empty)
    }

    pub fn load(&self) -> Result<&mut T, Error> {
        self.key
            .get_value::<T>(self.redis_type)
            .map_err(Error::Redis)?
            .ok_or(Error::NotFound)
    }

    pub fn store(&self, value: T) -> Result<(), Error> {
        self.key
            .set_value::<T>(self.redis_type, value)
            .map_err(Error::Redis)
    }

    pub fn delete(&self) -> Result<(), Error> {
        self.key.delete().map_err(Error::Redis).map(|_| ())
    }
}
