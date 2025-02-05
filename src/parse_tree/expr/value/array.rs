use std::sync::Arc;

use crate::{
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
