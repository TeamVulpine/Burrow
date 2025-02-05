use std::sync::Arc;

use crate::{
    parse_tree::{
        if_next, if_parse, if_parse_fn, next_else, require_next, require_parse, require_parse_fn,
        while_next,
    },
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::{try_next, try_parse_fn, ParserError};

#[derive(Debug, Clone, PartialEq)]
pub struct Type {
    pub slice: StringSlice,
    pub kind: TypeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TypeKind {
    Value(ValueType),
    Function(FunctionType),
    Or(Arc<[Type]>),
    And(Arc<[Type]>),
    Prototype(Arc<Type>),
    Class,
    This,
    None,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ValueType {
    pub slice: StringSlice,
    pub name: Arc<str>,
    pub generics: Arc<[Type]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FunctionType {
    pub slice: StringSlice,
    pub params: Arc<[Type]>,
    pub ret: Option<Arc<Type>>,
}

impl Type {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        return Self::try_parse_and(tokenizer);
    }

    pub fn try_parse_and(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse_fn!(ty, Self::try_parse_or, tokenizer);

        let mut tys = vec![ty];

        while_next!(TokenKind::Keyword(Keyword::And), _, tokenizer, {
            require_parse_fn!(ty, Self::try_parse_or, tokenizer);

            tys.push(ty);
        });

        if tys.len() == 1 {
            return Ok(Some(tys.first().unwrap().clone()));
        }

        return Ok(Some(Self {
            slice: tys.first().unwrap().slice.merge(&tys.last().unwrap().slice),
            kind: TypeKind::Or(tys.into_boxed_slice().into()),
        }));
    }

    pub fn try_parse_or(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse_fn!(ty, Self::try_parse_basic, tokenizer);

        let mut tys = vec![ty];

        while_next!(TokenKind::Keyword(Keyword::Or), _, tokenizer, {
            require_parse_fn!(ty, Self::try_parse_basic, tokenizer);

            tys.push(ty);
        });

        if tys.len() == 1 {
            return Ok(Some(tys.first().unwrap().clone()));
        }

        return Ok(Some(Self {
            slice: tys.first().unwrap().slice.merge(&tys.last().unwrap().slice),
            kind: TypeKind::Or(tys.into_boxed_slice().into()),
        }));
    }

    pub fn try_parse_basic(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse_fn!(paren, Self::try_parse_paren, tokenizer, {
            return Ok(Some(paren));
        });

        if_parse!(func, FunctionType, tokenizer, {
            return Ok(Some(Self {
                slice: func.slice.clone(),
                kind: TypeKind::Function(func),
            }));
        });

        if_parse_fn!(value, Self::try_parse_value, tokenizer, {
            return Ok(Some(value));
        });

        if_parse_fn!(value, Self::try_parse_prototype, tokenizer, {
            return Ok(Some(value));
        });

        let start = tokenizer.peek(0)?.slice;
        if_next!(TokenKind::Keyword(Keyword::Class), tokenizer, {
            return Ok(Some(Self {
                slice: start,
                kind: TypeKind::Class,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::This), tokenizer, {
            return Ok(Some(Self {
                slice: start,
                kind: TypeKind::This,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::None), tokenizer, {
            return Ok(Some(Self {
                slice: start,
                kind: TypeKind::None,
            }));
        });

        return Ok(None);
    }

    pub fn try_parse_value(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Identifier(name), tokenizer);

        if let TokenKind::Symbol(Symbol::BracketOpen) = tokenizer.peek(0)?.kind {
            if let TokenKind::Symbol(Symbol::BracketClose) = tokenizer.peek(0)?.kind {
                return Ok(Some(Self {
                    slice: start.clone(),
                    kind: TypeKind::Value(ValueType {
                        slice: start,
                        name,
                        generics: Arc::new([]),
                    }),
                }));
            }

            tokenizer.next()?;

            let mut tys = vec![];

            let end = loop {
                let end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer, {
                    break end;
                });

                require_parse!(ty, Type, tokenizer);

                tys.push(ty);

                let end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer, {
                    break end;
                });

                require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
            };

            let slice = start.merge(&end);

            return Ok(Some(Self {
                slice: slice.clone(),
                kind: TypeKind::Value(ValueType {
                    slice,
                    name,
                    generics: tys.into_boxed_slice().into(),
                }),
            }));
        }

        return Ok(Some(Self {
            slice: start.clone(),
            kind: TypeKind::Value(ValueType {
                slice: start,
                name,
                generics: Arc::new([]),
            }),
        }));
    }

    pub fn try_parse_paren(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer);

        require_parse!(ty, Type, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            kind: ty.kind,
        }));
    }

    pub fn try_parse_prototype(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Prototype), tokenizer);

        require_next!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer);

        require_parse!(ty, Type, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            kind: TypeKind::Prototype(Arc::new(ty)),
        }));
    }
}

impl FunctionType {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Function), tokenizer);

        require_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer);

        let mut params: Vec<Type> = vec![];
        let mut end = tokenizer.peek(0)?.slice;

        next_else!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
            loop {
                require_parse!(param, Type, tokenizer);

                params.push(param);

                end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
                    break;
                });

                require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
            }
        });

        let ret = Type::try_parse(tokenizer)?;

        let end = ret.clone().map(|it| it.slice).unwrap_or(end);

        return Ok(Some(Self {
            slice: start.merge(&end),
            params: params.into_boxed_slice().into(),
            ret: ret.map(Arc::new),
        }));
    }
}
