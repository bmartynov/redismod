mod config;
mod requests;
mod types;

use redis_module as rm;
use redismod::{module, Module, ModuleStores, Store};
use requests::TaskCreate;
use types::Task;

module![ExampleModule];

#[derive(Debug, thiserror::Error)]
pub enum ExampleError {}

struct ExampleModule {
    store_task: Store<Task>,
}

impl Module for ExampleModule {
    const VERSION: i32 = 1;
    const NAME: &'static str = "example";

    type Error = ExampleError;
    type Config = config::ExampleConfig;
    type Requests = (TaskCreate,);
    type DataTypes = (Task,);

    fn stop(&self, _ctx: &rm::Context) -> Result<(), Self::Error> {
        log::info!(target: "module", "stop");

        Ok(())
    }
    fn start(&mut self, _ctx: &rm::Context) -> Result<(), Self::Error> {
        log::info!(target: "module", "start");

        Ok(())
    }
    fn create(_ctx: &rm::Context, _config: Self::Config, stores: ModuleStores<Self>) -> Result<Self, Self::Error> {
        log::info!(target: "module", "create");

        let (store_task,) = stores;

        Ok(Self { store_task })
    }
}
