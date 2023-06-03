use redis_module::RedisError;
#[cfg(feature = "with_rkyv")]
use rkyv::ser::serializers::AllocSerializer;
use rkyv::{Archive, Deserialize, Infallible, Serialize};

use super::{IOLoader, IOSaver, Loader as _, Saver as _};

pub trait RDBLoadSave: Sized {
    fn rdb_save(saver: &IOSaver, value: &Self);
    fn rdb_load(loader: &IOLoader, encver: usize) -> Result<Self, RedisError>;
}

#[cfg(feature = "with_rkyv")]
impl<T> RDBLoadSave for T
where
    T: Archive,
    T: Serialize<AllocSerializer<256>>,
    T::Archived: Deserialize<T, Infallible>,
{
    fn rdb_save(saver: &IOSaver, value: &Self) {
        let bytes = match rkyv::to_bytes::<_, 256>(value) {
            Ok(bytes) => bytes,
            Err(err) => return log::error!("cannot marshal: {:?}", err),
        };

        saver.buffer(&bytes);
    }

    fn rdb_load(loader: &IOLoader, _encver: usize) -> Result<Self, RedisError> {
        let bytes = loader.buffer()?;
        let archived = unsafe { rkyv::archived_root::<Self>(bytes.as_ref()) };

        let mut deserializer = Infallible;

        let result = archived
            .deserialize(&mut deserializer)
            .map_err(|_| RedisError::Str("cannot unmarshal"))?;

        Ok(result)
    }
}
