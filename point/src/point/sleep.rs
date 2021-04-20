use chord_common::point::{PointArg, PointValue};
use chord_common::value::Json;
use async_std::task::sleep;
use std::time::Duration;

pub async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let seconds = pt_arg.config()["seconds"].as_i64().unwrap_or(0) as u64;
    sleep(Duration::from_secs(seconds)).await;
    return Ok(Json::Null);
}