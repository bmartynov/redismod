use std::fmt::Debug;
use std::{str, time};

use redis_module as rm;
use redis_module::{NextArg, RedisError};

type Result<T> = std::result::Result<T, RedisError>;

pub trait FromArgs: TryFrom<Vec<rm::RedisString>, Error = RedisError> {}

impl<R> FromArgs for R where R: TryFrom<Vec<rm::RedisString>, Error = RedisError> {}

impl<T> NextArgExt for T where T: NextArg {}

pub trait NextArgExt: NextArg {
    fn next_parse<T>(&mut self) -> Result<T>
    where
        T: str::FromStr,
        T::Err: Debug,
    {
        let id: T = self.next_str()?.parse().map_err(|parse_error| {
            RedisError::String(format!(
                "cannot parse as type {}: {:?}",
                std::any::type_name::<T>(),
                parse_error
            ))
        })?;

        Ok(id)
    }

    fn next_millis(&mut self) -> Result<time::Duration> {
        Ok(time::Duration::from_millis(self.next_u64()?))
    }

    fn next_vec(&mut self) -> Result<Vec<u8>> {
        let s = self.next_arg()?;

        Ok(s.as_slice().to_vec())
    }

    fn next_usize(&mut self) -> Result<usize> {
        self.next_u64().map(|v| v as usize)
    }

    fn next_unsigned<U: TryFrom<u64>>(&mut self) -> Result<U> {
        self.next_u64()?
            .try_into()
            .map_err(|_| RedisError::Str(concat!(stringify!(T), "from i64 failed")))
    }

    fn next_signed<U: TryFrom<i64>>(&mut self) -> Result<U> {
        self.next_i64()?
            .try_into()
            .map_err(|_| RedisError::Str(concat!(stringify!(T), "from i64 failed")))
    }
}
