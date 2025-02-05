use crate::tokenizer::{token::Token, TokenizeError};

pub mod decl;
pub mod expr;
pub mod stmt;
pub mod tree;
pub mod ty;

pub macro require_next($p: pat, $tokenizer: expr) {
    let next = $tokenizer.next()?;
    let $p = next.kind else {
        return Err(ParserError::unexpected_token(next));
    };
}

pub macro if_next($p: pat, $tokenizer: expr, $tree: tt) {
    if let $p = $tokenizer.peek(0)?.kind {
        $tokenizer.next()?;
        $tree;
    }
}

pub macro next_else($p: pat, $tokenizer: expr, $tree: tt) {
    if let $p = $tokenizer.peek(0)?.kind {
        $tokenizer.next()?;
    } else {
        $tree;
    }
}

pub macro if_next_or_none($p: pat, $tokenizer: expr, $tree: tt) {
    if let $p = $tokenizer.peek(0)?.kind {
        $tokenizer.next()?;
        $tree
    } else {
        None
    }
}

pub macro allow_accidental($p: pat, $tokenizer: expr) {
    if let $p = $tokenizer.peek(0)?.kind {
        $tokenizer.next()?;
    }
}

pub macro is_next($p: pat, $tokenizer: expr) {
    if let $p = $tokenizer.peek(0)?.kind {
        $tokenizer.next()?;
        true
    } else {
        false
    }
}

pub macro try_next($p: pat, $tokenizer: expr) {
    let $p = $tokenizer.peek(0)?.kind else {
        return Ok(None);
    };
    $tokenizer.next()?;
}

pub macro while_next($p: pat, $n: pat, $tokenizer: expr, $tree: tt) {
    while let $p = $tokenizer.peek(0)?.kind {
        let $n = $tokenizer.next()?;
        $tree
    }
}

pub macro peek_nth($p: pat, $n: expr, $tokenizer: expr) {
    let peek = $tokenizer.peek($n)?;
    let $p = peek.kind else {
        return Ok(None);
    };
}

pub macro peek_not_nth($p: pat, $n: expr, $tokenizer: expr) {
    let peek = $tokenizer.peek($n)?;
    if $p = peek.kind {
        return Ok(None);
    };
}

pub macro parse_else_fn($name: pat, $f: expr, $tokenizer: expr, $el: tt) {
    let Some($name) = $f($tokenizer)? else { $el };
}

pub macro parse_else($name: pat, $ty: ty, $tokenizer: expr, $el: tt) {
    parse_else_fn!($name, <$ty>::try_parse, $tokenizer, $el);
}

pub macro try_parse_fn($name: pat, $f: expr, $tokenizer: expr) {
    parse_else_fn!($name, $f, $tokenizer, { return Ok(None) });
}

pub macro try_parse($name: pat, $ty: ty, $tokenizer: expr) {
    try_parse_fn!($name, <$ty>::try_parse, $tokenizer);
}

pub macro require_parse_fn($name: pat, $f: expr, $tokenizer: expr) {
    let peek = $tokenizer.peek(0)?;
    parse_else_fn!($name, $f, $tokenizer, {
        return Err(ParserError::unexpected_token(peek));
    });
}

pub macro require_parse($name: pat, $ty: ty, $tokenizer: expr) {
    require_parse_fn!($name, <$ty>::try_parse, $tokenizer);
}

pub macro if_parse_fn($name: pat, $f: expr, $tokenizer: expr, $block: tt) {
    if let Some($name) = $f($tokenizer)? $block
}

pub macro if_parse($name: pat, $ty: ty, $tokenizer: expr, $block: tt) {
    if_parse_fn!($name, <$ty>::try_parse, $tokenizer, $block);
}

pub macro if_parse_or_none($name: pat, $ty: ty, $tokenizer: expr, $block: tt) {
    if_parse_or_none_fn!($name, <$ty>::try_parse, $tokenizer, $block);
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParserError {
    TokenizeError(TokenizeError),
    UnexpectedToken {
        token: Token,
        throwing_location: String,
    },
}

impl ParserError {
    #[track_caller]
    pub fn unexpected_token(token: Token) -> Self {
        return Self::UnexpectedToken {
            token,
            throwing_location: format!("{}", std::panic::Location::caller()),
        };
    }
}

impl From<TokenizeError> for ParserError {
    fn from(value: TokenizeError) -> Self {
        Self::TokenizeError(value)
    }
}
