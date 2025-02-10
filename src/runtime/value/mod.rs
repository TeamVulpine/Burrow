use std::sync::Arc;

use reference_pool::Reference;
use string_pool::StrReference;

use super::Runtime;

pub mod reference_pool;
pub mod string_pool;

#[derive(Clone)]
pub enum Value {
    String(StrReference),
    Reference(Reference),
    Integer(i32),
    Float(f32),
    Boolean(bool),
    None,
}

pub struct Invocation {
    pub runtime: Arc<Runtime>,
    pub this: Value,
    pub parameters: Vec<Value>,
}

pub type Invocable = dyn Fn(Invocation) -> Result<Value, Value>;
