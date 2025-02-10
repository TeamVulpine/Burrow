use std::sync::Arc;

use crate::{
    parse_tree::{if_next, require_next, try_next, try_parse, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, Clone, PartialEq)]
pub struct ImportDecl {
    pub slice: StringSlice,
    pub kind: ImportKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImportKind {
    Direct(DirectImport),
    From(FromImport),
}

/// import "file.bur"
#[derive(Debug, Clone, PartialEq)]
pub struct DirectImport {
    pub slice: StringSlice,
    pub file: Arc<str>,
}

/// from "file.bur" import x
///
/// from "file.bur" export x
#[derive(Debug, Clone, PartialEq)]
pub struct FromImport {
    pub slice: StringSlice,
    pub file: Arc<str>,
    pub values: Arc<[FromInportValue]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FromInportValue {
    pub slice: StringSlice,
    pub kind: FromImportKind,
    pub rename: Option<Arc<str>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FromImportKind {
    Everything,
    Single(Arc<str>),
}

impl ImportDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if let Some(direct) = DirectImport::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: direct.slice.clone(),
                kind: ImportKind::Direct(direct),
            }));
        }

        try_parse!(from, FromImport, tokenizer);

        return Ok(Some(Self {
            slice: from.slice.clone(),
            kind: ImportKind::From(from),
        }));
    }
}

impl DirectImport {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Import), tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::String(file), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            file,
        }));
    }
}

impl FromImport {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::From), tokenizer);

        require_next!(TokenKind::String(file), tokenizer);

        let end = tokenizer.peek(0)?.slice;

        require_next!(TokenKind::Keyword(Keyword::Import), tokenizer);

        let mut values = vec![];
        let mut end = end;

        while let Some(value) = FromInportValue::try_parse(tokenizer)? {
            end = value.slice.clone();

            values.push(value);
        }

        return Ok(Some(Self {
            slice: start.merge(&end),
            file,
            values: values.into_boxed_slice().into(),
        }));
    }
}

impl FromInportValue {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        if_next!(TokenKind::Keyword(Keyword::Everything), tokenizer, {
            if let Some((rename, end)) = Self::try_parse_as(tokenizer)? {
                return Ok(Some(Self {
                    slice: start.merge(&end),
                    kind: FromImportKind::Everything,
                    rename: Some(rename),
                }));
            }

            return Ok(Some(Self {
                slice: start,
                kind: FromImportKind::Everything,
                rename: None,
            }));
        });

        try_next!(TokenKind::Identifier(name), tokenizer);

        if let Some((rename, end)) = Self::try_parse_as(tokenizer)? {
            return Ok(Some(Self {
                slice: start.merge(&end),
                kind: FromImportKind::Single(name),
                rename: Some(rename),
            }));
        }

        return Ok(Some(Self {
            slice: start,
            kind: FromImportKind::Single(name),
            rename: None,
        }));
    }

    fn try_parse_as(
        tokenizer: &mut Tokenizer,
    ) -> Result<Option<(Arc<str>, StringSlice)>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::As), tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Identifier(name), tokenizer);

        return Ok(Some((name, start.merge(&end))));
    }
}
