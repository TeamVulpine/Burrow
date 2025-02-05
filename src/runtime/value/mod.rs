use std::sync::Arc;

use reference_pool::Reference;
use string_pool::StrReference;

use super::Runtime;

pub mod reference_pool;
pub mod string_pool;

pub enum Value {
    Integer(i64),
    Float(f64),
    Boolean(bool),
    String(StrReference),
    Reference(Reference),
    None,
}

pub struct Invocation {
    pub runtime: Arc<Runtime>,
    pub this: Value,
    pub parameters: Vec<Value>,
}

pub type Invocable = dyn Fn(Invocation) -> Result<Value, Value>;
