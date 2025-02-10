use std::sync::Arc;

use crate::{
    parse_tree::{if_next, if_next_or_none, peek_nth, require_next, try_parse_fn, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::VariableList;

#[derive(Debug, Clone, PartialEq)]
pub struct ClassDecl {
    pub slice: StringSlice,
    pub export: bool,
    pub name: Arc<str>,
    pub generics: Option<VariableList>,
    pub extends: Option<Arc<str>>,
    pub params: Option<VariableList>,
}

impl ClassDecl {
    fn parse_keyword(tokenizer: &mut Tokenizer) -> Result<Option<bool>, ParserError> {
        if_next!(TokenKind::Keyword(Keyword::Class), tokenizer, {
            return Ok(Some(false));
        });

        peek_nth!(TokenKind::Keyword(Keyword::Export), 0, tokenizer);
        peek_nth!(TokenKind::Keyword(Keyword::Class), 1, tokenizer);

        tokenizer.next()?;
        tokenizer.next()?;

        return Ok(Some(true));
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_parse_fn!(export, Self::parse_keyword, tokenizer);

        let mut end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Identifier(name), tokenizer);

        let generics = if_next_or_none!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer, {
            let generics = VariableList::try_parse(tokenizer)?;

            require_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer);

            generics
        });

        let extends = if_next_or_none!(TokenKind::Keyword(Keyword::Extends), tokenizer, {
            end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Identifier(name), tokenizer);

            Some(name)
        });

        let params = if_next_or_none!(TokenKind::Keyword(Keyword::Is), tokenizer, {
            let params = VariableList::try_parse(tokenizer)?;

            end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Keyword(Keyword::End), tokenizer);
            params
        });

        return Ok(Some(Self {
            slice: start.merge(&end),
            export,
            name,
            generics,
            extends,
            params,
        }));
    }
}
