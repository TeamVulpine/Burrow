use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use value::{
    object_pool::ObjectPool,
    string_pool::{StrReference, StringPool},
    Value,
};

use crate::bytecode::CompiledModule;

pub mod value;

pub struct Runtime {
    pub string_pool: Arc<StringPool>,
    pub object_pool: Arc<ObjectPool>,
    pub module_cache: RwLock<HashMap<StrReference, Arc<Module>>>,
}

pub struct Module {
    pub bytecode: Option<Arc<BytecodeModule>>,
    pub export: Value,
}

pub struct BytecodeModule {
    pub bytecode: CompiledModule,
    pub context: Value,
}

impl Runtime {
    pub fn new() -> Self {
        let string_pool = StringPool::new();
        let reference_pool = ObjectPool::new();

        return Self {
            string_pool,
            object_pool: reference_pool,
            module_cache: RwLock::new(HashMap::new()),
        };
    }

    pub fn create_native_module(&self, name: Arc<str>, value: Value) -> Arc<Module> {
        let mut modules = self.module_cache.write().unwrap();

        return modules
            .insert(
                self.string_pool.acquire(name).unwrap(),
                Arc::new(Module {
                    bytecode: None,
                    export: value,
                }),
            )
            .unwrap();
    }
}
