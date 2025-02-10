use std::sync::Arc;

use crate::string::{parser::StringParser, StringSlice};

pub trait Keywords: Sized {
    fn parse(s: &str) -> Option<Self>;
    fn stringify(self) -> String;
}

pub macro keywords($name: ident {
    $($variant: ident ($value: expr)),*$(,)?
}) {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum $name {
        $($variant),*
    }

    impl Keywords for $name {
        fn parse(s: & str) -> Option<Self> {
            return Some(match s {
                $($value => Self::$variant),*,
                _ => return None
            });
        }

        fn stringify(self) -> String {
            return match self {
                $(Self::$variant => $value),*
            }.to_string();
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub slice: StringSlice,
    pub kind: TokenKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    Identifier(Arc<str>),
    String(Arc<str>),
    Number(Number),
    Symbol(Symbol),
    Keyword(Keyword),
    Eof,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Number {
    Integer(i32),
    Floating(f32),
}

keywords!(Keyword {
    Declare("declare"),
    Module("module"),

    Let("let"),
    Const("const"),
    Function("function"),
    Class("class"),
    Extends("extends"),
    Where("where"),
    Prototype("prototype"),

    While("while"),
    Until("until"),

    For("for"),
    Each("each"),
    In("in"),

    If("if"),
    Else("else"),

    Do("do"),
    Then("then"),
    End("end"),

    This("this"),

    None("none"),

    True("true"),
    False("false"),

    Infinity("Infinity"),
    NaN("NaN"),

    From("from"),
    Import("import"),
    Export("export"),
    Everything("everything"),
    As("as"),

    Is("is"),
    Not("not"),
    And("and"),
    Or("or"),

    Return("return"),
    Break("break"),
    Continue("continue"),

    Throw("throw"),
    Try("try"),
    Catch("catch"),
});

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Symbol {
    // Parenthesis and co.
    BracketOpen,  // [
    BracketClose, // ]
    BraceOpen,    // {
    BraceClose,   // }
    ParenOpen,    // (
    ParenClose,   // )

    // Math operators
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Rem, // %

    // Comparisons
    Greater,      // >
    Less,         // <
    GreaterEqual, // >=
    LessEqual,    // <=
    Equal,        // ==
    NotEqual,     // !=

    // Other symbols
    Colon,     // :
    Assign,    // =
    Comma,     // ,
    Dot,       // .
    Semicolon, // ;
}

impl Symbol {
    pub fn from(parser: &mut StringParser) -> Option<(StringSlice, Self)> {
        symbol_match!(parser,
            ":" => Self::Colon,
            "." => Self::Dot,
            "," => Self::Comma,
            ";" => Self::Semicolon,

            ">=" => Self::GreaterEqual,
            ">" => Self::Greater,
            "<=" => Self::LessEqual,
            "<" => Self::Less,
            "!=" => Self::NotEqual,
            "==" => Self::Equal,
            "=" => Self::Assign,

            "*" => Self::Mul,
            "%" => Self::Rem,
            "/" => Self::Div,
            "+" => Self::Add,
            "-" => Self::Sub,

            "[" => Self::BracketOpen,
            "]" => Self::BracketClose,
            "(" => Self::ParenOpen,
            ")" => Self::ParenClose,
            "{" => Self::BraceOpen,
            "}" => Self::BraceClose,
        );
        return None;
    }
}

macro symbol_match($parser: expr, $($st: expr => $sy: expr),+ $(,)?) {
    $(symbol_try!($parser, $st, $sy));+
}

macro symbol_try($parser: expr, $st: expr, $sy: expr) {
    if let Some(st) = $parser.try_consume_str($st) {
        return Some((st, $sy));
    }
}
