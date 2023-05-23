mod arg_ext;
#[macro_use]
mod macros;
mod command;
pub mod keyspace;
mod logger;
pub mod store;
pub mod timer;

use std::os::raw::{c_char, c_int};

pub use redis_module as rm;
pub use once_cell::sync::OnceCell as __OnceCell;
pub use arg_ext::{FromArgs, NextArgExt};
pub use command::{CommandKeys, Commands, Handler};
// reexport
pub use redis_module::{
    Context,
    NextArg,
    RedisError,
    RedisResult,
    RedisString,
    RedisValue,
    REDIS_OK,
};

use crate::store::{DataTypes, Stores};

pub trait Config: TryFrom<Vec<RedisString>, Error = RedisError> {
    fn validate(&self) -> Result<(), ()> {
        Ok(())
    }
}

impl<R> Config for R where R: TryFrom<Vec<RedisString>, Error = RedisError> {}

pub trait Instance<M: Module> {
    fn set(module: M);
    fn get() -> Option<&'static M>;
}

pub trait Module: Sized + Instance<Self> {
    const VERSION: i32;
    const NAME: &'static str;

    type Error: std::error::Error;
    type Config: Config;
    type Commands: Commands<Self>;
    type DataTypes: DataTypes;
    type KeyspaceEvents;

    fn stop(&self, ctx: &Context) -> Result<(), Self::Error>;

    fn start(&mut self, ctx: &Context) -> Result<(), Self::Error>;

    fn create(
        ctx: &Context,
        config: Self::Config,
        stores: store::ModuleStores<Self>,
    ) -> Result<Self, Self::Error>;
}

pub fn on_unload<M: Module + 'static>(ctx: *mut rm::RedisModuleCtx) -> rm::Status {
    let instance = match M::get() {
        Some(instance) => instance,
        None => return rm::Status::Err,
    };

    let ctx = &Context::new(ctx);

    match instance.stop(ctx) {
        Ok(_) => rm::Status::Ok,
        Err(_) => rm::Status::Err,
    }
}

pub fn on_load<M: Module + 'static>(
    ctx: *mut rm::RedisModuleCtx,
    argv: *mut *mut rm::RedisModuleString,
    argc: c_int,
) -> rm::Status {
    let ctx = &Context::new(ctx);

    if let Err(err) = module_init::<M>(ctx) {
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

    if let Err(err) = M::Commands::register(ctx) {
        log::error!("cmds register failed: {:?}", err);

        return rm::Status::Err;
    }

    if let Err(err) = module.start(ctx) {
        log::error!("module start failed: {:?}", err);

        return rm::Status::Err;
    }

    M::set(module);

    rm::Status::Ok
}

fn module_init<M: Module>(ctx: &Context) -> Result<(), ()> {
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

    let module_name = buffer_name_ptr.cast_const().cast::<c_char>();
    let module_version = M::VERSION as c_int;
    let module_api_version = redis_module::REDISMODULE_APIVER_1 as c_int;

    let status = rm::Status::from(unsafe {
        rm::raw::Export_RedisModule_Init(ctx.ctx, module_name, module_version, module_api_version)
    });

    match status {
        rm::Status::Ok => Ok(()),
        rm::Status::Err => Err(()),
    }
}
