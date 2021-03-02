use crate::model::PointContext;
use std::thread;
use async_std::sync::Arc;

pub async fn run_point(context: PointContext) -> Result<(),()>{
    let url = context.get_config()["url"].as_str().unwrap();
    println!("run_point {} on thread {:?}", url, thread::current().id());
    return Ok(());
}