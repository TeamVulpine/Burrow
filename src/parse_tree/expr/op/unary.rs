use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{expr::Expr, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOpExpr {
    pub slice: StringSlice,
    pub op: UnaryOpKind,
    pub value: Arc<Expr>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum UnaryOpKind {
    Add,
    Sub,
    Not,
}

impl UnaryOpExpr {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        self.value.generate_bytecode(bytecode)?;

        bytecode.push(OpCode::SetSlice {
            slice: self.slice.clone(),
        });

        match self.op {
            UnaryOpKind::Add => bytecode.push(OpCode::OpUnaryAdd),
            UnaryOpKind::Sub => bytecode.push(OpCode::OpUnarySub),
            UnaryOpKind::Not => bytecode.push(OpCode::OpUnaryNot),
        }

        return Ok(());
    }
}

impl UnaryOpKind {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let peek = tokenizer.peek(0)?;

        return Ok(Some(match peek.kind {
            TokenKind::Symbol(Symbol::Add) => Self::Add,
            TokenKind::Symbol(Symbol::Sub) => Self::Sub,
            TokenKind::Keyword(Keyword::Not) => Self::Not,

            _ => return Ok(None),
        }));
    }
}
