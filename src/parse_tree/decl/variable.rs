use crate::{
    bytecode::{op_code::OpCode, BytecodeGenerationError},
    parse_tree::{
        expr::Expr, if_next, if_next_or_none, if_parse_fn, peek_nth, require_parse, try_parse,
        try_parse_fn, ParserError,
    },
    string::StringSlice,
    tokenizer::{
        token::{Keyword, Symbol, TokenKind},
        Tokenizer,
    },
};

use super::VariableName;

#[derive(Debug, Clone, PartialEq)]
pub struct VariableDecl {
    pub slice: StringSlice,
    pub export: bool,
    pub is_const: bool,
    pub param: VariableName,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VariableImpl {
    pub slice: StringSlice,
    pub decl: VariableDecl,
    pub init: Option<Expr>,
}

impl VariableDecl {
    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        if_parse_fn!(var, Self::try_parse_let, tokenizer, {
            return Ok(Some(var));
        });

        if_parse_fn!(var, Self::try_parse_const, tokenizer, {
            return Ok(Some(var));
        });

        return Ok(None);
    }

    fn parse_keyword_let(tokenizer: &mut Tokenizer) -> Result<Option<bool>, ParserError> {
        if_next!(TokenKind::Keyword(Keyword::Let), tokenizer, {
            return Ok(Some(false));
        });

        peek_nth!(TokenKind::Keyword(Keyword::Export), 0, tokenizer);
        peek_nth!(TokenKind::Keyword(Keyword::Let), 1, tokenizer);

        tokenizer.next()?;
        tokenizer.next()?;

        return Ok(Some(true));
    }

    fn parse_keyword_const(tokenizer: &mut Tokenizer) -> Result<Option<bool>, ParserError> {
        if_next!(TokenKind::Keyword(Keyword::Const), tokenizer, {
            return Ok(Some(false));
        });

        peek_nth!(TokenKind::Keyword(Keyword::Export), 0, tokenizer);
        peek_nth!(TokenKind::Keyword(Keyword::Const), 1, tokenizer);

        tokenizer.next()?;
        tokenizer.next()?;

        return Ok(Some(true));
    }

    pub fn try_parse_let(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_parse_fn!(export, Self::parse_keyword_let, tokenizer);

        require_parse!(param, VariableName, tokenizer);

        let end = param.slice.clone();

        return Ok(Some(Self {
            slice: start.merge(&end),
            export,
            is_const: false,
            param,
        }));
    }

    pub fn try_parse_const(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        let start = tokenizer.peek(0)?.slice;
        try_parse_fn!(export, Self::parse_keyword_const, tokenizer);

        require_parse!(param, VariableName, tokenizer);

        let end = param.slice.clone();

        return Ok(Some(Self {
            slice: start.merge(&end),
            export,
            is_const: false,
            param,
        }));
    }
}

impl VariableImpl {
    pub fn generate_bytecode(
        &self,
        bytecode: &mut Vec<OpCode>,
        allow_export: bool,
    ) -> Result<(), BytecodeGenerationError> {
        if !allow_export && self.decl.export {
            return Err(BytecodeGenerationError::IllegalExport(self.slice.clone()));
        }

        let name = self.decl.param.name.clone();

        bytecode.push(OpCode::InitVariable { name: name.clone() });

        if let Some(init) = &self.init {
            init.generate_bytecode(bytecode)?;

            bytecode.push(OpCode::StoreVariable { name: name.clone() });
        }

        if self.decl.is_const {
            bytecode.push(OpCode::MarkVariableConst { name: name.clone() });
        }

        if self.decl.export {
            bytecode.push(OpCode::Export { name: name.clone() });
        }

        return Ok(());
    }

    pub fn try_parse(tokenizer: &mut Tokenizer) -> Result<Option<Self>, ParserError> {
        try_parse!(decl, VariableDecl, tokenizer);

        let init: Option<Expr> = if_next_or_none!(TokenKind::Symbol(Symbol::Assign), tokenizer, {
            require_parse!(expr, Expr, tokenizer);

            Some(expr)
        });

        let end = init
            .clone()
            .map(|it| it.slice)
            .unwrap_or(decl.slice.clone());

        return Ok(Some(Self {
            slice: decl.slice.merge(&end),
            decl,
            init,
        }));
    }
}
