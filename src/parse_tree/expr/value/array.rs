use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{expr::Expr, if_next, require_next, require_parse, try_next, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Symbol, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayExpr {
    pub slice: StringSlice,
    pub values: Arc<[Expr]>,
}

impl ArrayExpr {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        bytecode.push(OpCode::SetSlice {
            slice: self.slice.clone(),
        });

        let len = self.values.len();
        bytecode.push(OpCode::PushNewArray { initial_size: len });

        for index in 0..len {
            let value = &self.values[index];
            bytecode.push(OpCode::PushConstInt {
                value: index as isize,
            });

            value.generate_bytecode(bytecode)?;
            
            bytecode.push(OpCode::StoreIndex);
            bytecode.push(OpCode::Pop);
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        try_next!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer);

        let mut values = vec![];

        let end = loop {
            let end = tokenizer.peek(0)?.slice;
            if_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer, {
                break end;
            });

            require_parse!(value, Expr, tokenizer);
            values.push(value);

            let end = tokenizer.peek(0)?.slice;
            if_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer, {
                break end;
            });

            require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
        };

        return Ok(Some(Self {
            slice: start.merge(&end),
            values: values.into_boxed_slice().into(),
        }));
    }
}
