use std::{ffi, fmt, ptr};

use redis_module::{error::Error, native_types, raw, Context};

use crate::store::{Store, Stores};
use super::{IOLoader, IOSaver};

pub trait Type: Sized + RDBLoadSave {
    type IDType: fmt::Display;

    const NAME: &'static str;
    const PREFIX: &'static str;

    const REDIS_NAME: &'static str;
    const REDIS_VERSION: i32;

    const KEY_PREFIX: &'static str;

    #[inline]
    fn key(id: &Self::IDType) -> String {
        format!("{}{}", Self::KEY_PREFIX, id)
    }

    fn free(_value: Box<Self>) {}
    fn mem_usage(value: &Self) -> usize;
}

pub trait RDBLoadSave: Sized {
    fn rdb_save(saver: &IOSaver, value: &Self);
    fn rdb_load(loader: &IOLoader, encver: usize) -> Result<Self, Error>;
}

pub trait DataTypes: Sized {
    type Stores: Stores;

    fn create() -> Self::Stores;
}

impl DataTypes for () {
    type Stores = ();

    fn create() -> Self::Stores {}
}

pub unsafe trait TypeMethods {
    fn redis_type() -> native_types::RedisType;

    unsafe extern "C" fn free(value: *mut ffi::c_void);
    unsafe extern "C" fn mem_usage(value: *const ffi::c_void) -> usize;
    unsafe extern "C" fn rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut ffi::c_void);
    unsafe extern "C" fn rdb_load(
        rdb: *mut raw::RedisModuleIO,
        encver: ffi::c_int,
    ) -> *mut ffi::c_void;
}

unsafe impl<T: Type> TypeMethods for T {
    fn redis_type() -> native_types::RedisType {
        let version: u64 = raw::REDISMODULE_TYPE_METHOD_VERSION.into();

        let free: raw::RedisModuleTypeFreeFunc = Some(<Self as TypeMethods>::free);
        let rdb_load: raw::RedisModuleTypeLoadFunc = Some(<Self as TypeMethods>::rdb_load);
        let rdb_save: raw::RedisModuleTypeSaveFunc = Some(<Self as TypeMethods>::rdb_save);

        let type_methods = raw::RedisModuleTypeMethods {
            version,
            free,
            rdb_load,
            rdb_save,
            aof_rewrite: None,
            // Currently unused by Redis
            mem_usage: None,
            digest: None,
            // Aux data
            aux_load: None,
            aux_save: None,
            aux_save_triggers: 0,
            free_effort: None,
            unlink: None,
            copy: None,
            defrag: None,

            copy2: None,
            unlink2: None,
            free_effort2: None,
            mem_usage2: None,
        };

        native_types::RedisType::new(Self::REDIS_NAME, Self::REDIS_VERSION, type_methods)
    }

    unsafe extern "C" fn free(value: *mut ffi::c_void) {
        let value = Box::from_raw(value.cast::<T>());

        T::free(value);
    }

    unsafe extern "C" fn mem_usage(value: *const ffi::c_void) -> usize {
        let value = &*value.cast::<T>();

        T::mem_usage(value)
    }

    unsafe extern "C" fn rdb_save(rdb: *mut raw::RedisModuleIO, value: *mut ffi::c_void) {
        let saver = IOSaver { rdb };
        let value = &*value.cast::<T>();

        T::rdb_save(&saver, value)
    }

    unsafe extern "C" fn rdb_load(
        rdb: *mut raw::RedisModuleIO,
        encver: ffi::c_int,
    ) -> *mut ffi::c_void {
        let loader = IOLoader { rdb };

        let loaded = match T::rdb_load(&loader, encver as usize) {
            Ok(loaded) => loaded,
            Err(_err) => return ptr::null_mut(),
        };

        Box::into_raw(Box::new(loaded)).cast::<ffi::c_void>()
    }
}

// adapted from core/src/fmt/cmds tuple
macro_rules! tuple_types {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<$($name: Type, )*> Stores for ($(Store<$name>, )*) {
            fn register(&self, ctx: &Context) -> Result<(), &str> {
                #[allow(non_snake_case)]
                let ($(ref $name,)+) = self;

                $( $name.register(ctx)?; )+

                Ok(())
            }
        }

        impl<$($name: Type, )*> DataTypes for ($($name, )*) {
            type Stores = ($(Store<$name>, )*);

            fn create() -> Self::Stores {
                ($(Store::<$name>::new(), )*)
            }
        }

        // skip first element and call macro again
        peel_types! { $($name,)+ }
    )
}

macro_rules! peel_types {
    ($name:ident, $($other:ident,)*) => (tuple_types! { $($other,)* })
}

tuple_types![T1, T2, T3, T4, T5, T6, T7, T8, T9, T10, T11, T12, T13, T14, T15, T16,];
