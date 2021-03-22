use std::future::Future;
use std::pin::Pin;


use crate::error::Error;
use crate::value::Json;
use chrono::{DateTime, Utc};

pub type PointValue = std::result::Result<Json, Error>;

pub trait PointArg {

    fn config_rendered(&self, path: Vec<&str>) -> Option<String>;

    fn config(&self) -> &Json;

    fn render(&self, text: &str) -> Result<String,Error>;
}

pub trait PointRunner{

    fn run<'a>(&self, pt_type: &'a str, pt_arg: &'a dyn PointArg) -> Pin<Box<dyn Future<Output=PointValue>+ 'a>>;
}

#[derive(Debug, Clone)]
pub enum PointState {
    Ok(Json),
    Fail(Json),
    Err(Error)
}

pub trait PointAssess {

    fn id(&self) -> &str;

    fn start(&self) -> DateTime<Utc>;

    fn end(&self) -> DateTime<Utc>;

    fn state(&self) -> &PointState;
}
