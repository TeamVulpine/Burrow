use std::sync::Arc;

use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{expr::Expr, ParserError},
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

#[derive(Debug, PartialEq, Clone)]
pub struct BinOpExpr {
    pub slice: StringSlice,
    pub lhs: Arc<Expr>,
    pub op: BinOpKind,
    pub rhs: Arc<Expr>,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum BinOpKind {
    Add,
    Sub,

    Mul,
    Div,
    Rem,

    Greater,
    Less,
    GreaterEqual,
    LessEqual,

    Equal,
    NotEqual,

    Is,
    IsNot,

    And,

    Or,
}

impl BinOpExpr {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
    ) -> Result<(), BytecodeGenerationError> {
        self.lhs.generate_bytecode(bytecode)?;
        self.rhs.generate_bytecode(bytecode)?;

        bytecode.push(OpCode::SetSlice {
            slice: self.slice.clone(),
        });

        match self.op {
            BinOpKind::Add => bytecode.push(OpCode::OpAdd),
            BinOpKind::Sub => bytecode.push(OpCode::OpSub),

            BinOpKind::Mul => bytecode.push(OpCode::OpAdd),
            BinOpKind::Div => bytecode.push(OpCode::OpAdd),
            BinOpKind::Rem => bytecode.push(OpCode::OpAdd),

            BinOpKind::Greater => bytecode.push(OpCode::OpGt),
            BinOpKind::Less => bytecode.push(OpCode::OpLt),
            BinOpKind::GreaterEqual => bytecode.push(OpCode::OpGe),
            BinOpKind::LessEqual => bytecode.push(OpCode::OpLe),

            BinOpKind::Equal => bytecode.push(OpCode::OpEq),
            BinOpKind::NotEqual => bytecode.push(OpCode::OpNe),

            BinOpKind::Is => bytecode.push(OpCode::ProtoEq),
            BinOpKind::IsNot => bytecode.push(OpCode::ProtoNe),

            BinOpKind::And => bytecode.push(OpCode::OpAnd),
            BinOpKind::Or => bytecode.push(OpCode::OpOr),
        }

        return Ok(());
    }
}

impl BinOpKind {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let peek = tokenizer.peek(0)?;

        return Ok(Some(match peek.kind {
            TokenKind::Symbol(Symbol::Add) => Self::Add,
            TokenKind::Symbol(Symbol::Sub) => Self::Sub,

            TokenKind::Symbol(Symbol::Mul) => Self::Mul,
            TokenKind::Symbol(Symbol::Div) => Self::Div,
            TokenKind::Symbol(Symbol::Rem) => Self::Rem,

            TokenKind::Symbol(Symbol::Greater) => Self::Greater,
            TokenKind::Symbol(Symbol::Less) => Self::Less,
            TokenKind::Symbol(Symbol::GreaterEqual) => Self::GreaterEqual,
            TokenKind::Symbol(Symbol::LessEqual) => Self::LessEqual,

            TokenKind::Symbol(Symbol::Equal) => Self::Equal,
            TokenKind::Symbol(Symbol::NotEqual) => Self::NotEqual,

            TokenKind::Keyword(Keyword::Is) => {
                if let TokenKind::Keyword(Keyword::Not) = tokenizer.peek(1)?.kind {
                    Self::IsNot
                } else {
                    Self::Is
                }
            }

            TokenKind::Keyword(Keyword::And) => Self::And,
            TokenKind::Keyword(Keyword::Or) => Self::Or,

            _ => return Ok(None),
        }));
    }

    pub fn binding(self) -> (usize, usize) {
        return match self {
            Self::Or => (3, 4),

            Self::And => (5, 6),

            Self::Is | Self::IsNot => (7, 8),

            Self::Equal | Self::NotEqual => (9, 10),

            Self::Greater | Self::Less | Self::GreaterEqual | Self::LessEqual => (11, 12),

            Self::Mul | Self::Div | Self::Rem => (13, 14),

            Self::Add | Self::Sub => (15, 16),
        };
    }

    pub fn consume_amount(self) -> usize {
        return match self {
            Self::IsNot => 2,
            _ => 1,
        };
    }
}
