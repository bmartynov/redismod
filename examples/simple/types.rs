use std::time::Duration;

use redis_module as rm;
use redismod::{IOLoader, IOSaver, Loader, Saver, Type};

#[derive(Debug, Clone)]
pub enum TaskState {
    Failed,
    Pending,
    Started,
    Finished,
}

impl From<TaskState> for u64 {
    fn from(value: TaskState) -> Self {
        match value {
            TaskState::Failed => 0,
            TaskState::Pending => 1,
            TaskState::Started => 2,
            TaskState::Finished => 3,
        }
    }
}

impl TryFrom<u64> for TaskState {
    type Error = rm::RedisError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(match value {
            0 => Self::Failed,
            1 => Self::Pending,
            2 => Self::Started,
            3 => Self::Finished,
            _ => return Err(rm::RedisError::Str("invalid task state")),
        })
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub id: xid::Id,
    pub r#type: String,
    pub retries: u64,
    pub timeout: Duration,
    pub worker: String,
    pub payload: Vec<u8>,
    pub state: TaskState,
}

impl Type for Task {
    type IDType = xid::Id;

    const NAME: &'static str = "task";
    const PREFIX: &'static str = "t";

    const REDIS_NAME: &'static str = "taskver10";
    const REDIS_VERSION: i32 = 1;

    fn free(_value: Box<Self>) {}

    fn mem_usage(_value: &Self) -> usize {
        0
    }

    fn rdb_save(saver: &IOSaver, value: &Self) {
        saver.buffer(value.id.as_bytes());
        saver.buffer(value.r#type.as_bytes());
        saver.unsigned(value.retries);
        saver.unsigned(value.timeout.as_millis().try_into().unwrap_or(5_000));
        saver.buffer(value.worker.as_bytes());
        saver.buffer(value.payload.as_slice());
        saver.unsigned(value.state.clone().into());
    }

    fn rdb_load(loader: &IOLoader, _encver: usize) -> Result<Self, rm::error::Error> {
        let id = {
            let bytes: [u8; 12] = loader
                .buffer()?
                .as_ref()
                .try_into()
                .map_err(|_| rm::error::Error::generic("id len missmatch"))?;

            xid::Id(bytes)
        };

        let r#type = loader.buffer()?.to_string().map_err(rm::error::Error::FromUtf8)?;

        let retries = loader.unsigned()?;

        let timeout = Duration::from_millis(loader.unsigned()?);
        let worker = loader.buffer()?.to_string().map_err(rm::error::Error::FromUtf8)?;

        let payload = loader.buffer()?.as_ref().to_vec();
        let state = loader.unsigned()?.try_into()?;

        Ok(Self {
            id,
            r#type,
            retries,
            timeout,
            worker,
            payload,
            state,
        })
    }
}
