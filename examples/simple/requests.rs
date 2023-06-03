use std::time::Duration;

use redis_module as rm;
use redis_module::NextArg as _;
use redismod::{CommandKeys, NextArgExt, RequestHandler};

use crate::types::{Task, TaskState};
use crate::ExampleModule;

#[derive(Debug)]
pub struct TaskCreate {
    id: xid::Id,
    r#type: String,
    retries: u64,
    timeout: Duration,
    worker: String,
    payload: Vec<u8>,
}

impl TryFrom<Vec<rm::RedisString>> for TaskCreate {
    type Error = rm::RedisError;

    fn try_from(value: Vec<rm::RedisString>) -> Result<Self, Self::Error> {
        let mut args = value.into_iter().skip(1);

        let id = args.next_parse::<xid::Id>()?;

        Ok(Self {
            id,
            r#type: args.next_string()?,
            retries: args.next_u64()?,
            timeout: Duration::from_millis(args.next_u64()?),
            worker: args.next_string()?,
            payload: args.next_vec()?,
        })
    }
}

impl RequestHandler<TaskCreate> for ExampleModule {
    const NAME: &'static str = "task_create";
    const FLAGS: &'static str = "fast write";
    const KEYS: CommandKeys = CommandKeys {
        first: 1,
        last: 1,
        step: 1,
    };

    type Result = rm::RedisResult;

    fn handle(&self, ctx: &rm::Context, req: TaskCreate) -> Self::Result {
        if self.store_task.exists(ctx, &req.id)? {
            return Err(rm::RedisError::Str("task already exists"));
        }

        let value = Task {
            id: req.id,
            r#type: req.r#type,
            retries: req.retries,
            timeout: req.timeout,
            worker: req.worker,
            payload: req.payload,
            state: TaskState::Pending,
        };

        self.store_task.get_mut(ctx, &value.id).store(value)?;

        Ok(rm::RedisValue::SimpleStringStatic("OK"))
    }
}
