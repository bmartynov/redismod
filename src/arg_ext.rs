use std::{str, time};
use std::fmt::Debug;

use redis_module as rm;

pub trait FromArgs: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {}

impl<R> FromArgs for R where R: TryFrom<Vec<rm::RedisString>, Error = rm::RedisError> {}

impl<T> NextArgExt for T where T: rm::NextArg {}

pub trait NextArgExt: rm::NextArg {
    fn next_parse<T>(&mut self) -> Result<T, rm::RedisError>
    where
        T: str::FromStr,
        T::Err: Debug,
    {
        let id: T = self.next_string()?.parse().map_err(|parse_error| {
            rm::RedisError::String(format!(
                "cannot parse as type {}: {:?}",
                std::any::type_name::<T>(),
                parse_error
            ))
        })?;

        Ok(id)
    }

    fn next_millis(&mut self) -> Result<time::Duration, rm::RedisError> {
        Ok(time::Duration::from_millis(self.next_u64()?))
    }

    fn next_vec(&mut self) -> Result<Vec<u8>, rm::RedisError> {
        let s = self.next_arg()?;

        Ok(s.as_slice().to_vec())
    }
}
