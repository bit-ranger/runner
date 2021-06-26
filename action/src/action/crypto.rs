use chord::step::{async_trait, Action, ActionFactory, ActionValue, CreateArg, RunArg};
use chord::value::Value;
use chord::Error;
use chord::{err, rerr};

pub struct CryptoFactory {}

impl CryptoFactory {
    pub async fn new(_: Option<Value>) -> Result<CryptoFactory, Error> {
        Ok(CryptoFactory {})
    }
}

#[async_trait]
impl ActionFactory for CryptoFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Crypto {}))
    }
}

struct Crypto {}

#[async_trait]
impl Action for Crypto {
    async fn run(&self, arg: &dyn RunArg) -> ActionValue {
        run(arg).await
    }
}

async fn run(arg: &dyn RunArg) -> ActionValue {
    let by = arg.config()["by"]
        .as_str()
        .ok_or(err!("010", "missing by"))?;

    let from = arg.config()["from"]
        .as_str()
        .map(|s| arg.render_str(s))
        .ok_or(err!("010", "missing from"))??;

    return match by {
        "md5" => {
            let digest = md5::compute(from);
            let digest = format!("{:x}", digest);
            return Ok(Value::String(digest));
        }
        _ => {
            rerr!("crypto", format!("unsupported {}", by))
        }
    };
}
