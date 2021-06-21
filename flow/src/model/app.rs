use std::borrow::Borrow;

use handlebars::Handlebars;

use crate::model::helper::register;
use chord::step::ActionFactory;

pub trait Context: Sync + Send {
    fn get_handlebars(&self) -> &Handlebars;

    fn get_action_factory(&self) -> &dyn ActionFactory;
}

pub struct FlowContextStruct<'reg> {
    handlebars: Handlebars<'reg>,
    action_factory: Box<dyn ActionFactory>,
}

impl<'reg> FlowContextStruct<'reg> {
    pub fn new(action_factory: Box<dyn ActionFactory>) -> FlowContextStruct<'reg> {
        let mut handlebars = Handlebars::new();
        register(&mut handlebars);
        FlowContextStruct {
            handlebars,
            action_factory,
        }
    }
}

impl<'reg> Context for FlowContextStruct<'reg> {
    fn get_handlebars(self: &FlowContextStruct<'reg>) -> &Handlebars<'reg> {
        self.handlebars.borrow()
    }

    fn get_action_factory(self: &FlowContextStruct<'reg>) -> &dyn ActionFactory {
        self.action_factory.as_ref()
    }
}

pub type RenderContext = handlebars::Context;
