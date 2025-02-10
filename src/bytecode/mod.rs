use std::sync::Arc;

use op_code::OpCode;

pub mod op_code;

pub struct CompiledModule {
    pub functions: Arc<[Function]>,
    pub init: Function,
}

pub struct Function {
    pub params: Arc<[Arc<str>]>,
    pub body: Arc<OpCode>,
}
