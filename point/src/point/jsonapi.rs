use std::str::FromStr;

use surf::{Body, RequestBuilder, Response, Url};
use surf::http::headers::{HeaderName, HeaderValue};
use surf::http::Method;

use chord_common::point::{PointArg, PointValue};
use chord_common::{rerr, err};

use chord_common::value::{Json, Map, Number};
use chord_common::error::Error;


pub async fn run(context: &dyn PointArg) -> PointValue{
    return run0(context).await.map_err(|e| e.0);
}

async fn run0(context: &dyn PointArg) -> std::result::Result<Json, Rae>{

    let url = context.config_rendered(vec!["url"]).ok_or(err!("010", "missing url"))?;
    let url = Url::from_str(url.as_str()).or(rerr!("011", format!("invalid url: {}", url)))?;

    let method = context.config_rendered(vec!["method"]).ok_or(err!("020", "missing method"))?;
    let method = Method::from_str(method.as_str()).or(rerr!("021", "invalid method"))?;

    let mut rb = RequestBuilder::new(method, url);
    rb = rb.header(HeaderName::from_str("Content-Type").unwrap(), HeaderValue::from_str("application/json")?);

    if let Some(header) = context.config()["header"].as_object() {
        for (k, v) in header.iter() {
            let hn = HeaderName::from_string(context.render(k)?)
                .or(rerr!("030", "invalid header name"))?;
            let hvt = context.render(v.as_str().ok_or(err!("031", "invalid header value"))?)?;
            let hv = HeaderValue::from_str(hvt.as_str())
                .or(rerr!("031", "invalid header value"))?;
            rb = rb.header(hn, hv);
        }
    }

    if let Some(body) = context.config_rendered(vec!["body"]){
        rb = rb.body(Body::from_string(body));
    }

    let mut res: Response = rb.send().await?;
    let mut res_data = Map::new();
    res_data.insert(String::from("status"), Json::Number(Number::from_str(res.status().to_string().as_str()).unwrap()));

    let mut header_data = Map::new();
    for header_name in res.header_names() {
        header_data.insert(header_name.to_string(), Json::String(res.header(header_name).unwrap().to_string()));
    }

    res_data.insert(String::from("header"), Json::Object(header_data));

    let body: Json = res.body_json().await?;
    res_data.insert(String::from("body"), body);
    return Ok(Json::Object(res_data))

}

struct Rae(chord_common::error::Error);


impl From<surf::Error> for Rae {
    fn from(err: surf::Error) -> Rae {
        Rae(err!("http", format!("{}", err.status())))
    }
}

impl From<chord_common::error::Error> for Rae {
    fn from(err: Error) -> Self {
        Rae(err)
    }
}




