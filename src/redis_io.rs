use redis_module as rm;

pub trait Loader {
    fn double(&self) -> Result<f64, rm::error::Error>;
    fn float(&self) -> Result<f32, rm::error::Error>;
    fn unsigned(&self) -> Result<u64, rm::error::Error>;
    fn signed(&self) -> Result<i64, rm::error::Error>;
    fn string(&self) -> Result<rm::RedisString, rm::error::Error>;
    fn buffer(&self) -> Result<rm::RedisBuffer, rm::error::Error>;
}

pub trait Saver {
    fn double(&self, val: f64);
    fn float(&self, val: f32);
    fn unsigned(&self, val: u64);
    fn signed(&self, val: i64);
    fn string<S: AsRef<str>>(&self, val: S);
    fn buffer<S: AsRef<[u8]>>(&self, val: S);
}

pub struct IOLoader {
    pub rdb: *mut redis_module::RedisModuleIO,
}

impl Loader for IOLoader {
    fn double(&self) -> Result<f64, rm::error::Error> {
        rm::load_double(self.rdb)
    }
    fn float(&self) -> Result<f32, rm::error::Error> {
        rm::load_float(self.rdb)
    }
    fn unsigned(&self) -> Result<u64, rm::error::Error> {
        rm::load_unsigned(self.rdb)
    }
    fn signed(&self) -> Result<i64, rm::error::Error> {
        rm::load_signed(self.rdb)
    }
    fn string(&self) -> Result<rm::RedisString, rm::error::Error> {
        rm::load_string(self.rdb)
    }
    fn buffer(&self) -> Result<rm::RedisBuffer, rm::error::Error> {
        rm::load_string_buffer(self.rdb)
    }
}

pub struct IOSaver {
    pub rdb: *mut rm::RedisModuleIO,
}

impl Saver for IOSaver {
    fn double(&self, val: f64) {
        rm::save_double(self.rdb, val)
    }
    fn float(&self, val: f32) {
        rm::save_float(self.rdb, val)
    }
    fn unsigned(&self, val: u64) {
        rm::save_unsigned(self.rdb, val)
    }
    fn signed(&self, val: i64) {
        rm::save_signed(self.rdb, val)
    }
    fn string<S: AsRef<str>>(&self, val: S) {
        rm::save_string(self.rdb, val.as_ref())
    }
    fn buffer<S: AsRef<[u8]>>(&self, val: S) {
        rm::save_slice(self.rdb, val.as_ref())
    }
}
