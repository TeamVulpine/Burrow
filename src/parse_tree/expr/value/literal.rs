use std::sync::Arc;

use crate::{
    parse_tree::{if_next, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Number, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralExpr {
    pub slice: StringSlice,
    pub kind: LiteralExprKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum LiteralExprKind {
    Number(Number),
    String(Arc<str>),
    Bool(bool),
    Variable(Arc<str>),
    This,
    Infinity,
    NaN,
    None,
}

impl LiteralExpr {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let slice = tokenizer.peek(0)?.slice;

        if_next!(TokenKind::Number(number), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::Number(number),
            }));
        });

        if_next!(TokenKind::String(string), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::String(string),
            }));
        });

        if_next!(TokenKind::Identifier(ident), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::Variable(ident),
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::True), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::Bool(true),
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::False), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::Bool(false),
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::None), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::None,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::Infinity), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::Infinity,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::NaN), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::NaN,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::This), tokenizer, {
            return Ok(Some(Self {
                slice,
                kind: LiteralExprKind::This,
            }));
        });

        return Ok(None);
    }
}
