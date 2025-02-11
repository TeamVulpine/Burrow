use std::sync::{Arc, RwLock};

use object_pool::{MarkChildren, ObjectReference};
use string_pool::StrReference;

use super::Runtime;

pub mod object_pool;
pub mod string_pool;

#[derive(Clone)]
pub enum Value {
    String(StrReference),
    Object(ObjectReference),
    Integer(isize),
    Float(f32),
    Boolean(bool),
    None,
}

pub trait NativeValue {
    /// This function should not create or modify any values, as that will cause a deadlock.
    fn mark_children(&self, marker: &mut MarkChildren);

    #[allow(unused_variables)]
    fn cleanup(&self) {}

    #[allow(unused_variables)]
    fn has_invoker(&self, runtime: Arc<Runtime>) -> bool {
        return false;
    }

    #[allow(unused_variables)]
    fn invoke(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        params: &[Value],
    ) -> Result<Value, Value> {
        return Ok(Value::None);
    }
}

pub struct Array {
    pub values: RwLock<Vec<RwLock<Value>>>,
}

impl NativeValue for Array {
    fn mark_children(&self, marker: &mut MarkChildren) {
        let values = self.values.read().unwrap();
        for value in values.iter() {
            let value = value.read().unwrap();
            marker.mark_value(&value);
        }
    }
}
