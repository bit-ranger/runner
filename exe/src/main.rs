use std::{env, fs};
use std::path::Path;
use std::time::SystemTime;

use flow::{AppContextStruct};
use point::PointRunnerDefault;
use port::load::file;
use common::error::Error;
use common::task::TaskResult;

mod logger;
use log::info;
use futures::future::join_all;


#[async_std::main]
async fn main() -> Result<(),usize> {
    let args: Vec<_> = env::args().collect();
    let mut opts = getopts::Options::new();
    opts.reqopt("j", "job", "job path", "job");
    opts.reqopt("l", "log", "log path", "log");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(_) => {
            println!("{}", opts.short_usage("runner"));
            return Err(1);
        }
    };

    let log_path = matches.opt_str("l").unwrap();
    logger::init(log::Level::Info,
                 log_path,
                 1,
                 2000000).unwrap();

    let duration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let execution_id = duration.as_millis().to_string();

    let job_path = matches.opt_str("j").unwrap();
    let job_path = Path::new(&job_path);
    if !job_path.is_dir(){
        panic!("job path is not a dir {}", job_path.to_str().unwrap());
    }

    let app_context = AppContextStruct::new(Box::new(PointRunnerDefault::new()));
    // async_task::block_on(async {
        run_job(job_path, execution_id.as_str(), &app_context).await;
    // });

    return Ok(());
}


pub async fn run_job<P: AsRef<Path>>(job_path: P, execution_id: &str, app_context: &AppContextStruct<'_>) -> Vec<TaskResult>{
    let job_path_str = job_path.as_ref().to_str().unwrap();

    info!("running job {}", job_path_str);
    let children = fs::read_dir(job_path.as_ref()).unwrap();

    let mut futures = Vec::new();
    for task_dir in children{
        if task_dir.is_err(){
            continue;
        }
        let task_dir = task_dir.unwrap();
        if !task_dir.path().is_dir(){
            continue;
        }

        futures.push(
            run_task(job_path.as_ref().join(task_dir.path()), execution_id, app_context)
        );
    }

    let task_result_vec = join_all(futures).await;
    let task_status = task_result_vec.iter()
        .map(|r| r.as_ref().map_or_else(|e| Err(e.get_code()), |_| Ok(true)))
        .collect::<Vec<Result<bool, &str>>>();
    info!("finish job {}, {:?}", job_path_str, task_status);
    return task_result_vec;
}

async fn run_task<P: AsRef<Path>>(task_path: P, execution_id: &str, app_context: &AppContextStruct<'_>) -> TaskResult {
    info!("running task {}", task_path.as_ref().to_str().unwrap());
    let task_path = Path::new(task_path.as_ref());
    let data_path = task_path.join("data.csv");
    let flow_path = task_path.join("flow.yml");

    let flow = match file::load_flow(
        &flow_path
    ) {
        Err(e) => {
            return Err(Error::new("001", format!("load config failure {}", e).as_str()))
        }
        Ok(value) => {
            value
        }
    };


    let data = match file::load_data(
        &data_path
    ) {
        Err(e) => {
            return Err(Error::new("000", format!("load data failure {}", e).as_str()));
        }
        Ok(vec) => {
            vec
        }
    };


    let task_result = flow::run(app_context, flow, data, task_path.file_name().unwrap().to_str().unwrap()).await;


    let report_path = task_path.join(format!("report_{}.csv", execution_id));
    let _ = port::report::csv::report(&task_result, &report_path).await;
    info!("finish task {} >>> {}", task_path.to_str().unwrap(), task_result.is_ok());
    return task_result;
}