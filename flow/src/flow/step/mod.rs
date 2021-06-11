use chrono::Utc;

use chord::error::Error;
use chord::step::{StepRunner, StepValue};
use res::StepAssessStruct;

use crate::flow::step::arg::RunArgStruct;
use crate::model::app::Context;
use async_std::future::timeout;
use chord::step::StepState;
use futures::FutureExt;
use log::trace;
use std::panic::AssertUnwindSafe;

pub mod arg;
pub mod res;

pub async fn run(
    _: &dyn Context,
    arg: &RunArgStruct<'_, '_, '_, '_>,
    runner: &dyn StepRunner,
) -> StepAssessStruct {
    trace!("step start {}", arg.id());
    let start = Utc::now();
    let future = AssertUnwindSafe(runner.run(arg)).catch_unwind();
    let timeout_value = timeout(arg.timeout(), future).await;
    let value = match timeout_value {
        Ok(cu) => match cu {
            Ok(sv) => sv,
            Err(_) => {
                return StepAssessStruct::new(
                    arg.id().clone(),
                    start,
                    Utc::now(),
                    StepState::Err(Error::new("002", "unwind")),
                );
            }
        },
        Err(_) => {
            return StepAssessStruct::new(
                arg.id().clone(),
                start,
                Utc::now(),
                StepState::Err(Error::new("001", "timeout")),
            );
        }
    };

    return match value {
        StepValue::Ok(json) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Ok(json))
        }
        StepValue::Err(e) => {
            StepAssessStruct::new(arg.id().clone(), start, Utc::now(), StepState::Err(e))
        }
    };
}
