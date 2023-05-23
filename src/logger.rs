use redis_module as rm;
use log::{self, Level, Metadata, Record};

struct RedisLogger;

impl log::Log for RedisLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        let level = match record.level() {
            Level::Error => rm::LogLevel::Warning,
            Level::Warn => rm::LogLevel::Warning,
            Level::Info => rm::LogLevel::Notice,
            Level::Debug => rm::LogLevel::Debug,
            Level::Trace => rm::LogLevel::Verbose,
        };

        let message = format!("{}: {}", record.target(), record.args());

        rm::logging::log(level, &message);
    }

    fn flush(&self) {}
}

pub fn setup(/*place args here*/) -> Result<(), ()> {
    log::set_max_level(log::LevelFilter::Trace);

    let logger = Box::new(RedisLogger);

    if let Err(_err) = log::set_boxed_logger(logger) {
        return Err(());
    }

    Ok(())
}
