use chrono::Utc;
use serde_json::to_value;

use common::error::Error;
use common::value::Json;
use result::CaseAssessStruct;

use crate::flow::case::arg::{CaseArgStruct, RenderContext};
use crate::flow::point;
use crate::model::app::AppContext;
use common::case::{CaseResult, CaseState};
use common::point::PointResult;

pub mod result;
pub mod arg;

pub async fn run(app_context: &dyn AppContext, case_arg: &mut CaseArgStruct<'_,'_>) -> CaseResult {
    let start = Utc::now();
    let mut render_context = case_arg.create_render_context();
    let mut point_result_vec = Vec::<(String, PointResult)>::new();
    for point_id in case_arg.get_point_id_vec().iter() {
        let point_context = case_arg.create_point(point_id, app_context, &render_context);
        if point_context.is_none(){
            return Err(Error::new("000", "invalid point"));
        }
        let point_context = point_context.unwrap();
        let result = point::run(app_context, &point_context).await;
        point_result_vec.push((point_id.clone(), result));
        let (point_id, point_result) = point_result_vec.last().unwrap();

        match point_result {
            Ok(r) => {
                let assert_true = point::assert(&point_context, r.result()).await;
                if assert_true {
                    register_dynamic(&mut render_context, point_id, r.result()).await;
                } else {
                    let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), CaseState::PointFailure);
                    return Ok(Box::new(result_struct));
                }
            },
            Err(e) =>  {
                let state = CaseState::PointError(e.clone());
                let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), state);
                return Ok(Box::new(result_struct));
            }
        }
    }

    let result_struct = CaseAssessStruct::new(point_result_vec, case_arg.id(), start, Utc::now(), CaseState::Ok);
    return Ok(Box::new(result_struct));
}


pub async fn register_dynamic(render_context: &mut RenderContext, point_id: &str, result: &Json) {
    if let Json::Object(data) = render_context.data_mut(){
        data["dyn"][point_id] = to_value(result).unwrap();
    }
}



