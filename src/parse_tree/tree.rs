use std::sync::Arc;

use crate::{
    parse_tree::{decl::function::FunctionImpl, require_next, try_next},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, TokenKind},
        Tokenizer,
    },
};

use super::{
    decl::{class::ClassDecl, import::ImportDecl, variable::VariableImpl, IdeDecl},
    stmt::Stmt,
    ParserError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    pub slice: StringSlice,
    pub imports: Arc<[ImportDecl]>,
    pub nodes: Arc<[ParseTreeNode]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTreeNode {
    pub slice: StringSlice,
    pub kind: ParseTreeNodeKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseTreeNodeKind {
    IdeDecl(IdeDecl),
    Decl(TopLevelDecl),
    Export(ExportDecl),
    Stmt(Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelDecl {
    pub slice: StringSlice,
    pub kind: TopLevelDeclKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TopLevelDeclKind {
    Class(ClassDecl),
    Function(FunctionImpl),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportDecl {
    pub slice: StringSlice,
    pub kind: ExportDeclKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExportDeclKind {
    TopLevel(TopLevelDecl),
    Variable(VariableImpl),
    Value(Arc<str>),
}

impl ParseTree {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        let mut imports = vec![];
        let mut end = start.clone();

        while let Some(import) = ImportDecl::try_parse(tokenizer)? {
            end = import.slice.clone();
            imports.push(import);
        }

        let mut nodes = vec![];

        while let Some(node) = ParseTreeNode::try_parse(tokenizer)? {
            end = node.slice.clone();
            nodes.push(node);
        }

        return Ok(Some(Self {
            slice: start.merge(&end),
            imports: imports.into_boxed_slice().into(),
            nodes: nodes.into_boxed_slice().into(),
        }));
    }
}

impl ParseTreeNode {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if let Some(ide_decl) = IdeDecl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: ide_decl.slice.clone(),
                kind: ParseTreeNodeKind::IdeDecl(ide_decl),
            }));
        }

        if let Some(decl) = TopLevelDecl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: ParseTreeNodeKind::Decl(decl),
            }));
        }

        if let Some(decl) = ExportDecl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: decl.slice.clone(),
                kind: ParseTreeNodeKind::Export(decl),
            }));
        }

        if let Some(stmt) = Stmt::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: stmt.slice.clone(),
                kind: ParseTreeNodeKind::Stmt(stmt),
            }));
        }

        return Ok(None);
    }
}

impl TopLevelDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if let Some(class) = ClassDecl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: class.slice.clone(),
                kind: TopLevelDeclKind::Class(class),
            }));
        }

        if let Some(function) = FunctionImpl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: function.slice.clone(),
                kind: TopLevelDeclKind::Function(function),
            }));
        }

        return Ok(None);
    }
}

impl ExportDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        try_next!(TokenKind::Keyword(Keyword::Export), tokenizer);

        if let Some(decl) = TopLevelDecl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: start.merge(&decl.slice),
                kind: ExportDeclKind::TopLevel(decl),
            }));
        }

        if let Some(decl) = VariableImpl::try_parse(tokenizer)? {
            return Ok(Some(Self {
                slice: start.merge(&decl.slice),
                kind: ExportDeclKind::Variable(decl),
            }));
        }

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Identifier(name), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            kind: ExportDeclKind::Value(name),
        }));
    }
}
