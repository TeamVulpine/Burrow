use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError}, parse_tree::decl::function::FunctionImpl, string::StringSlice, tokenizer::{token::TokenKind, Tokenizer}
};

use super::{
    decl::{class::ClassDecl, import::{DirectImport, FromImport, FromImportKind, ImportDecl, ImportKind}, IdeDecl},
    require_next,
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
    pub fn generate_init_bytecode(&self, bytecode: &mut Vec<OpCode>) -> Result<(), BytecodeGenerationError> {
        for import in self.imports.iter() {
            bytecode.push(OpCode::SetSlice { slice: import.slice.clone() });

            match &import.kind {
                ImportKind::Direct(DirectImport {
                    slice: _,
                    file
                }) => {
                    bytecode.push(OpCode::Import { path: file.clone() });
                    bytecode.push(OpCode::Pop);
                }
                ImportKind::From(FromImport {
                    slice: _,
                    file,
                    values
                }) => {
                    bytecode.push(OpCode::Import { path: file.clone() });
                    for value  in values.iter() {
                        match &value.kind {
                            FromImportKind::Everything { name } => {
                                bytecode.push(OpCode::Dupe);
                                bytecode.push(OpCode::InitVariable { name: name.clone() });
                                bytecode.push(OpCode::StoreVariable { name: name.clone() });
                                bytecode.push(OpCode::MarkVariableConst { name: name.clone() });
                            },
                            FromImportKind::Single { name, rename } => {
                                bytecode.push(OpCode::Dupe);
                                let value_name = if let Some(rename) = rename {
                                    rename
                                } else {
                                    name
                                };
                                
                                bytecode.push(OpCode::InitVariable { name: value_name.clone() });
                                bytecode.push(OpCode::PushConstString { value: name.clone() });
                                bytecode.push(OpCode::PushIndex);
                                bytecode.push(OpCode::StoreVariable { name: value_name.clone() });
                                bytecode.push(OpCode::MarkVariableConst { name: value_name.clone() });
                            }
                        }
                    }
                    bytecode.push(OpCode::Pop);
                }
            }
        }

        for class in self.classes.iter() {
            bytecode.push(OpCode::SetSlice { slice: class.slice.clone() });

            bytecode.push(OpCode::InitVariable { name: class.name.clone() });
            bytecode.push(OpCode::PushNewObject);
            if let Some(extends) = &class.extends {
                bytecode.push(OpCode::Dupe);
                bytecode.push(OpCode::PushVariable { name: extends.clone() });
                bytecode.push(OpCode::StoreProtorype);
            }
            bytecode.push(OpCode::StoreVariable { name: class.name.clone() });
            bytecode.push(OpCode::MarkVariableConst { name: class.name.clone() });

            if class.export {
                bytecode.push(OpCode::Export { name: class.name.clone() });
            }
        }

        for i in 0..self.functions.len() {
            let func = &self.functions[i];

            bytecode.push(OpCode::SetSlice { slice: func.slice.clone() });

            if let Some(base) = &func.decl.base {
                bytecode.push(OpCode::PushVariable { name: base.clone() });
                bytecode.push(OpCode::PushConstString { value: func.decl.name.clone() });
                bytecode.push(OpCode::PushFunction { index: i });
                bytecode.push(OpCode::StoreIndex);
                continue;
            }

            bytecode.push(OpCode::InitVariable { name: func.decl.name.clone() });
            bytecode.push(OpCode::PushFunction { index: i });
            bytecode.push(OpCode::StoreVariable { name: func.decl.name.clone() });
            bytecode.push(OpCode::MarkVariableConst { name: func.decl.name.clone() });

            if func.export {
                bytecode.push(OpCode::Export { name: func.decl.name.clone()  });
            }
        }

        for stmt in self.stmts.iter() {
            stmt.generate_bytecode(bytecode, true, false)?;
        }

        return Ok(());
    }

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

        require_next!(TokenKind::Eof, tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            imports: imports.into_boxed_slice().into(),
            functions: functions.into_boxed_slice().into(),
            classes: classes.into_boxed_slice().into(),
            stmts: stmts.into_boxed_slice().into(),
        }));
    }
}
