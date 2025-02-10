use std::sync::Arc;

use op_code::OpCode;

use crate::{parse_tree::ParserError, string::StringSlice};

pub mod op_code;

pub struct CompiledModule {
    pub functions: Arc<[Function]>,
    pub init: Function,
}

pub struct Function {
    pub params: Arc<[Arc<str>]>,
    pub body: Arc<OpCode>,
}

#[derive(Debug)]
pub enum BytecodeGenerationError {
    ParserError(ParserError),
    IllegalAssignment(StringSlice),
    IllegalExport(StringSlice),
    IllegalBreak(StringSlice),
    IllegalContinue(StringSlice),
}

impl From<ParserError> for BytecodeGenerationError {
    fn from(value: ParserError) -> Self {
        return Self::ParserError(value);
    }
}
