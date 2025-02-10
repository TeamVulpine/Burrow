use std::sync::Arc;

use op_code::OpCode;

pub mod op_code;

pub struct CompiledModule {
    pub functions: Arc<[Function]>,
    pub init: Function,
}

pub struct Function {
    pub param_count: usize,
    pub instance_func: bool,
    pub body: Arc<OpCode>,
}
