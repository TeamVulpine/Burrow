use std::sync::Arc;

use class::ClassDecl;
use function::FunctionDecl;
use variable::VariableDecl;

use crate::{
    parse_tree::{if_next_or_none, next_else, require_next, require_parse, try_next, try_parse},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::{if_parse, ty::Type, ParserError};

pub mod class;
pub mod function;
pub mod import;
pub mod variable;

#[derive(Debug, Clone, PartialEq)]
pub struct IdeDecl {
    pub slice: StringSlice,
    pub kind: IdeDeclKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IdeDeclKind {
    Function(FunctionDecl),
    Class(ClassDecl),
    Variable(VariableDecl),
    Module(IdeModule)
}

#[derive(Debug, Clone, PartialEq)]
pub struct IdeModule {
    pub slice: StringSlice,
    pub name: Arc<str>,
    pub values: Arc<[IdeDecl]>
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableName {
    pub slice: StringSlice,
    pub name: Arc<str>,
    pub ty: Option<Type>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableList {
    pub slice: StringSlice,
    pub values: Arc<[VariableName]>,
}

impl IdeDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Declare), tokenizer);

        if_parse!(decl, ClassDecl, tokenizer, {
            return Ok(Some(Self {
                slice: start.merge(&decl.slice),
                kind: IdeDeclKind::Class(decl),
            }));
        });

        if_parse!(decl, VariableDecl, tokenizer, {
            return Ok(Some(Self {
                slice: start.merge(&decl.slice),
                kind: IdeDeclKind::Variable(decl),
            }));
        });

        if_parse!(decl, IdeModule, tokenizer, {
            return Ok(Some(Self {
                slice: start.merge(&decl.slice),
                kind: IdeDeclKind::Module(decl),
            }));
        });

        require_parse!(decl, FunctionDecl, tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&decl.slice),
            kind: IdeDeclKind::Function(decl),
        }));
    }

    fn try_parse_no_declare(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse!(decl, ClassDecl, tokenizer, {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: IdeDeclKind::Class(decl),
            }));
        });

        if_parse!(decl, VariableDecl, tokenizer, {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: IdeDeclKind::Variable(decl),
            }));
        });

        if_parse!(decl, FunctionDecl, tokenizer, {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: IdeDeclKind::Function(decl),
            }));
        });
        
        return Ok(None);
    }
}

impl IdeModule {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        try_next!(TokenKind::Keyword(Keyword::Module), tokenizer);

        require_next!(TokenKind::String(name), tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Is), tokenizer);

        let mut decls = vec![];

        while let Some(decl) = IdeDecl::try_parse_no_declare(tokenizer)? {
            decls.push(decl);
        }

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            name,
            values: decls.into_boxed_slice().into()
        }));
    }
}

impl VariableName {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Identifier(name), tokenizer);

        let ty: Option<Type> = if_next_or_none!(TokenKind::Symbol(Symbol::Colon), tokenizer, {
            require_parse!(ty, Type, tokenizer);
            Some(ty)
        });

        let end = ty.clone().map(|it| it.slice).unwrap_or(start.clone());

        return Ok(Some(Self {
            slice: start.merge(&end),
            name,
            ty,
        }));
    }
}

impl VariableList {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_parse!(var, VariableName, tokenizer);

        let mut vars: Vec<VariableName> = vec![var];
        let mut end = start.clone();

        loop {
            next_else!(TokenKind::Symbol(Symbol::Comma), tokenizer, {
                break;
            });

            require_parse!(var, VariableName, tokenizer);
            end = var.slice.clone();
            vars.push(var);
        }

        return Ok(Some(Self {
            slice: start.merge(&end),
            values: vars.into_boxed_slice().into(),
        }));
    }
}
