use crate::{
    parse_tree::{
        expr::Expr, if_next_or_none, if_parse_fn, require_parse, try_next, try_parse, ParserError,
    },
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::VariableName;

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub slice: StringSlice,
    pub is_const: bool,
    pub param: VariableName,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableImpl {
    pub slice: StringSlice,
    pub decl: VariableDecl,
    pub init: Option<Expr>,
}

impl VariableDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse_fn!(var, Self::try_parse_let, tokenizer, {
            return Ok(Some(var));
        });

        if_parse_fn!(var, Self::try_parse_const, tokenizer, {
            return Ok(Some(var));
        });

        return Ok(None);
    }

    pub fn try_parse_let(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Let), tokenizer);

        require_parse!(param, VariableName, tokenizer);

        let end = param.slice.clone();

        return Ok(Some(Self {
            slice: start.merge(&end),
            is_const: false,
            param,
        }));
    }

    pub fn try_parse_const(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Const), tokenizer);

        require_parse!(param, VariableName, tokenizer);

        let end = param.slice.clone();

        return Ok(Some(Self {
            slice: start.merge(&end),
            is_const: false,
            param,
        }));
    }
}

impl VariableImpl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse!(decl, VariableDecl, tokenizer);

        let init: Option<Expr> = if_next_or_none!(TokenKind::Symbol(Symbol::Assign), tokenizer, {
            require_parse!(expr, Expr, tokenizer);

            Some(expr)
        });

        let end = init
            .clone()
            .map(|it| it.slice)
            .unwrap_or(decl.slice.clone());

        return Ok(Some(Self {
            slice: decl.slice.merge(&end),
            decl,
            init,
        }));
    }
}
