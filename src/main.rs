#![feature(decl_macro, negative_impls, str_as_str, let_chains)]
#![allow(clippy::needless_return)]

use parse_tree::tree::ParseTree;
use tokenizer::Tokenizer;

pub mod bytecode;
pub mod parse_tree;
pub mod runtime;
pub mod string;
pub mod tokenizer;

fn main() {
    let mut tokenizer = Tokenizer::new(include_str!("../test.bur").into());
    println!("{:#?}", ParseTree::try_parse(&mut tokenizer));
}
