mod arg_ext;
#[macro_use]
mod macros;
mod logger;
mod redis_io;
mod requests;
mod store;

use std::{
    marker::PhantomData,
    os::raw::{c_char, c_int},
};

use redis_module as rm;

pub use once_cell::sync::OnceCell as __OnceCell;

pub use arg_ext::{FromArgs, NextArgExt};

pub use redis_io::{IOLoader, IOSaver, Loader, Saver};

pub use requests::{Command, CommandKeys, RequestHandler, Requests};

pub use store::{Entry, EntryMut, Error, ModuleStores, Store, Stores, Type, Types};

pub trait Config: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {
    fn validate(&self) -> Result<(), ()> {
        Ok(())
    }
}

impl<R> Config for R where R: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {}

pub trait InstanceMngr<M: Module> {
    fn set(module: M);
    fn get() -> Option<&'static M>;
}

pub trait Module: Sized {
    const VERSION: i32;
    const NAME: &'static str;

    type Error: std::error::Error;
    type Config: Config;
    type Requests: Requests<Self>;
    type DataTypes: Types;

    fn stop(&self, _ctx: &rm::Context) -> Result<(), Self::Error> {
        Ok(())
    }

    fn start(&mut self, _ctx: &rm::Context) -> Result<(), Self::Error> {
        Ok(())
    }

    fn create(
        ctx: &rm::Context,
        config: Self::Config,
        stores: ModuleStores<Self>,
    ) -> Result<Self, Self::Error>;
}

pub struct Instance<M, G>
where
    M: Module,
    G: InstanceMngr<M>,
{
    marker: PhantomData<(M, G)>,
}

impl<M, G> Instance<M, G>
where
    M: Module + 'static,
    G: InstanceMngr<M>,
{
    pub fn on_load(
        ctx: *mut rm::RedisModuleCtx,
        argv: *mut *mut rm::RedisModuleString,
        argc: c_int,
    ) -> rm::Status {
        let ctx = &rm::Context::new(ctx);

        if let Err(err) = Self::module_init(ctx) {
            log::error!("module init failed: {:?}", err);

            return rm::Status::Err;
        }

        if logger::setup().is_err() {
            return rm::Status::Err;
        }

        let args = rm::decode_args(ctx.ctx, argv, argc);

        let config = match M::Config::try_from(args) {
            Ok(config) => config,
            Err(err) => {
                log::error!("cannot parse config: {:?}", err);

                return rm::Status::Err;
            }
        };

        if let Err(err) = Config::validate(&config) {
            log::error!("config validate failed: {:?}", err);

            return rm::Status::Err;
        }

        let stores = M::DataTypes::create();

        if let Err(err) = stores.register(ctx) {
            log::error!("cannot register data type: {:?}", err);

            return rm::Status::Err;
        }

        let mut module = match M::create(ctx, config, stores) {
            Ok(module) => module,
            Err(err) => {
                log::error!("module create failed: {:?}", err);

                return rm::Status::Err;
            }
        };

        if let Err(err) = M::Requests::register::<G>(ctx) {
            log::error!("requests register failed: {:?}", err);

            return rm::Status::Err;
        }

        if let Err(err) = module.start(ctx) {
            log::error!("module start failed: {:?}", err);

            return rm::Status::Err;
        }

        G::set(module);

        rm::Status::Ok
    }

    pub fn on_unload(ctx: *mut rm::RedisModuleCtx) -> rm::Status {
        let instance = match G::get() {
            Some(instance) => instance,
            None => return rm::Status::Err,
        };

        let ctx = &rm::Context::new(ctx);

        match instance.stop(ctx) {
            Ok(_) => rm::Status::Ok,
            Err(_) => rm::Status::Err,
        }
    }

    fn module_init(ctx: &rm::Context) -> Result<(), ()> {
        // We use a statically sized buffer to avoid allocating.
        // This is needed since we use a custom allocator that relies on the Redis allocator,
        // which isn't yet ready at this point.
        let mut name_buffer = [0; 64];

        let module_name_ptr = M::NAME.as_ptr();
        let module_name_len = M::NAME.len();

        let buffer_name_ptr = name_buffer.as_mut_ptr();

        unsafe {
            std::ptr::copy(module_name_ptr, buffer_name_ptr, module_name_len);
        }

        let module_name = buffer_name_ptr as *const c_char;
        let module_version = M::VERSION as c_int;
        let module_api_version = redis_module::REDISMODULE_APIVER_1 as c_int;

        let status = rm::Status::from(unsafe {
            rm::raw::Export_RedisModule_Init(
                ctx.ctx,
                module_name,
                module_version,
                module_api_version,
            )
        });

        match status {
            rm::Status::Ok => Ok(()),
            rm::Status::Err => Err(()),
        }
    }
}
