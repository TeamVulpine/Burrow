use std::sync::Arc;

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
    Uninitialized,
}

impl Value {
    #[allow(unused_variables)]
    fn invoke(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        params: &[Value],
    ) -> Result<Value, Value> {
        if let Self::Object(obj) = self {
            let native_value = obj.get().native_value.read().unwrap().clone();

            let Some(native_value) = native_value else {
                return Err(Value::String(runtime.string_pool.acquire("Cannot invoke value".into()).unwrap()));
            };

            if !native_value.has_invoker(runtime.clone()) {
                return Err(Value::String(runtime.string_pool.acquire("Cannot invoke value".into()).unwrap()));
            }

            return native_value.invoke(runtime.clone(), this_obj, params);
        }

        return Err(Value::String(runtime.string_pool.acquire("Cannot invoke value".into()).unwrap()));
    }
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

impl<TFn: Fn(Arc<Runtime>, &Value, &[Value]) -> Result<Value, Value>> NativeValue for TFn {
    #[allow(unused_variables)]
    fn mark_children(&self, marker: &mut MarkChildren) {}

    #[allow(unused_variables)]
    fn has_invoker(&self, runtime: Arc<Runtime>) -> bool {
        return true;
    }

    fn invoke(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        params: &[Value],
    ) -> Result<Value, Value> {
        return self(runtime, this_obj, params);
    }
}
