use std::sync::Arc;

use value::{reference_pool::ReferencePool, string_pool::StringPool};

pub mod value;

pub struct Runtime {
    pub string_pool: Arc<StringPool>,
    pub reference_pool: Arc<ReferencePool>,
}

impl Runtime {
    pub fn new() -> Self {
        return Self {
            string_pool: StringPool::new(),
            reference_pool: ReferencePool::new(),
        };
    }
}
