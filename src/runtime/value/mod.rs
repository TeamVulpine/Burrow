use std::sync::{Arc, RwLock};

use reference_pool::{MarkChildren, Reference};
use string_pool::StrReference;

use super::Runtime;

pub mod reference_pool;
pub mod string_pool;

#[derive(Clone)]
pub enum Value {
    String(StrReference),
    Reference(Reference),
    Integer(isize),
    Float(f32),
    Boolean(bool),
    None,
}

pub trait NativeValue {
    /// This function should not create or modify any values, as that will cause a deadlock.
    fn mark_children(&self, marker: &mut MarkChildren);

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

    /// Return Ok(None) to signify that there is no value there.
    #[allow(unused_variables)]
    fn get_index(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        index: &Value,
    ) -> Result<Option<Value>, Value> {
        return Ok(None);
    }

    /// Return Ok(None) to signify that there is no value there.
    #[allow(unused_variables)]
    fn set_index(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        index: &Value,
        value: &Value,
    ) -> Result<Option<Value>, Value> {
        return Ok(None);
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

    #[allow(unused_variables)]
    fn get_index(
        &self,
        runtime: Arc<Runtime>,
        this_obj: &Value,
        index: &Value,
    ) -> Result<Option<Value>, Value> {
        let index = if let Value::Integer(int) = index {
            *int
        } else if let Value::Float(float) = index {
            (*float) as isize
        } else {
            return Ok(None);
        };

        {
            let values = self.values.read().unwrap();

            if index as usize >= values.len() {
                return Ok(None);
            }
    
            return Ok(Some(values[index as usize].read().unwrap().clone()));
        }
    }

    #[allow(unused_variables)]
    fn set_index(
            &self,
            runtime: Arc<Runtime>,
            this_obj: &Value,
            index: &Value,
            value: &Value,
    ) -> Result<Option<Value>, Value> {
        let index = if let Value::Integer(int) = index {
            *int
        } else if let Value::Float(float) = index {
            (*float) as isize
        } else {
            return Ok(None);
        };

        {
            let mut values = self.values.write().unwrap();
            while index as usize >= values.len() {
                values.push(RwLock::new(Value::None));
            }

            let mut out = values[index as usize].write().unwrap();
            let out: &mut Value = &mut out;
            *out = value.clone();
        }

        return Ok(Some(value.clone()));
    }
}
