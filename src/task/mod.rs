use async_std::task::spawn;
use core::result::Result;
use core::result::Result::Ok;
use std::thread;
use crate::case::run_case;
use crate::model::TaskContext;
use futures::future::join_all;
use async_std::sync::Arc;

pub async fn run_task(task_context: TaskContext) -> Result<(),()>{
    let tc_context_vec =  Arc::new(task_context).create_case();
    join_all(tc_context_vec
        .into_iter()
        .map(|tc_context| run_case(tc_context))
        .map(|tc_future| spawn(tc_future))
    ).await;

    println!("run_task on thread {:?}", thread::current().id());
    return Ok(());
}
