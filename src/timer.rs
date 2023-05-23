use std::time::Duration;

use redis_module::{Context, RedisModuleTimerID};

use crate::Module;

pub trait TimerHandler<R> {
    fn handle(&self, ctx: &Context, req: R);
}

pub fn create_timer<M, R>(ctx: &Context, period: Duration, data: R) -> RedisModuleTimerID
where
    M: Module,
    M: TimerHandler<R>,
    R: 'static,
    M: 'static,
{
    ctx.create_timer(period, callback::<M, R>, data)
}

fn callback<M, R>(ctx: &Context, data: R)
where
    M: Module,
    M: TimerHandler<R>,
    M: 'static,
{
    let instance = match M::get() {
        Some(instance) => instance,
        None => {
            log::error!("instance missed");
            return;
        }
    };

    instance.handle(ctx, data);
}
