use chord::step::{async_trait, Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::value::Value;
use chord::Error;

pub struct Factory {}

impl Factory {
    pub async fn new(_: Option<Value>) -> Result<Factory, Error> {
        Ok(Factory {})
    }
}

#[async_trait]
impl ActionFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Runner {}))
    }
}

struct Runner {}

#[async_trait]
impl Action for Runner {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        let config = arg.render_value(arg.config())?;
        return Ok(config);
    }
}