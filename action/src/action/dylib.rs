use std::sync::Arc;

use dynamic_reload::{DynamicReload, Lib, PlatformName, Search, Symbol};

use chord::err;
use chord::step::{async_trait, Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::value::{to_string, Value};
use chord::Error;

pub struct DylibFactory {}

impl DylibFactory {
    pub async fn new(_: Option<Value>) -> Result<DylibFactory, Error> {
        Ok(DylibFactory {})
    }
}

#[async_trait]
impl ActionFactory for DylibFactory {
    async fn create(&self, arg: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        let dir = arg.config()["dir"]
            .as_str()
            .ok_or(err!("010", "missing dir"))?;

        let mut reload_handler = DynamicReload::new(Some(vec![dir]), Some(dir), Search::Default);

        let lib = reload_handler.add_library("fdylib", PlatformName::Yes)?;

        let action_create: Symbol<fn(&str, &str) -> Result<(), Error>> =
            unsafe { lib.lib.get(b"init")? };

        let config_str = to_string(arg.config())?;
        let config_str = arg.render_str(config_str.as_str())?;
        action_create(arg.id().to_string().as_str(), config_str.as_str())?;

        Ok(Box::new(Dylib { lib }))
    }
}

struct Dylib {
    lib: Arc<Lib>,
}

#[async_trait]
impl Action for Dylib {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let action_run: Symbol<fn(&str, &str) -> ActionValue> =
            unsafe { self.lib.lib.get(b"run")? };

        let config_str = to_string(arg.config())?;
        let config_str = arg.render_str(config_str.as_str())?;
        action_run(arg.id().to_string().as_str(), config_str.as_str())
    }
}
