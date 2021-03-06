use rlua::StdLib;

use chord::action::prelude::*;
use chord::value::{Map, Number};

pub struct LuaFactory {}

impl LuaFactory {
    pub async fn new(_: Option<Value>) -> Result<LuaFactory, Error> {
        Ok(LuaFactory {})
    }
}

#[async_trait]
impl Factory for LuaFactory {
    async fn create(&self, _: &dyn CreateArg) -> Result<Box<dyn Action>, Error> {
        Ok(Box::new(Lua {}))
    }
}

struct Lua {}

#[async_trait]
impl Action for Lua {
    async fn run(&self, arg: &dyn RunArg) -> Result<Box<dyn Scope>, Error> {
        let rt = rlua::Lua::new_with(
            StdLib::BASE | StdLib::TABLE | StdLib::STRING | StdLib::UTF8 | StdLib::MATH,
        );
        rt.set_memory_limit(Some(1024000));
        rt.context(|lua| {
            let args = arg.args();

            if let Some(globals) = args["global"].as_object() {
                for (k, v) in globals {
                    let v = rlua_serde::to_value(lua, v)?;
                    lua.globals().set(k.as_str(), v)?;
                }
            }

            let code = arg.args()["code"]
                .as_str()
                .ok_or(err!("100", "missing code"))?;

            self.eval(lua, code.to_string())
        })
    }
}

impl Lua {
    fn eval(&self, lua: rlua::Context, code: String) -> Result<Box<dyn Scope>, Error> {
        match lua.load(code.as_str()).eval::<rlua::Value>() {
            Ok(v) => {
                let v: Value = to_value(&v)?;
                Ok(Box::new(v))
            }
            Err(e) => Err(err!("101", format!("{}", e))),
        }
    }
}

fn to_value(lua_value: &rlua::Value) -> Result<Value, Error> {
    match lua_value {
        rlua::Value::Nil => Ok(Value::Null),
        rlua::Value::String(v) => Ok(Value::String(v.to_str()?.to_string())),
        rlua::Value::Integer(v) => Ok(Value::Number(Number::from(v.clone()))),
        rlua::Value::Boolean(v) => Ok(Value::Bool(v.clone())),

        rlua::Value::Number(v) => {
            Ok(Number::from_f64(v.clone()).map_or(Value::Null, |v| Value::Number(v)))
        }
        rlua::Value::Table(v) => {
            let mut map = Map::new();
            for pair in v.clone().pairs::<String, rlua::Value>() {
                let (k, v) = pair?;
                let v = to_value(&v)?;
                map.insert(k, v);
            }
            Ok(Value::Object(map))
        }

        _ => Err(err!("102", "invalid value")),
    }
}
