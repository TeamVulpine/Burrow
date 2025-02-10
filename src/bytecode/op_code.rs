use std::sync::Arc;

pub enum OpCode {
    /// Pushes a value
    Push {
        name: Arc<str>,
    },
    /// Pushes the currently held exception
    PushException,
    /// Pushes the current "this" value
    PushThis,
    /// Pops the value at the top of the stack and pushes it's prototype
    PushPrototype,

    /// Pushes a constant integer
    PushConstInt {
        value: i32,
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
        /// Set this to true if the function is to be used as a closure
        use_current_context: bool
    },
    PushNewObject,
    PushNewArray,

    /// Pushes a constant none
    PushConstNone,

    /// Stores the value at the top of the stack
    Store {
        name: Arc<str>,
    },

    /// Adds a value to the current context, taking precedence over contexts further down the chain
    InitValue {
        name: Arc<str>,
    },
    /// Marks the value as const
    MarkValueConst {
        name: Arc<str>,
    },

    PushIndex,
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
    OpNot,
    OpOr,
    OpAnd,

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
}
