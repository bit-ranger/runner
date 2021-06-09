use std::str::FromStr;

use async_std::io::BufReader;
use async_std::prelude::*;
use async_std::process::{Child, Command, Stdio};
use log::trace;
use serde::{Deserialize, Serialize};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;
use surf::{Body, RequestBuilder, Response, Url};

use chord_common::error::Error;
use chord_common::step::{
    async_trait, CreateArg, RunArg, StepRunner, StepRunnerFactory, StepValue,
};
use chord_common::value::{from_str, to_string_pretty, Json};
use chord_common::{err, rerr};

pub struct Factory {
    registry_protocol: String,
    registry_address: String,
    port: usize,
    child: Child,
}

impl Factory {
    pub async fn new(config: Option<Json>) -> Result<Factory, Error> {
        if config.is_none() {
            return rerr!("010", "missing config");
        }
        let config = config.as_ref().unwrap();

        if config.is_null() {
            return rerr!("010", "missing config");
        }

        let jar_path = config["jar_path"]
            .as_str()
            .ok_or(err!("010", "missing jar_path"))?;

        let port = config["port"]
            .as_u64()
            .map(|p| p as usize)
            .ok_or(err!("010", "missing port"))?;

        let registry_protocol = config["registry"]["protocol"]
            .as_str()
            .unwrap_or("zookeeper")
            .to_owned();

        let registry_address = config["registry"]["address"]
            .as_str()
            .ok_or(err!("010", "missing registry_address"))?
            .to_owned();

        let mut child = Command::new("java")
            .arg("-jar")
            .arg(jar_path)
            .arg(format!("--server.port={}", port))
            .stdout(Stdio::piped())
            .spawn()?;

        let std_out = child.stdout.ok_or(err!("020", "missing stdout"))?;
        let std_out = BufReader::new(std_out);
        let mut lines = std_out.lines();
        loop {
            let line = lines.next().await;
            if line.is_none() {
                return rerr!("020", "failed to start dubbo-generic-gateway");
            }
            let line = line.unwrap()?;
            trace!("{}", line);
            if line == "----dubbo-generic-gateway-started----" {
                break;
            }
        }

        child.stdout = None;
        Ok(Factory {
            registry_protocol,
            registry_address,
            port,
            child,
        })
    }
}

#[async_trait]
impl StepRunnerFactory for Factory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn StepRunner>, Error> {
        Ok(Box::new(Runner {
            registry_protocol: self.registry_protocol.clone(),
            registry_address: self.registry_address.clone(),
            port: self.port,
        }))
    }
}

struct Runner {
    registry_protocol: String,
    registry_address: String,
    port: usize,
}

#[async_trait]
impl StepRunner for Runner {
    async fn run(&self, run_arg: &dyn RunArg) -> StepValue {
        let method_long = run_arg.config()["method"]
            .as_str()
            .ok_or(err!("010", "missing method"))?;
        let parts = method_long
            .split(&['#', '(', ',', ')'][..])
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        if parts.len() < 2 {
            return rerr!("010", "invalid method");
        }

        let args_raw = &run_arg.config()["args"];
        let args: Vec<Json> = match args_raw {
            Json::Array(aw_vec) => {
                let mut ar_vec: Vec<Json> = vec![];
                for aw in aw_vec {
                    let ar = render(run_arg, aw)?;
                    ar_vec.push(ar);
                }
                ar_vec
            }
            _ => render(run_arg, args_raw)?
                .as_array()
                .ok_or(err!("010", "missing args"))?
                .clone(),
        };

        let invoke = GenericInvoke {
            reference: Reference {
                application: Application {
                    name: "chord".to_owned(),
                },
                registry: Registry {
                    protocol: self.registry_protocol.clone(),
                    address: self.registry_address.clone(),
                },
                interface: parts[0].to_owned(),
            },

            method: parts[1].to_owned(),
            arg_types: parts[2..].iter().map(|p| p.to_owned().to_owned()).collect(),
            args,
        };

        let value = remote_invoke(self.port, invoke).await.map_err(|e| e.0)?;
        let value = &value;
        if value["success"].as_bool().unwrap_or(false) {
            return Ok(value["data"].clone());
        }

        return rerr!("dubbo", format!("{}::{}", value["code"], value["message"]));
    }
}

async fn remote_invoke(port: usize, remote_arg: GenericInvoke) -> Result<Json, Rae> {
    let rb = RequestBuilder::new(
        Method::Post,
        Url::from_str(format!("http://127.0.0.1:{}/api/dubbo/generic/invoke", port).as_str())
            .or(rerr!("021", "invalid url"))?,
    );
    let mut rb = rb.header(
        HeaderName::from_str("Content-Type").unwrap(),
        HeaderValue::from_str("application/json")?,
    );

    rb = rb.body(Body::from_json(&remote_arg)?);

    let mut res: Response = rb.send().await?;
    if !res.status().is_success() {
        return rerr!("dubbo", res.status().to_string())?;
    } else {
        let body: Json = res.body_json().await?;
        Ok(body)
    }
}

impl Drop for Factory {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

#[derive(Serialize, Deserialize)]
struct GenericInvoke {
    reference: Reference,
    method: String,
    arg_types: Vec<String>,
    args: Vec<Json>,
}

#[derive(Serialize, Deserialize)]
struct Reference {
    interface: String,
    application: Application,
    registry: Registry,
}

#[derive(Serialize, Deserialize)]
struct Application {
    name: String,
}

#[derive(Serialize, Deserialize)]
struct Registry {
    protocol: String,
    address: String,
}

struct Rae(chord_common::error::Error);

impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("dubbo", format!("{}", err.status())))
    }
}

impl From<serde_json::error::Error> for Rae {
    fn from(err: serde_json::error::Error) -> Rae {
        Rae(err!("dubbo", format!("{}:{}", err.line(), err.column())))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}

fn render(arg: &dyn RunArg, content: &Json) -> Result<Json, Error> {
    if content.is_null() {
        return Ok(Json::Null);
    }

    let body_str: String = if content.is_string() {
        content
            .as_str()
            .ok_or(err!("032", "invalid content"))?
            .to_owned()
    } else {
        to_string_pretty(content).or(rerr!("032", "invalid content"))?
    };
    let content_str = arg.render(body_str.as_str())?;
    return Ok(from_str(content_str.as_str())?);
}
