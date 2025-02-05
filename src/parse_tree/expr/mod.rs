use std::sync::Arc;

use access::AccessExpr;
use op::{
    binary::{BinOpExpr, BinOpKind},
    unary::{UnaryOpExpr, UnaryOpKind},
};
use value::{array::ArrayExpr, literal::LiteralExpr, object::ObjectExpr};

use crate::{
    parse_tree::{if_next, if_parse, require_next, require_parse},
    string::StringSlice,
    tokenizer::{
        token::{Symbol, TokenKind},
        Tokenizer,
    },
};

use super::ParserError;

pub mod access;
pub mod op;
pub mod value;

#[derive(Debug, PartialEq, Clone)]
pub struct Expr {
    pub slice: StringSlice,
    pub kind: ExprKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind {
    Literal(LiteralExpr),
    Object(ObjectExpr),
    Array(ArrayExpr),
    BinOp(BinOpExpr),
    UnaryOp(UnaryOpExpr),
    Access(AccessExpr),
}

impl Expr {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        return Self::try_parse_binop(tokenizer, 0);
    }

    pub fn try_parse_binop(
        tokenizer: &mut Tokenizer,
        binding: usize,
    ) -> Result<Option<Self>, ParserError> {
        let Some(mut expr) = Self::try_parse_unary(tokenizer)? else {
            return Ok(None);
        };

        loop {
            let Some(op) = BinOpKind::try_parse(tokenizer)? else {
                break;
            };

            let (lhs_binding, rhs_binding) = op.binding();
            if lhs_binding < binding {
                break;
            }

            for _ in 0..(op.consume_amount()) {
                tokenizer.next()?;
            }

            let peek = tokenizer.peek(0)?;
            let Some(rhs) = Self::try_parse_binop(tokenizer, rhs_binding)? else {
                return Err(ParserError::unexpected_token(peek));
            };

            let slice = expr.slice.merge(&rhs.slice);

            expr = Expr {
                slice: slice.clone(),
                kind: ExprKind::BinOp(BinOpExpr {
                    slice,
                    lhs: Arc::new(expr),
                    op,
                    rhs: Arc::new(rhs),
                }),
            };
        }

        return Ok(Some(expr));
    }

    pub fn try_parse_unary(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let mut unary_ops = vec![];

        loop {
            let slice = tokenizer.peek(0)?.slice;
            let Some(op) = UnaryOpKind::try_parse(tokenizer)? else {
                break;
            };
            tokenizer.next()?;

            unary_ops.push((slice, op));
        }

        let Some(mut expr) = Self::try_parse_access(tokenizer)? else {
            return Ok(None);
        };

        while let Some((slice, op)) = unary_ops.pop() {
            let slice = slice.merge(&expr.slice);
            expr = Expr {
                slice: slice.clone(),
                kind: ExprKind::UnaryOp(UnaryOpExpr {
                    slice,
                    op,
                    value: Arc::new(expr),
                }),
            };
        }

        return Ok(Some(expr));
    }

    pub fn try_parse_access(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let Some(mut value) = Self::try_parse_paren(tokenizer)? else {
            return Ok(None);
        };

        while let Some(access) = AccessExpr::try_parse(tokenizer, &value)? {
            value = Self {
                slice: access.slice.clone(),
                kind: ExprKind::Access(access),
            };
        }

        return Ok(Some(value));
    }

    pub fn try_parse_paren(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        if_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer, {
            require_parse!(value, Expr, tokenizer);

            let end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer);

            return Ok(Some(Self {
                slice: start.merge(&end),
                kind: value.kind,
            }));
        });

        return Self::try_parse_value(tokenizer);
    }

    pub fn try_parse_value(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse!(array, ArrayExpr, tokenizer, {
            return Ok(Some(Self {
                slice: array.slice.clone(),
                kind: ExprKind::Array(array),
            }));
        });

        if_parse!(obj, ObjectExpr, tokenizer, {
            return Ok(Some(Self {
                slice: obj.slice.clone(),
                kind: ExprKind::Object(obj),
            }));
        });

        if_parse!(lit, LiteralExpr, tokenizer, {
            return Ok(Some(Self {
                slice: lit.slice.clone(),
                kind: ExprKind::Literal(lit),
            }));
        });

        return Ok(None);
    }
}
