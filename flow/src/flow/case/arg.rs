use std::fmt::{Display, Formatter};

use async_std::sync::Arc;

use chord::action::Action;
use chord::case::CaseId;
use chord::collection::TailDropVec;
use chord::flow::Flow;
use chord::task::TaskId;
use chord::value::Value;
use chord::value::{to_value, Map};

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::Context;
use crate::model::app::RenderContext;
use chord::Error;

#[derive(Clone)]
pub struct CaseIdStruct {
    task_id: Arc<dyn TaskId>,
    exec_id: Arc<String>,
    case: String,
}

impl CaseIdStruct {
    pub fn new(task_id: Arc<dyn TaskId>, case_id: String, exec_id: Arc<String>) -> CaseIdStruct {
        CaseIdStruct {
            task_id,
            exec_id,
            case: case_id,
        }
    }
}

impl CaseId for CaseIdStruct {
    fn case(&self) -> &str {
        self.case.as_str()
    }

    fn exec_id(&self) -> &str {
        self.exec_id.as_str()
    }

    fn task_id(&self) -> &dyn TaskId {
        self.task_id.as_ref()
    }
}

impl Display for CaseIdStruct {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_str(format!("{}-{}-{}", self.task_id, self.exec_id, self.case).as_str())
    }
}

pub struct CaseArgStruct {
    flow: Arc<Flow>,
    step_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
    data: Value,
    pre_ctx: Option<Arc<Value>>,
    id: Arc<CaseIdStruct>,
}

impl CaseArgStruct {
    pub fn new(
        flow: Arc<Flow>,
        step_vec: Arc<TailDropVec<(String, Box<dyn Action>)>>,
        data: Value,
        pre_ctx: Option<Arc<Value>>,
        task_id: Arc<dyn TaskId>,
        case_id: String,
        case_exec_id: Arc<String>,
    ) -> CaseArgStruct {
        let id = Arc::new(CaseIdStruct::new(task_id, case_id, case_exec_id));

        let context = CaseArgStruct {
            flow,
            step_vec,
            data,
            pre_ctx,
            id,
        };

        return context;
    }

    pub fn create_render_context(self: &CaseArgStruct) -> RenderContext {
        let mut render_data: Map = Map::new();
        let config_def = self.flow.def();
        if let Some(def) = config_def {
            render_data.insert(String::from("def"), to_value(def).unwrap());
        }
        render_data.insert(String::from("case"), self.data.clone());
        render_data.insert(String::from("step"), Value::Object(Map::new()));
        render_data.insert(String::from("curr"), Value::Null);
        if let Some(pre_ctx) = self.pre_ctx.as_ref() {
            render_data.insert(String::from("pre"), pre_ctx.as_ref().clone());
        }

        return RenderContext::wraps(render_data).unwrap();
    }

    pub fn step_arg_create<'app, 'h, 'reg, 'r, 'p>(
        self: &CaseArgStruct,
        step_id: &str,
        flow_ctx: &'app dyn Context,
        render_ctx: &'r RenderContext,
    ) -> Result<RunArgStruct<'_, 'h, 'reg, 'r, 'p>, Error>
    where
        'app: 'h,
        'app: 'reg,
        'app: 'p,
    {
        RunArgStruct::new(
            self.flow.as_ref(),
            flow_ctx.get_handlebars(),
            render_ctx,
            flow_ctx.get_flow_parse(),
            self.id.clone(),
            step_id.to_owned(),
        )
    }

    pub fn step_vec(self: &CaseArgStruct) -> Arc<TailDropVec<(String, Box<dyn Action>)>> {
        self.step_vec.clone()
    }

    pub fn id(&self) -> Arc<CaseIdStruct> {
        self.id.clone()
    }

    pub fn take_data(self) -> Value {
        self.data
    }
}
