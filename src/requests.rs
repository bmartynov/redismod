use std::{ffi, ffi::CString};

use redis_module as rm;

use crate::{InstanceMngr, Module};

pub trait Command: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {
    fn validate(&self) -> Result<(), rm::RedisError> {
        Ok(())
    }
}

impl<R> Command for R where R: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {}

pub struct CommandKeys {
    pub first: u8,
    pub last: u8,
    pub step: u8,
}

impl CommandKeys {
    fn as_redis_keys(&self) -> (ffi::c_int, ffi::c_int, ffi::c_int) {
        (
            self.first as ffi::c_int,
            self.last as ffi::c_int,
            self.step as ffi::c_int,
        )
    }
}

pub trait RequestHandler<R: Command> {
    const NAME: &'static str;
    const FLAGS: &'static str;
    const KEYS: CommandKeys;

    type Result: Into<rm::RedisResult>;

    fn handle(&self, ctx: &rm::Context, req: R) -> Self::Result;
}

pub trait Requests<M: Module> {
    fn register<G: InstanceMngr<M>>(_ctx: &rm::Context) -> Result<(), ()>;
}

impl<M: Module> Requests<M> for () {
    fn register<G: InstanceMngr<M>>(_ctx: &rm::Context) -> Result<(), ()> {
        Ok(())
    }
}

fn command_register<M, C, G>(ctx: &rm::Context) -> Result<(), ()>
where
    M: 'static,
    M: Module,
    C: Command,
    G: InstanceMngr<M>,
    M: RequestHandler<C>,
{
    let module_name = <M as Module>::NAME;
    let command_name = <M as RequestHandler<C>>::NAME;
    let command_name_full = format!("{}.{}", module_name, command_name);

    let keys = <M as RequestHandler<C>>::KEYS;
    let name = CString::new(command_name_full).unwrap();
    let flags = CString::new(<M as RequestHandler<C>>::FLAGS).unwrap();

    let (key_first, key_last, key_step) = keys.as_redis_keys();

    let status = rm::Status::from(unsafe {
        rm::raw::RedisModule_CreateCommand.unwrap()(
            ctx.ctx,
            name.as_ptr(),
            Some(do_command::<M, C, G>),
            flags.as_ptr(),
            key_first,
            key_last,
            key_step,
        )
    });

    if rm::Status::Ok == status {
        Ok(())
    } else {
        Err(())
    }
}

extern "C" fn do_command<M, C, G>(
    ctx: *mut rm::RedisModuleCtx,
    argv: *mut *mut rm::RedisModuleString,
    argc: ffi::c_int,
) -> ffi::c_int
where
    M: 'static,
    M: Module,
    C: Command,
    G: InstanceMngr<M>,
    M: RequestHandler<C>,
{
    let ctx = &rm::Context::new(ctx);
    let args = rm::decode_args(ctx.ctx, argv, argc);

    let instance = match G::get() {
        Some(instance) => instance,
        None => return ctx.reply_error_string("instance missed") as ffi::c_int,
    };

    let req = match C::try_from(args) {
        Ok(req) => req,
        Err(err) => return ctx.reply(Err(err)) as ffi::c_int,
    };

    let result = instance.handle(ctx, req);

    ctx.reply(result.into()) as ffi::c_int
}

// adapted from core/src/fmt/mod.rs tuple
macro_rules! tuple {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<M: 'static, $($name, )*> Requests<M> for ($($name, )*)
        where
            M: Module,
            $( $name: Command, )*
            $( M: RequestHandler<$name>, )*
        {
            fn register<G: InstanceMngr<M>>(ctx: &rm::Context) -> Result<(), ()> {
                $( command_register::<M, $name, G>(ctx)?; )*

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
