use std::sync::Arc;

use control::ControlStmt;

use crate::{
    parse_tree::if_next,
    string::StringSlice,
    tokenizer::{
        token::{Symbol, TokenKind},
        Tokenizer,
    },
};

use super::{decl::variable::VariableImpl, expr::Expr, if_parse, ParserError};

pub mod control;

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub slice: StringSlice,
    pub kind: StmtKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Control(ControlStmt),
    Expr(Expr),
    Variable(VariableImpl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub slice: StringSlice,
    pub stmts: Arc<[Stmt]>,
}

impl Stmt {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse!(control, ControlStmt, tokenizer, {
            return Ok(Some(Self {
                slice: control.slice.clone(),
                kind: StmtKind::Control(control),
            }));
        });

        if_parse!(decl, VariableImpl, tokenizer, {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: StmtKind::Variable(decl),
            }));
        });

        if_parse!(expr, Expr, tokenizer, {
            return Ok(Some(Self {
                slice: expr.slice.clone(),
                kind: StmtKind::Expr(expr),
            }));
        });

        return Ok(None);
    }
}

impl Block {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        let mut stmts = vec![];
        let mut end = start.clone();

        while let Some(expr) = Stmt::try_parse(tokenizer)? {
            end = expr.slice.clone();
            stmts.push(expr);

            let peek = tokenizer.peek(0)?;
            if_next!(TokenKind::Symbol(Symbol::Semicolon), tokenizer, {
                end = peek.slice;
            });
        }

        if stmts.len() == 0 {
            return Ok(None);
        }

        return Ok(Some(Self {
            slice: start.merge(&end),
            stmts: stmts.into_boxed_slice().into(),
        }));
    }
}
