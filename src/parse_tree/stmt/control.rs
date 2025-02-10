use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{
        expr::Expr, if_next, if_parse, is_next, peek_nth, require_next, require_parse,
        require_parse_fn, try_next, try_parse, ParserError,
    },
    string::StringSlice,
    tokenizer::{
        token::{Keyword, TokenKind},
        Tokenizer,
    },
};

use super::Block;

#[derive(Debug, Clone, PartialEq)]
pub struct ControlStmt {
    pub slice: StringSlice,
    pub kind: ControlKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ControlKind {
    While(WhileStmt),
    If(IfStmt),
    For(ForStmt),
    Try(TryStmt),
    Throw(Expr),
    Return(Option<Expr>),
    Export(Arc<str>),
    Continue,
    Break,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TryStmt {
    pub slice: StringSlice,
    pub try_block: Block,
    pub catch_name: Arc<str>,
    pub catch_block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WhileStmt {
    pub slice: StringSlice,
    pub until: bool,
    pub arm: ConditionArm,
}

#[derive(Debug, Clone, PartialEq)]
pub struct IfStmt {
    pub slice: StringSlice,
    pub arms: Arc<[ConditionArm]>,
    pub else_arm: Option<Block>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ForStmt {
    pub slice: StringSlice,
    pub name: Arc<str>,
    pub expr: Expr,
    pub block: Block,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ConditionArm {
    pub slice: StringSlice,
    pub condition: Expr,
    pub block: Block,
}

impl ControlStmt {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
        allow_export: bool,
        allow_break_continue: bool,
    ) -> Result<(), BytecodeGenerationError> {
        bytecode.push(OpCode::SetSlice {
            slice: self.slice.clone(),
        });
        match &self.kind {
            ControlKind::Break => {
                if !allow_break_continue {
                    return Err(BytecodeGenerationError::IllegalBreak(self.slice.clone()));
                }
                bytecode.push(OpCode::TempBreak);
            }
            ControlKind::Continue => {
                if !allow_break_continue {
                    return Err(BytecodeGenerationError::IllegalContinue(self.slice.clone()));
                }
                bytecode.push(OpCode::TempContinue);
            }
            ControlKind::Export(name) => {
                if !allow_export {
                    return Err(BytecodeGenerationError::IllegalExport(self.slice.clone()));
                }
                bytecode.push(OpCode::Export { name: name.clone() });
            }
            ControlKind::Return(value) => {
                if let Some(value) = value {
                    value.generate_bytecode(bytecode)?;
                    bytecode.push(OpCode::SetSlice {
                        slice: self.slice.clone(),
                    });
                } else {
                    bytecode.push(OpCode::PushConstNone);
                }
                bytecode.push(OpCode::Return);
            }
            ControlKind::Throw(value) => {
                value.generate_bytecode(bytecode)?;
                bytecode.push(OpCode::SetSlice {
                    slice: self.slice.clone(),
                });
                bytecode.push(OpCode::Throw);
            }

            ControlKind::For(stmt) => stmt.generate_bytecode(bytecode)?,

            ControlKind::If(stmt) => stmt.generate_bytecode(bytecode)?,

            ControlKind::Try(stmt) => stmt.generate_bytecode(bytecode)?,

            ControlKind::While(stmt) => stmt.generate_bytecode(bytecode)?,
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse!(stmt, WhileStmt, tokenizer, {
            return Ok(Some(Self {
                slice: stmt.slice.clone(),
                kind: ControlKind::While(stmt),
            }));
        });

        if_parse!(stmt, IfStmt, tokenizer, {
            return Ok(Some(Self {
                slice: stmt.slice.clone(),
                kind: ControlKind::If(stmt),
            }));
        });

        if_parse!(stmt, ForStmt, tokenizer, {
            return Ok(Some(Self {
                slice: stmt.slice.clone(),
                kind: ControlKind::For(stmt),
            }));
        });

        if_parse!(stmt, TryStmt, tokenizer, {
            return Ok(Some(Self {
                slice: stmt.slice.clone(),
                kind: ControlKind::Try(stmt),
            }));
        });

        let start = tokenizer.peek(0)?.slice;
        if_next!(TokenKind::Keyword(Keyword::Throw), tokenizer, {
            require_parse!(expr, Expr, tokenizer);

            return Ok(Some(Self {
                slice: start.merge(&expr.slice),
                kind: ControlKind::Throw(expr),
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::Return), tokenizer, {
            if_parse!(expr, Expr, tokenizer, {
                return Ok(Some(Self {
                    slice: start.merge(&expr.slice),
                    kind: ControlKind::Return(Some(expr)),
                }));
            });
            return Ok(Some(Self {
                slice: start,
                kind: ControlKind::Return(None),
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::Continue), tokenizer, {
            return Ok(Some(Self {
                slice: start,
                kind: ControlKind::Continue,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::Break), tokenizer, {
            return Ok(Some(Self {
                slice: start,
                kind: ControlKind::Break,
            }));
        });

        peek_nth!(TokenKind::Keyword(Keyword::Export), 0, tokenizer);
        peek_nth!(TokenKind::Identifier(ident), 1, tokenizer);

        tokenizer.next()?;
        let end = tokenizer.next()?.slice;

        return Ok(Some(Self {
            slice: start.merge(&end),
            kind: ControlKind::Export(ident),
        }));
    }
}

impl TryStmt {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        let catch_update_index = bytecode.len();

        bytecode.push(OpCode::PushCatch { location: 0 });

        self.try_block.generate_bytecode(bytecode, false, false)?;

        bytecode.push(OpCode::PopCatch);

        let jump_update_index = bytecode.len();

        bytecode.push(OpCode::Jump { location: 0 });

        bytecode[catch_update_index] = OpCode::PushCatch {
            location: bytecode.len(),
        };

        bytecode.push(OpCode::InitVariable {
            name: self.catch_name.clone(),
        });
        bytecode.push(OpCode::PushException);
        bytecode.push(OpCode::StoreVariable {
            name: self.catch_name.clone(),
        });
        bytecode.push(OpCode::Pop);
        bytecode.push(OpCode::MarkVariableConst {
            name: self.catch_name.clone(),
        });

        self.catch_block.generate_bytecode(bytecode, false, false)?;

        bytecode[jump_update_index] = OpCode::Jump {
            location: bytecode.len(),
        };

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::Try), tokenizer);

        require_parse!(try_block, Block, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Catch), tokenizer);

        require_next!(TokenKind::Identifier(catch_name), tokenizer);

        require_parse!(catch_block, Block, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&end),
            try_block,
            catch_name,
            catch_block,
        }));
    }
}

impl WhileStmt {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        let condition_index = bytecode.len();
        self.arm.condition.generate_bytecode(bytecode)?;

        let jump_update_index = bytecode.len();
        bytecode.push(OpCode::JumpTrue { location: 0 });

        self.arm.block.generate_bytecode(bytecode, false, true)?;

        let exit_index = bytecode.len();

        for i in (jump_update_index + 1)..exit_index {
            if let OpCode::TempBreak = bytecode[i] {
                bytecode[i] = OpCode::Jump {
                    location: exit_index,
                };
                continue;
            }

            if let OpCode::TempContinue = bytecode[i] {
                bytecode[i] = OpCode::Jump {
                    location: condition_index,
                };
                continue;
            }
        }

        if self.until {
            bytecode[jump_update_index] = OpCode::JumpTrue {
                location: exit_index,
            };
        } else {
            bytecode[jump_update_index] = OpCode::JumpFalse {
                location: exit_index,
            };
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;

        if_next!(TokenKind::Keyword(Keyword::While), tokenizer, {
            require_parse_fn!(arm, ConditionArm::try_parse_do, tokenizer);

            return Ok(Some(Self {
                slice: start.merge(&arm.slice),
                until: false,
                arm,
            }));
        });

        if_next!(TokenKind::Keyword(Keyword::Until), tokenizer, {
            require_parse_fn!(arm, ConditionArm::try_parse_do, tokenizer);

            return Ok(Some(Self {
                slice: start.merge(&arm.slice),
                until: true,
                arm,
            }));
        });

        return Ok(None);
    }
}

impl IfStmt {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        let mut jump_update_indices: Vec<usize> = vec![];

        for arm in self.arms.iter() {
            arm.condition.generate_bytecode(bytecode)?;
            jump_update_indices.push(bytecode.len());
            bytecode.push(OpCode::JumpTrue { location: 0 });
        }

        let else_index = bytecode.len();
        bytecode.push(OpCode::Jump { location: 0 });

        let mut block_indices: Vec<usize> = vec![];

        let mut exit_indices: Vec<usize> = vec![];

        for arm in self.arms.iter() {
            block_indices.push(bytecode.len());
            arm.block.generate_bytecode(bytecode, false, false)?;
            exit_indices.push(bytecode.len());
            bytecode.push(OpCode::Jump { location: 0 });
        }

        for i in 0..self.arms.len() {
            bytecode[jump_update_indices[i]] = OpCode::JumpTrue {
                location: block_indices[i],
            };
        }

        bytecode[else_index] = OpCode::Jump {
            location: bytecode.len(),
        };

        if let Some(else_arm) = &self.else_arm {
            else_arm.generate_bytecode(bytecode, false, false)?;
        }

        let exit_index = bytecode.len();
        for i in exit_indices {
            bytecode[i] = OpCode::Jump {
                location: exit_index,
            };
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::If), tokenizer);

        require_parse_fn!((arm, mut is_else), ConditionArm::try_parse_else, tokenizer);

        let mut arms = vec![arm];

        while is_else {
            if_next!(TokenKind::Keyword(Keyword::If), tokenizer, {
                require_parse_fn!((arm, next_is_else), ConditionArm::try_parse_else, tokenizer);
                is_else = next_is_else;
                arms.push(arm);
                continue;
            });

            require_parse!(block, Block, tokenizer);

            let end = tokenizer.peek(0)?.slice;
            require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

            return Ok(Some(Self {
                slice: start.merge(&end),
                arms: arms.into_boxed_slice().into(),
                else_arm: Some(block),
            }));
        }

        return Ok(Some(Self {
            slice: start.merge(&arms.last().unwrap().slice),
            arms: arms.into_boxed_slice().into(),
            else_arm: None,
        }));
    }
}

impl ForStmt {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        let value_name: Arc<str> = format!("__each_{}_value__", self.name).into();
        let index_name: Arc<str> = format!("__each_{}_index__", self.name).into();

        bytecode.push(OpCode::InitVariable {
            name: value_name.clone(),
        });
        self.expr.generate_bytecode(bytecode)?;
        bytecode.push(OpCode::StoreVariable {
            name: value_name.clone(),
        });
        bytecode.push(OpCode::Pop);

        bytecode.push(OpCode::InitVariable {
            name: index_name.clone(),
        });
        bytecode.push(OpCode::PushConstInt { value: 0 });
        bytecode.push(OpCode::StoreVariable {
            name: index_name.clone(),
        });
        bytecode.push(OpCode::Pop);

        bytecode.push(OpCode::Jump {
            location: bytecode.len() + 6,
        });

        let increment_pos = bytecode.len();

        bytecode.push(OpCode::PushVariable {
            name: index_name.clone(),
        });
        bytecode.push(OpCode::PushConstInt { value: 1 });
        bytecode.push(OpCode::OpAdd);
        bytecode.push(OpCode::StoreVariable {
            name: index_name.clone(),
        });
        bytecode.push(OpCode::Pop);

        bytecode.push(OpCode::PushVariable {
            name: index_name.clone(),
        });
        bytecode.push(OpCode::PushVariable {
            name: value_name.clone(),
        });
        bytecode.push(OpCode::PushConstString {
            value: "length".into(),
        });
        bytecode.push(OpCode::PushIndex);
        bytecode.push(OpCode::OpLt);

        let update_exit_pos = bytecode.len();
        bytecode.push(OpCode::Jump { location: 0 });

        self.block.generate_bytecode(bytecode, false, true)?;

        let exit_pos = bytecode.len();

        for pos in (update_exit_pos + 1)..exit_pos {
            let value = &mut bytecode[pos];

            if let OpCode::TempBreak = value {
                *value = OpCode::Jump { location: exit_pos };
                continue;
            }

            if let OpCode::TempContinue = value {
                *value = OpCode::Jump {
                    location: increment_pos,
                };
                continue;
            }
        }

        bytecode[update_exit_pos] = OpCode::Jump { location: exit_pos };

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_next!(TokenKind::Keyword(Keyword::For), tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Each), tokenizer);

        require_next!(TokenKind::Identifier(name), tokenizer);

        require_next!(TokenKind::Keyword(Keyword::In), tokenizer);

        require_parse!(expr, Expr, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Do), tokenizer);

        require_parse!(block, Block, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: start.merge(&block.slice),
            name,
            expr,
            block,
        }));
    }
}

impl ConditionArm {
    pub fn try_parse_do(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse!(condition, Expr, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Do), tokenizer);

        require_parse!(block, Block, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: condition.slice.merge(&end),
            condition,
            block,
        }));
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse!(condition, Expr, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Then), tokenizer);

        require_parse!(block, Block, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        require_next!(TokenKind::Keyword(Keyword::End), tokenizer);

        return Ok(Some(Self {
            slice: condition.slice.merge(&end),
            condition,
            block,
        }));
    }

    pub fn try_parse_else(tokenizer: &mut Tokenizer) -> Result<Option<(Self, bool)>, ParserError> {
        try_parse!(condition, Expr, tokenizer);

        require_next!(TokenKind::Keyword(Keyword::Then), tokenizer);

        require_parse!(block, Block, tokenizer);

        let end = tokenizer.peek(0)?.slice;
        let is_else = is_next!(TokenKind::Keyword(Keyword::Else), tokenizer);
        if !is_else {
            require_next!(TokenKind::Keyword(Keyword::End), tokenizer);
        }

        return Ok(Some((
            Self {
                slice: condition.slice.merge(&end),
                condition,
                block,
            },
            is_else,
        )));
    }
}
