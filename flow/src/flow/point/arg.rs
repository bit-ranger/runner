use std::borrow::Borrow;
use std::collections::BTreeMap;

use handlebars::{Handlebars};
use log::info;
use serde::Serialize;
use serde_json::to_value;

use common::error::Error;
use common::point::PointArg;
use common::value::Json;
use common::err;

use crate::flow::case::arg::RenderContext;

#[derive(Debug)]
pub struct PointArgStruct<'c, 'd, 'h, 'reg, 'r>
{
    flow: &'c Json,
    data: &'d BTreeMap<String,String>,
    point_id: String,
    handlebars: &'h Handlebars<'reg>,
    render_context: &'r RenderContext,
}


impl <'c, 'd, 'h, 'reg, 'r> PointArgStruct<'c, 'd, 'h, 'reg, 'r> {


    pub fn new(flow: &'c Json,
               data: &'d BTreeMap<String,String>,
               point_id: &str,
               handlebars: &'h Handlebars<'reg>,
               render_context: &'r RenderContext
    ) -> PointArgStruct<'c, 'd, 'h, 'reg, 'r>{

        let context = PointArgStruct {
            flow,
            data,
            point_id: String::from(point_id),
            handlebars,
            render_context
        };

        return context;
    }

    #[allow(dead_code)]
    pub fn get_id(self :&PointArgStruct<'c, 'd, 'h, 'reg, 'r>) -> &str{
        return self.point_id.as_str();
    }



    pub async fn get_meta_str(self : &PointArgStruct<'c, 'd, 'h, 'reg, 'r>, path: Vec<&str>) ->Option<String>
    {
        let config = self.flow["point"][&self.point_id].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => {
                match self.render_inner(s){
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            },
            None=> None
        }
    }

    fn render_inner(self: &PointArgStruct<'c, 'd, 'h, 'reg, 'r>, text: &str) -> Result<String, Error> {
        let render = self.handlebars.render_template_with_context(
            text, self.render_context);
        return match render {
            Ok(r) => Ok(r),
            Err(e) => err!("tpl", format!("{}", e).as_str())
        };
    }

    fn render_inner_with<T>(self: &PointArgStruct<'c, 'd, 'h, 'reg, 'r>, text: &str, with_data: (&str, &T)) -> Result<String, Error>
        where
            T: Serialize
    {
        let mut ctx = self.render_context.data().clone();

        if let Json::Object(data) = &mut ctx{
            let (n, d) = with_data;
            data.insert(String::from(n), to_value(d).unwrap());
        }

        // let handlebars = Handlebars::new();
        let render = self.handlebars.render_template(
            text, &ctx);
        return match render {
            Ok(r) => Ok(r),
            Err(e) => err!("tpl", format!("{}", e).as_str())
        };
    }

    pub async fn assert <T>(&self, condition: &str, with_data: &T) -> bool
        where
            T: Serialize
    {
        let template = format!(
            "{{{{#if {condition}}}}}true{{{{else}}}}false{{{{/if}}}}",
            condition = condition
        );

        let result = self.render_inner_with(&template, ("res", with_data));
        match result {
            Ok(result) => if result.eq("true") {true} else {false},
            Err(e) => {
                info!("assert failure: {} >>> {}", condition, e);
                false
            }
        }
    }

}


impl <'c, 'd, 'h, 'reg, 'r> PointArg for PointArgStruct<'c, 'd, 'h, 'reg, 'r> {


    fn get_config_rendered(self: &PointArgStruct<'c, 'd, 'h, 'reg, 'r>, path: Vec<&str>) -> Option<String>
    {
        let config = self.flow["point"][&self.point_id]["config"].borrow();

        let raw_config = path.iter()
            .fold(config,
                  |acc, k| acc[k].borrow()
            );

        match raw_config.as_str(){
            Some(s) => {
                match self.render_inner(s){
                    Ok(s) => Some(s),
                    Err(_) => None
                }
            },
            None=> None
        }

    }

    fn get_config(&self) -> &Json {
        let config = self.flow["point"][&self.point_id]["config"].borrow();
        return config;
    }

    fn render(&self, text: &str) -> Result<String, Error> {
        return self.render_inner(text);
    }
}