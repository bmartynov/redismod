use std::ffi::c_int;

use redis_module::Context;
// pub use redis_module::NotifyEvent;

use crate::Module;

pub type Subscriptions = &'static [u8];

pub trait Handler<M: Module> {
    // type Events: Events;

    const EVENT: ();
    const SUBS: Subscriptions;

    fn handle(&self, ctx: &Context, module: &M);
}

trait Events {
    const BITS: c_int;
}

// trait NamedEvent {
//     const EVENT: &'static str;
// }

trait EventHandler {}

impl<E: Events, H> EventHandler for (E, H) {}

pub trait EventHandlers<M: Module> {
    fn register(_ctx: &Context) -> Result<(), ()>;
}

trait RawEvent {
    const RAW: c_int;
}

pub struct EventGeneric;
pub struct EventString;
pub struct EventList;
pub struct EventSet;
pub struct EventHash;
pub struct EventZset;
pub struct EventExpired;
pub struct EventEvicted;
pub struct EventStream;
pub struct EventModule;
pub struct EventLoaded;
pub struct EventMissed;
pub struct EventAll;

macro_rules! impl_raw_event {
    ($(($type:ty, $raw:expr)),* $(,)*) => {
        $(
            impl RawEvent for $type {
                const RAW: c_int = $raw.bits();
            }
        )*
    };
}

impl_raw_event!(
    (EventGeneric, NotifyEvent::GENERIC),
    (EventString, NotifyEvent::STRING),
    (EventList, NotifyEvent::LIST),
    (EventSet, NotifyEvent::SET),
    (EventHash, NotifyEvent::HASH),
    (EventZset, NotifyEvent::ZSET),
    (EventExpired, NotifyEvent::EXPIRED),
    (EventEvicted, NotifyEvent::EVICTED),
    (EventStream, NotifyEvent::STREAM),
    (EventModule, NotifyEvent::MODULE),
    (EventLoaded, NotifyEvent::LOADED),
    (EventMissed, NotifyEvent::MISSED),
);

// adapted from core/src/fmt/cmds tuple
macro_rules! tuple_events {
    () => ();
    ( $($name:ident,)+ ) => (
        impl<$($name: RawEvent, )*> Events for ($($name, )*) {
            const BITS: c_int = $($name::RAW | )* 0;
        }

        // skip first element and call macro again
        peel_types! { $($name,)+ }
    )
}

macro_rules! peel_types {
    ($name:ident, $($other:ident,)*) => (tuple_events! { $($other,)* })
}

tuple_events![T1, T2, T3, T4,];
