use std::{ffi, ffi::CString};

use redis_module::{decode_args, raw, Context, RedisError, RedisResult, RedisString, Status};

use crate::Module;

pub trait TryFromArgs: TryFrom<Vec<RedisString>, Error = RedisError> {
    fn validate(&self) -> Result<(), RedisError> {
        Ok(())
    }
}

impl<T> TryFromArgs for T where T: TryFrom<Vec<RedisString>, Error = RedisError> {}

pub trait Handler<R: TryFromArgs>: Module {
    const NAME: &'static str;
    const FLAGS: &'static str;
    const KEYS: CommandKeys;

    type Result: Into<RedisResult>;

    fn handle(&self, ctx: &Context, req: R) -> Self::Result;
}

pub struct CommandKeys {
    pub first: u8,
    pub last: u8,
    pub step: u8,
}

impl From<CommandKeys> for (ffi::c_int, ffi::c_int, ffi::c_int) {
    fn from(value: CommandKeys) -> Self {
        (
            value.first as ffi::c_int,
            value.last as ffi::c_int,
            value.step as ffi::c_int,
        )
    }
}

pub trait Commands<M: Module> {
    fn register(ctx: &Context) -> Result<(), RedisError>;
}

impl<M: Module> Commands<M> for () {
    fn register(_ctx: &Context) -> Result<(), RedisError> {
        Ok(())
    }
}

fn command_register<M, R>(ctx: &Context) -> Result<(), RedisError>
where
    M: 'static,
    M: Handler<R>,
    R: TryFromArgs,
{
    let module_name = <M as Module>::NAME;
    let command_name = <M as Handler<R>>::NAME;
    let command_name_full = format!("{}.{}", module_name, command_name);

    let keys = <M as Handler<R>>::KEYS;
    let name = CString::new(command_name_full).unwrap();
    let flags = CString::new(<M as Handler<R>>::FLAGS).unwrap();

    let (key_first, key_last, key_step) = keys.into();

    let status = Status::from(unsafe {
        raw::RedisModule_CreateCommand.unwrap()(
            ctx.ctx,
            name.as_ptr(),
            Some(do_command::<M, R>),
            flags.as_ptr(),
            key_first,
            key_last,
            key_step,
        )
    });

    status.into()
}

#[inline]
extern "C" fn do_command<M, R>(
    ctx: *mut raw::RedisModuleCtx,
    argv: *mut *mut raw::RedisModuleString,
    argc: ffi::c_int,
) -> ffi::c_int
where
    M: 'static,
    M: Handler<R>,
    R: TryFromArgs,
{
    let ctx = &Context::new(ctx);
    let args = decode_args(ctx.ctx, argv, argc);

    let instance = match M::get() {
        Some(instance) => instance,
        None => return ctx.reply_error_string("instance missed") as ffi::c_int,
    };

    let req = match R::try_from(args) {
        Ok(req) => req,
        Err(err) => return ctx.reply(Err(err)) as ffi::c_int,
    };

    let result = instance.handle(ctx, req);

    ctx.reply(result.into()) as ffi::c_int
}

// adapted from core/src/fmt/cmds tuple
macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<M: 'static, $($name, )*> Commands<M> for ($($name, )*)
        where
            M: Module,
            $(
            $name: TryFromArgs,
            M: Handler<$name>,
            )*
        {
            fn register(ctx: &Context) -> Result<(), RedisError> {
                $( command_register::<M, $name>(ctx)?; )*

                Ok(())
            }
        }

        // skip first element and call macro again
        peel! { $($name,)+ }
    )
}

macro_rules! peel {
    ($name:ident, $($other:ident,)*) => (tuple! { $($other,)* })
}

tuple![C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11, C12, C13, C14, C15, C16,];

#[macro_export]
macro_rules! command_no_args {
    ($ty:ident) => {
        pub struct $ty;

        impl TryFrom<Vec<RedisString>> for $ty {
            type Error = $crate::RedisError;

            fn try_from(value: Vec<RedisString>) -> Result<Self, Self::Error> {
                Ok($ty)
            }
        }
    };
}
