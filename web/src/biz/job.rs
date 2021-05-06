use std::path::Path;

use async_std::fs::{read_dir};
use async_std::sync::Arc;
use async_std::task::Builder;
use futures::future::join_all;
use log::debug;

use chord_common::error::Error;
use chord_common::flow::Flow;
use chord_common::task::TaskState;
use chord_flow::AppContext;
use chord_port::report::mongodb::{Reporter, Database, Collection, Document};
use crate::app::conf::Config;
use futures::StreamExt;

pub async fn run<P: AsRef<Path>>(job_path: P,
                                 job_name: String,
                                 exec_id: String,
                                 app_ctx: Arc<dyn AppContext>,
                                 db: Arc<Database>) -> Result<Vec<TaskState>, Error>{

    debug!("job start {}, {}", job_path.as_ref().to_str().unwrap(), job_name.as_str());
    let job_collection = Arc::new(db.collection::<Document>(job_name.as_str()));

    let mut job_dir = read_dir(job_path.as_ref()).await.unwrap();

    let mut futures = Vec::new();
    loop {
        let task_dir  = job_dir.next().await;
        if task_dir.is_none(){
            break;
        }
        let task_dir = task_dir.unwrap();
        if task_dir.is_err(){
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir().await{
            continue;
        }

        let builder = Builder::new()
            .name(task_dir.file_name().to_str().unwrap().into());

        let task_input_dir = job_path.as_ref().join(task_dir.path());
        let jh = builder.spawn(run_task(
            task_input_dir,
            exec_id.clone(),
            app_ctx.clone(),
            job_collection.clone()))
            .unwrap();
        futures.push(jh);
    }

    let task_state_vec = join_all(futures).await;
    debug!("job end {}, {}", job_path.as_ref().to_str().unwrap(), job_name.as_str());
    return Ok(task_state_vec);
}

async fn run_task<P: AsRef<Path>>(
    input_dir: P,
    exec_id: String,
    app_ctx: Arc<dyn AppContext>,
    collection: Arc<Collection>
) -> TaskState
{
    let rt = run_task0(input_dir, exec_id, app_ctx, collection).await;
    match rt {
        Ok(ts) => ts,
        Err(e) => TaskState::Err(e)
    }
}

async fn run_task0<P: AsRef<Path>>(task_path: P,
                                   exec_id: String,
                                   app_ctx: Arc<dyn AppContext>,
                                   collection: Arc<Collection>) -> Result<TaskState, Error> {
    let task_path = Path::new(task_path.as_ref());
    let task_id = task_path.file_name().unwrap().to_str().unwrap();
    chord_flow::TASK_ID.with(|tid| tid.replace(task_id.to_owned()));
    debug!("task start {}", task_path.to_str().unwrap());

    let flow_path = task_path.clone().join("flow.yml");

    let flow = chord_port::load::flow::yml::load(&flow_path)?;
    let flow = Flow::new(flow.clone())?;

    //read
    let data_path = task_path.clone().join("data.csv");
    let case_batch_size = Config::get_singleton().case_batch_size();
    let mut data_loader = chord_port::load::data::csv::Loader::new(data_path, case_batch_size).await?;

    //write
    let mut assess_reporter = Reporter::new(collection, task_id, exec_id).await?;

    //runner
    let mut runner = chord_flow::Runner::new(app_ctx, Arc::new(flow), String::from(task_id)).await?;

    let mut total_task_state = TaskState::Ok(vec![]);
    loop{
        let data = data_loader.load().await?;
        let data_len = data.len();

        let task_assess = runner.run(data).await;

        //write
        assess_reporter.write(task_assess.as_ref()).await?;

        match task_assess.state(){
            TaskState::Err(e) => {
                total_task_state = TaskState::Err(e.clone());
                break;
            },
            TaskState::Fail(_) => total_task_state = TaskState::Fail(vec![]),
            _ => ()
        }

        if data_len < case_batch_size {
            break;
        }
    }

    data_loader.close().await?;
    assess_reporter.close().await?;

    debug!("task end {}", task_path.to_str().unwrap());
    return Ok(total_task_state);
}
