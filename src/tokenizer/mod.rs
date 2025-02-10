use std::{collections::VecDeque, sync::Arc};

use token::{Keyword, Keywords, Number, Symbol, Token, TokenKind};

use crate::string::{parser::StringParser, StringSlice};

pub mod token;

fn valid_ident_start(c: char) -> bool {
    return c.is_alphabetic() || c == '_';
}

fn valid_ident_cont(c: char) -> bool {
    return c.is_alphanumeric() || c == '_';
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenizeError {
    InvalidString(StringSlice),
    InvalidChar(StringSlice),
    InvalidEscape(StringSlice),
    UnclosedStr(StringSlice),
    UnexpectedEof,
}

pub struct Tokenizer {
    parser: StringParser,
    peek: VecDeque<Token>,
}

impl Tokenizer {
    pub fn new(src: Arc<str>) -> Self {
        return Self {
            parser: StringParser::new(src),
            peek: VecDeque::new(),
        };
    }

    fn try_parse_ident(&mut self) -> Option<StringSlice> {
        if self.parser.is_func(valid_ident_start) {
            return self.parser.while_func(valid_ident_cont);
        }
        return None;
    }

    fn try_parse_number(&mut self) -> Option<(StringSlice, Number)> {
        self.parser.checkout();

        if let Some(whole_slice) = self.parser.while_func(char::is_numeric) {
            let whole: isize = whole_slice.value().parse().unwrap();

            if self.parser.is_char('.') {
                self.parser.checkout();
                self.parser.next();

                if self.parser.while_func(char::is_numeric).is_some() {
                    let decimal: f32 = self.parser.commit()?.value().parse().unwrap();
                    return Some((
                        self.parser.commit()?,
                        Number::Floating(whole as f32 + decimal),
                    ));
                }
                self.parser.rollback();
            }

            return Some((self.parser.commit()?, Number::Integer(whole)));
        }

        self.parser.rollback();

        return None;
    }

    fn try_parse_string(&mut self) -> Result<Option<(StringSlice, String)>, TokenizeError> {
        if !self.parser.is_char('"') {
            return Ok(None);
        }

        self.parser.checkout();

        self.parser.next();

        let mut str = "".to_string();

        while let Some(c) = self.parser.curr() {
            match c {
                'a'..='z'
                | 'A'..='Z'
                | '0'..='9'
                | ' '
                | '!'
                | '#'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '*'
                | '+'
                | ','
                | '-'
                | '.'
                | '/'
                | ':'
                | ';'
                | '<'
                | '>'
                | '='
                | '?'
                | '^'
                | '_'
                | '|'
                | '~' => {
                    str.push(c);
                    self.parser.next();
                }
                '\\' => {
                    let Some(c) = self.parser.next() else {
                        return Err(TokenizeError::UnexpectedEof);
                    };
                    let val = match c {
                        'n' => '\n',
                        'r' => '\r',
                        '\\' => '\\',
                        't' => '\t',
                        '"' => '"',
                        '\'' => '\'',
                        _ => {
                            let Some(s) = self.parser.commit() else {
                                return Err(TokenizeError::UnexpectedEof);
                            };

                            return Err(TokenizeError::InvalidEscape(s));
                        }
                    };

                    str.push(val);
                    self.parser.next();
                }
                '"' => {
                    self.parser.next();
                    let Some(s) = self.parser.commit() else {
                        return Err(TokenizeError::UnexpectedEof);
                    };

                    return Ok(Some((s, str)));
                }
                '\n' => {
                    let Some(s) = self.parser.commit() else {
                        return Err(TokenizeError::UnexpectedEof);
                    };
                    return Err(TokenizeError::UnclosedStr(s));
                }
                _ => {
                    let Some(s) = self.parser.commit() else {
                        return Err(TokenizeError::UnexpectedEof);
                    };
                    return Err(TokenizeError::InvalidString(s));
                }
            }
        }

        self.parser.rollback();

        return Err(TokenizeError::UnexpectedEof);
    }

    fn skip_ignores(&mut self) -> Result<(), TokenizeError> {
        loop {
            let mut exit = true;
            if self.parser.while_func(char::is_whitespace).is_some() {
                exit = false;
            }

            if self.parser.try_consume_str("#").is_some() {
                self.parser.while_func(|it| it != '\n');
                exit = false;
            }

            if self.parser.curr().is_none() || exit {
                break;
            }
        }
        return Ok(());
    }

    pub fn peek(&mut self, n: usize) -> Result<Token, TokenizeError> {
        if let Some(token) = self.peek.get(n) {
            return Ok(token.clone());
        }

        while n >= self.peek.len() {
            let next = self.next_raw()?;
            self.peek.push_back(next);
        }

        return Ok(self.peek.get(n).unwrap().clone());
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Result<Token, TokenizeError> {
        if let Some(peek) = self.peek.pop_front() {
            return Ok(peek);
        }

        return self.next_raw();
    }

    fn next_raw(&mut self) -> Result<Token, TokenizeError> {
        self.skip_ignores()?;

        if self.parser.curr().is_none() {
            self.parser.checkout();
            return Ok(Token {
                slice: self.parser.commit().unwrap(),
                kind: TokenKind::Eof,
            });
        }

        if let Some(slice) = self.try_parse_ident() {
            let value = slice.value();

            if let Some(keyword) = Keyword::parse(&value) {
                return Ok(Token {
                    slice,
                    kind: TokenKind::Keyword(keyword),
                });
            }

            return Ok(Token {
                slice,
                kind: TokenKind::Identifier(value),
            });
        }

        if let Some((slice, symbol)) = Symbol::from(&mut self.parser) {
            return Ok(Token {
                slice,
                kind: TokenKind::Symbol(symbol),
            });
        }

        if let Some((slice, number)) = self.try_parse_number() {
            return Ok(Token {
                slice,
                kind: TokenKind::Number(number),
            });
        }

        if let Some((slice, str)) = self.try_parse_string()? {
            return Ok(Token {
                slice,
                kind: TokenKind::String(str.into()),
            });
        }

        return Err(TokenizeError::InvalidChar(self.parser.commit().unwrap()));
    }
}
