use std::sync::Arc;

use crate::string::StringSlice;

#[derive(Debug)]
pub enum OpCode {
    SetSlice {
        slice: StringSlice,
    },

    /// Pushes a value
    PushVariable {
        name: Arc<str>,
    },
    /// Pushes the currently held exception
    PushException,
    /// Pushes the current "this" value
    PushThis,
    /// Pops the value at the top of the stack and pushes it's prototype
    PushPrototype,
    StoreProtorype,

    /// Pushes a constant integer
    PushConstInt {
        value: isize,
    },
    /// Pushes a constant float
    PushConstFloat {
        value: f32,
    },
    /// Pushes a constant boolean
    PushConstBool {
        value: bool,
    },
    PushConstString {
        value: Arc<str>,
    },
    PushFunction {
        index: usize,
    },
    PushNewObject,
    PushNewArray {
        initial_size: usize,
    },

    /// Pushes a constant none
    PushConstNone,

    /// Stores the value at the top of the stack
    StoreVariable {
        name: Arc<str>,
    },

    /// Adds a value to the current context, taking precedence over contexts further down the chain
    InitVariable {
        name: Arc<str>,
    },

    /// Marks the value as const
    MarkVariableConst {
        name: Arc<str>,
    },

    /// Stack structure: <params...> <function> <this?>
    Invoke {
        param_count: usize,
        this_call: bool,
    },

    PushContext,
    PopContext,

    /// Stack structure: <index> <object>
    PushIndex,
    /// Stack structure: <value> <index> <object>
    StoreIndex,

    /// Duplicates the value at the top of the stack
    Dupe,

    /// Pops the value at the top of the stack
    Pop,
    /// Throws the value at the top of the stack
    Throw,
    /// Returns the value at the top of the stack
    Return,

    OpAdd,
    OpSub,
    OpMul,
    OpDiv,
    OpRem,
    OpGe,
    OpLe,
    OpGt,
    OpLt,
    OpEq,
    OpNe,
    OpOr,
    OpAnd,
    OpUnaryAdd,
    OpUnarySub,
    OpUnaryNot,

    /// Pops the first two values off of the stack (The top of which needs to be a prototype, otherwise it throws an exception) and check if the value is the prototype
    ProtoEq,
    /// Pops the first two values off of the stack (The top of which needs to be a prototype, otherwise it throws an exception) and check if the value is not the prototype
    ProtoNe,

    /// Jumps to that location
    Jump {
        location: usize,
    },
    /// Jumps to that location if the value at the top of the stack is truthy
    JumpTrue {
        location: usize,
    },
    /// Jumps to that location if the value at the top of the stack is falsy
    JumpFalse {
        location: usize,
    },
    /// Pushes a catch to move to that location
    PushCatch {
        location: usize,
    },
    /// Pops the catch
    PopCatch,

    /// This can only be called in the init function.
    Import {
        path: Arc<str>,
    },

    Export {
        name: Arc<str>,
    },

    /// A temporary instruction to store a break stmt. An error should be thrown if come across during execution
    TempBreak,
    /// A temporary instruction to store a continue stmt. An error should be thrown if come across during execution
    TempContinue,
}
