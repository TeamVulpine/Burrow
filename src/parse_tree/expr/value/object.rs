use std::sync::Arc;

use crate::{
    parse_tree::{
        allow_accidental, expr::Expr, if_next, require_next, require_parse, try_next, ParserError,
    },
    string::StringSlice,
    tokenizer::{
        token::{Symbol, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectExpr {
    pub slice: StringSlice,
    pub values: Arc<[ObjectValue]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ObjectValue {
    pub slice: StringSlice,
    pub name: Arc<str>,
    pub value: Expr,
}

impl ObjectExpr {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Symbol(Symbol::BraceOpen), tokenizer);

        let mut values = vec![];

        let end = loop {
            let end = tokenizer.peek(0)?.slice;
            if_next!(TokenKind::Symbol(Symbol::BraceClose), tokenizer, {
                break end;
            });

            require_parse!(value, ObjectValue, tokenizer);

            values.push(value);

            allow_accidental!(
                TokenKind::Symbol(Symbol::Comma | Symbol::Semicolon),
                tokenizer
            );
        };

        return Ok(Some(Self {
            slice: start.merge(&end),
            values: values.into_boxed_slice().into(),
        }));
    }
}

impl ObjectValue {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Identifier(name), tokenizer);

        require_next!(TokenKind::Symbol(Symbol::Assign), tokenizer);

        require_parse!(value, Expr, tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&value.slice),
            name,
            value,
        }));
    }
}
