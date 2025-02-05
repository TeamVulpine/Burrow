use std::sync::Arc;

use crate::{
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
    Await,
}

impl UnaryOpKind {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let peek = tokenizer.peek(0)?;

        return Ok(Some(match peek.kind {
            TokenKind::Symbol(Symbol::Add) => Self::Add,
            TokenKind::Symbol(Symbol::Sub) => Self::Sub,
            TokenKind::Keyword(Keyword::Not) => Self::Not,
            TokenKind::Keyword(Keyword::Await) => Self::Await,

            _ => return Ok(None),
        }));
    }
}
