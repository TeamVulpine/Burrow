use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{if_next, require_next, require_parse, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::{
    value::literal::{LiteralExpr, LiteralExprKind},
    Expr, ExprKind,
};

#[derive(Debug, PartialEq, Clone)]
pub struct AccessExpr {
    pub slice: StringSlice,
    pub base: Arc<Expr>,
    pub access: Arc<[AccessArm]>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AccessArm {
    pub slice: StringSlice,
    pub kind: AccessKind,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AccessKind {
    Ident(Arc<str>),
    Index(Arc<Expr>),
    Invoke(Arc<[Expr]>),
    Assign(Arc<Expr>),
    Prototype,
}

impl AccessExpr {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        if self.access.len() == 1
            && let AccessKind::Assign(assign) = &self.access[0].kind
            && let ExprKind::Literal(LiteralExpr {
                slice: _,
                kind: LiteralExprKind::Variable(name),
            }) = &self.base.kind
        {
            assign.generate_bytecode(bytecode)?;

            bytecode.push(OpCode::SetSlice {
                slice: self.slice.clone(),
            });
            bytecode.push(OpCode::StoreVariable { name: name.clone() });

            return Ok(());
        }

        self.base.generate_bytecode(bytecode)?;

        let mut idx = 0;

        while idx < self.access.len() {
            let value = &self.access[idx];
            bytecode.push(OpCode::SetSlice {
                slice: value.slice.clone(),
            });

            match &value.kind {
                AccessKind::Assign(_) => {
                    return Err(BytecodeGenerationError::IllegalAssignment(
                        value.slice.clone(),
                    ))
                }

                AccessKind::Ident(name) => {
                    if idx < self.access.len() - 1
                        && let AccessKind::Assign(assignment) = &self.access[idx + 1].kind
                    {
                        bytecode.push(OpCode::PushConstString {
                            value: name.clone(),
                        });

                        assignment.generate_bytecode(bytecode)?;

                        bytecode.push(OpCode::SetSlice {
                            slice: value.slice.clone(),
                        });

                        bytecode.push(OpCode::StoreIndex);

                        // When parsing, index is always the last one
                        break;
                    }

                    if idx < self.access.len() - 1
                        && let AccessKind::Invoke(invocation) = &self.access[idx + 1].kind
                    {
                        bytecode.push(OpCode::Dupe);

                        bytecode.push(OpCode::PushConstString {
                            value: name.clone(),
                        });
                        bytecode.push(OpCode::PushIndex);

                        for value in invocation.iter() {
                            value.generate_bytecode(bytecode)?;
                        }
                        bytecode.push(OpCode::SetSlice {
                            slice: self.access[idx + 1].slice.clone(),
                        });

                        bytecode.push(OpCode::Invoke {
                            param_count: invocation.len(),
                            this_call: true,
                        });

                        idx += 2;
                        continue;
                    }

                    bytecode.push(OpCode::PushConstString {
                        value: name.clone(),
                    });
                    bytecode.push(OpCode::PushIndex);
                }

                AccessKind::Index(index) => {
                    if idx < self.access.len() - 1
                        && let AccessKind::Assign(assignment) = &self.access[idx + 1].kind
                    {
                        index.generate_bytecode(bytecode)?;

                        assignment.generate_bytecode(bytecode)?;

                        bytecode.push(OpCode::SetSlice {
                            slice: value.slice.clone(),
                        });

                        bytecode.push(OpCode::StoreIndex);

                        // When parsing, index is always the last one
                        break;
                    }

                    if idx < self.access.len() - 1
                        && let AccessKind::Invoke(invocation) = &self.access[idx + 1].kind
                    {
                        bytecode.push(OpCode::Dupe);

                        index.generate_bytecode(bytecode)?;

                        bytecode.push(OpCode::SetSlice {
                            slice: value.slice.clone(),
                        });

                        bytecode.push(OpCode::PushIndex);

                        for value in invocation.iter() {
                            value.generate_bytecode(bytecode)?;
                        }
                        bytecode.push(OpCode::SetSlice {
                            slice: self.access[idx + 1].slice.clone(),
                        });

                        bytecode.push(OpCode::Invoke {
                            param_count: invocation.len(),
                            this_call: true,
                        });

                        idx += 2;
                        continue;
                    }

                    index.generate_bytecode(bytecode)?;
                    bytecode.push(OpCode::SetSlice {
                        slice: value.slice.clone(),
                    });

                    bytecode.push(OpCode::PushIndex);
                }

                AccessKind::Invoke(invocation) => {
                    for value in invocation.iter() {
                        value.generate_bytecode(bytecode)?;
                    }
                    bytecode.push(OpCode::SetSlice {
                        slice: value.slice.clone(),
                    });

                    bytecode.push(OpCode::Invoke {
                        param_count: invocation.len(),
                        this_call: false,
                    });
                }

                AccessKind::Prototype => {
                    if idx < self.access.len() - 1
                        && let AccessKind::Assign(assignment) = &self.access[idx + 1].kind
                    {
                        assignment.generate_bytecode(bytecode)?;
                        bytecode.push(OpCode::SetSlice {
                            slice: value.slice.clone(),
                        });

                        bytecode.push(OpCode::StoreProtorype);

                        // When parsing, index is always the last one
                        break;
                    }

                    if idx < self.access.len() - 1
                        && let AccessKind::Invoke(invocation) = &self.access[idx + 1].kind
                    {
                        bytecode.push(OpCode::Dupe);

                        bytecode.push(OpCode::PushPrototype);
                        bytecode.push(OpCode::PushIndex);

                        for value in invocation.iter() {
                            value.generate_bytecode(bytecode)?;
                        }
                        bytecode.push(OpCode::SetSlice {
                            slice: self.access[idx + 1].slice.clone(),
                        });

                        bytecode.push(OpCode::Invoke {
                            param_count: invocation.len(),
                            this_call: true,
                        });

                        idx += 2;
                        continue;
                    }

                    bytecode.push(OpCode::PushPrototype);
                    bytecode.push(OpCode::PushIndex);
                }
            }

            idx += 1;
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer, base: &Expr) -> Result<Option<Self>, ParserError> {
        let mut access = vec![];
        let mut end = base.slice.clone();
        loop {
            let start = tokenizer.peek(0)?.slice;

            if_next!(TokenKind::Symbol(Symbol::Dot), tokenizer, {
                end = tokenizer.peek(0)?.slice;
                if_next!(TokenKind::Keyword(Keyword::Prototype), tokenizer, {
                    access.push(AccessArm {
                        slice: start.merge(&end),
                        kind: AccessKind::Prototype,
                    });

                    continue;
                });

                require_next!(TokenKind::Identifier(ident), tokenizer);

                access.push(AccessArm {
                    slice: start.merge(&end),
                    kind: AccessKind::Ident(ident),
                });

                continue;
            });

            if_next!(TokenKind::Symbol(Symbol::BracketOpen), tokenizer, {
                require_parse!(expr, Expr, tokenizer);

                end = tokenizer.peek(0)?.slice;
                require_next!(TokenKind::Symbol(Symbol::BracketClose), tokenizer);

                access.push(AccessArm {
                    slice: start.merge(&end),
                    kind: AccessKind::Index(Arc::new(expr)),
                });
                continue;
            });

            if_next!(TokenKind::Symbol(Symbol::ParenOpen), tokenizer, {
                let mut values = vec![];

                end = loop {
                    let end = tokenizer.peek(0)?.slice;
                    if_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
                        break end;
                    });

                    require_parse!(value, Expr, tokenizer);
                    values.push(value);

                    let end = tokenizer.peek(0)?.slice;
                    if_next!(TokenKind::Symbol(Symbol::ParenClose), tokenizer, {
                        break end;
                    });

                    require_next!(TokenKind::Symbol(Symbol::Comma), tokenizer);
                };

                access.push(AccessArm {
                    slice: start.merge(&end),
                    kind: AccessKind::Invoke(values.into_boxed_slice().into()),
                });

                continue;
            });

            if_next!(TokenKind::Symbol(Symbol::Assign), tokenizer, {
                require_parse!(expr, Expr, tokenizer);

                end = expr.slice.clone();

                access.push(AccessArm {
                    slice: start.merge(&end),
                    kind: AccessKind::Assign(Arc::new(expr)),
                });

                break;
            });

            break;
        }

        if access.len() == 0 {
            return Ok(None);
        }

        return Ok(Some(Self {
            slice: base.slice.merge(&end),
            base: Arc::new(base.clone()),
            access: access.into_boxed_slice().into(),
        }));
    }
}
