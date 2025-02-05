use std::sync::Arc;

use crate::{
    parse_tree::{if_next, require_next, require_parse, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::Expr;

#[derive(Debug, PartialEq, Clone)]
pub struct AccessExpr {
    pub slice: StringSlice,
    pub base: Arc<Expr>,
    pub kind: AccessKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AccessKind {
    Ident(Arc<str>),
    Index(Arc<Expr>),
    Invoke(Arc<[Expr]>),
    Prototype,
}

impl AccessExpr {
    pub fn try_parse(tokenizer: &mut Tokenizer, base: &Expr) -> Result<Option<Self>, ParserError> {
        if_next!(TokenKind::Symbol(Symbol::Dot), tokenizer, {
            let end = tokenizer.peek(0)?.slice;
            if_next!(TokenKind::Keyword(Keyword::Prototype), tokenizer, {
                return Ok(Some(Self {
                    slice: base.slice.merge(&end),
                    base: Arc::new(base.clone()),
                    kind: AccessKind::Prototype,
                }));
            });

            require_next!(TokenKind::Identifier(ident), tokenizer);

            return Ok(Some(Self {
                slice: base.slice.merge(&end),
                base: Arc::new(base.clone()),
                kind: AccessKind::Ident(ident),
            }));
        });

        if_next!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer, {
            require_parse!(expr, Expr, tokenizer);

            let end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer);

            return Ok(Some(Self {
                slice: base.slice.merge(&end),
                base: Arc::new(base.clone()),
                kind: AccessKind::Index(Arc::new(expr)),
            }));
        });

        if_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer, {
            let mut values = vec![];

            let end = loop {
                let end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
                    break end;
                });

                require_parse!(value, Expr, tokenizer);
                values.push(value);

                let end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
                    break end;
                });

                require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
            };

            return Ok(Some(Self {
                slice: base.slice.merge(&end),
                base: Arc::new(base.clone()),
                kind: AccessKind::Invoke(values.into_boxed_slice().into()),
            }));
        });

        return Ok(None);
    }
}
