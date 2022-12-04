use redis_module::{RedisError, RedisString};

pub struct ExampleConfig;

impl TryFrom<Vec<RedisString>> for ExampleConfig {
    type Error = RedisError;

    fn try_from(_value: Vec<RedisString>) -> Result<Self, Self::Error> {
        Ok(Self)
    }
}
