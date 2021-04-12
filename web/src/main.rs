mod controller;
use tide::{Request, Response};
use chord_common::value::Json;
use chord_common::error::Error;
use tide::prelude::*;
use tide::http::StatusCode;
use validator::{ValidationErrors, ValidationErrorsKind};
use std::path::Path;
use chord_cmd::logger;

#[derive(Serialize, Deserialize)]
struct ErrorBody{
    code: String,
    message: String
}

fn common_error_json(e: &Error) -> Json {
    json!(ErrorBody{
                code: e.code().into(),
                message: e.message().into()
            })
}

fn validator_error_json_nested(e: &ValidationErrors) -> Vec<String> {
    return e.errors()
        .iter()
        .map(|(k, e)|
            match e {
                ValidationErrorsKind::Field(ev) =>
                    ev.iter()
                        .map(|e| format!("[{}] {}", k, e.to_string()))
                        .collect(),
                ValidationErrorsKind::Struct(f) =>
                    validator_error_json_nested(f.as_ref()),
                ValidationErrorsKind::List(m) =>
                    m.iter()
                        .map(|(_i, e)|
                            validator_error_json_nested(e.as_ref()))
                        .fold(Vec::new(), |mut l,e| {
                            l.extend(e);
                            return l;
                        })
            }
        )
        .fold(Vec::new(), |mut l,e| {
            l.extend(e);
            return l;
        })
}

fn validator_error_json(e: &ValidationErrors) ->Json{
    json!(ErrorBody{
                code: "400".into(),
                message: validator_error_json_nested(e).into_iter().last().unwrap()
            })
}




#[macro_export]
macro_rules! json_handler {
    ($func:path) => {{
        |mut req: Request<()>| async move {
            let rb =  req.body_json().await?;
            if let Err(e) = validator::Validate::validate(&rb){
                return Ok(Response::builder(StatusCode::InternalServerError)
                    .body(validator_error_json(&e)))
            };
            let rst = $func(rb).await;
            match rst{
                Ok(r) => Ok(Response::builder(StatusCode::Ok)
                    .body(json!(r))),
                Err(e) => Ok(Response::builder(StatusCode::InternalServerError)
                    .body(common_error_json(&e)))
            }
        }
    }}
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let log_file_path = Path::new("/var/logs/log.log");
    let _log_handler = logger::init(vec![], &log_file_path).await?;

    let mut app = tide::new();

    app.at("/job/exec").post(
        json_handler!(controller::job::exec)
    );

    app.listen("127.0.0.1:8080").await?;

    Ok(())
}


