use chord_common::value::{Json, Number, from_str};
use redis::{Value as RedisValue};
use chord_common::point::{PointArg, PointValue, PointRunner, async_trait};
use chord_common::{err};
use chord_common::error::Error;

struct Redis {}

#[async_trait]
impl PointRunner for Redis {

    async fn run(&self, arg: &dyn PointArg) -> PointValue {
        run(arg).await
    }
}

pub async fn create(_: &Json) -> Result<Box<dyn PointRunner>, Error>{
    Ok(Box::new(Redis {}))
}



async fn run(pt_arg: &dyn PointArg) -> PointValue {
    let url = pt_arg.config_rendered(vec!["url"]).ok_or(err!("010", "missing url"))?;
    let cmd = pt_arg.config_rendered(vec!["cmd"]).ok_or(err!("012", "missing cmd"))?;

    let client = redis::Client::open(url)?;
    let mut con = client.get_async_connection().await?;

    let mut command = redis::cmd(cmd.as_str());
    let args_opt = pt_arg.config()["args"].as_array();
    if args_opt.is_some(){
        for arg in args_opt.unwrap() {
            command.arg(arg.to_string().as_str());
        }
    }

    let redis_value:RedisValue = command.query_async(&mut con).await?;
    let result = match &redis_value {
        RedisValue::Nil => {
            Json::Null
        },
        RedisValue::Int(i) => {
            Json::Number(Number::from(i.clone()))
        },
        RedisValue::Data(data) => {
            let data = String::from_utf8_lossy(data);
            let dv = from_str(data.as_ref());
            match dv {
                Ok(v) => v,
                Err(_) => Json::String(data.to_string())
            }
        },
        RedisValue::Status(status) => {
            Json::String(status.clone())
        },
        RedisValue::Okay => {
            Json::String("OK".to_string())
        },
        _ => {
            Json::Array(vec![])
        }
    };
    return Ok(result);
}