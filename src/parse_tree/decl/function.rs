use std::sync::Arc;

use crate::{
    parse_tree::{
        if_next_or_none, if_parse_or_none, is_next, next_else, peek_nth, require_next,
        require_parse, stmt::Block, try_next, try_parse, ty::Type, ParserError,
    },
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::VariableList;

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionImpl {
    pub slice: StringSlice,
    pub export: bool,
    pub decl: FunctionDecl,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionDecl {
    pub slice: StringSlice,
    pub base: Option<Arc<str>>,
    pub name: Arc<str>,
    pub generics: Option<VariableList>,
    pub this: bool,
    pub this_ty: Option<Type>,
    pub params: Option<VariableList>,
    pub ty: Option<Type>,
}

impl FunctionImpl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let (decl, export) = if let Some(decl) = FunctionDecl::try_parse_with_export(tokenizer)? {
            (decl, true)
        } else {
            try_parse!(decl, FunctionDecl, tokenizer);
            (decl, false)
        };

        require_parse!(block, Block, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: decl.slice.merge(&end),
            export,
            decl,
            block,
        }));
    }
}

impl FunctionDecl {
    pub fn try_parse_with_export(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        peek_nth!(TokenKind::Keyword(Keyword::Export), 0, tokenizer);
        peek_nth!(TokenKind::Keyword(Keyword::Function), 1, tokenizer);

        tokenizer.next()?;
        tokenizer.next()?;

        return Ok(Some(Self::parse_base(tokenizer, start)?));
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Function), tokenizer);

        return Ok(Some(Self::parse_base(tokenizer, start)?));
    }

    fn parse_base(tokenizer: &mut Tokenizer, start: StringSlice) -> Result<Self, ParserError> {
        require_next!(TokenKind::Identifier(mut name), tokenizer);

        let base: Option<Arc<str>> = if_next_or_none!(TokenKind::Symbol(Symbol::Dot), tokenizer, {
            require_next!(TokenKind::Identifier(new_name), tokenizer);
            let old_name = name;
            name = new_name;
            Some(old_name)
        });

        let generics = if_next_or_none!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer, {
            let generics = VariableList::try_parse(tokenizer)?;

            require_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer);

            generics
        });

        require_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer);

        let this = is_next!(TokenKind::Keyword(Keyword::This), tokenizer);

        let this_ty: Option<Type> = if this {
            if_parse_or_none!(this_ty, Type, tokenizer, { Some(this_ty) })
        } else {
            None
        };

        let mut params = None;

        let mut end = tokenizer.peek(0)?.slice;

        next_else!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
            if this {
                require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
            }

            params = VariableList::try_parse(tokenizer)?;

            end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer);
        });

        let ty: Option<Type> = if_next_or_none!(TokenKind::Symbol(Symbol::Colon), tokenizer, {
            require_parse!(ty, Type, tokenizer);
            Some(ty)
        });

        let end = ty.clone().map(|it| it.slice).unwrap_or(end);

        return Ok(Self {
            slice: start.merge(&end),
            base,
            name,
            generics,
            this,
            this_ty,
            params,
            ty,
        });
    }
}
