use std::sync::Arc;

use crate::{
    parse_tree::{
        expr::Expr, if_next, if_parse, is_next, require_next, require_parse, require_parse_fn,
        try_next, try_parse, ParserError,
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

        return Ok(None);
    }
}

impl TryStmt {
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
