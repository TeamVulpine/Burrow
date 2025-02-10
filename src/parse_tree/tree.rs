use std::sync::Arc;

use crate::{parse_tree::decl::function::FunctionImpl, string::StringSlice, tokenizer::Tokenizer};

use super::{
    decl::{class::ClassDecl, import::ImportDecl, IdeDecl},
    stmt::Stmt,
    ParserError,
};

#[derive(Debug, Clone, PartialEq)]
pub struct ParseTree {
    pub slice: StringSlice,
    pub imports: Arc<[ImportDecl]>,
    pub functions: Arc<[FunctionImpl]>,
    pub classes: Arc<[ClassDecl]>,
    pub stmts: Arc<[Stmt]>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TopLevelClass {
    pub decl: ClassDecl,
    pub export: bool,
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

        let mut stmts = vec![];
        let mut functions = vec![];
        let mut classes = vec![];

        loop {
            if let Some(_) = IdeDecl::try_parse(tokenizer)? {
                continue;
            }

            if let Some(stmt) = Stmt::try_parse(tokenizer)? {
                stmts.push(stmt);
                continue;
            }

            if let Some(function) = FunctionImpl::try_parse(tokenizer)? {
                functions.push(function);
                continue;
            }

            if let Some(class) = ClassDecl::try_parse(tokenizer)? {
                classes.push(class);
                continue;
            }

            break;
        }

        return Ok(Some(Self {
            slice: start.merge(&end),
            imports: imports.into_boxed_slice().into(),
            functions: functions.into_boxed_slice().into(),
            classes: classes.into_boxed_slice().into(),
            stmts: stmts.into_boxed_slice().into(),
        }));
    }
}
