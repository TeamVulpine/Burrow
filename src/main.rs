#![feature(decl_macro, negative_impls, str_as_str, let_chains)]
#![allow(clippy::needless_return)]

use runtime::{
    value::{
        reference_pool::{ObjectValue, ReferenceType},
        Value,
    },
    Runtime,
};

pub mod bytecode;
pub mod parse_tree;
pub mod runtime;
pub mod string;
pub mod tokenizer;

fn main() {
    let rt = Runtime::new();
    let name = rt.string_pool.acquire("test".into()).unwrap();

    {
        let obj_ref = rt.reference_pool.new_object().unwrap();
        let obj = obj_ref.get();
        let ReferenceType::Object(val) = obj.as_ref() else {
            panic!();
        };
        let mut values = val.values.write().unwrap();

        values.insert(
            name,
            ObjectValue::of_mutable(Value::Reference(obj_ref.clone())),
        );
    }

    rt.reference_pool.collect_garbage().unwrap();

    // let mut tokenizer = Tokenizer::new(include_str!("../test.bur").into());
    // println!("{:#?}", ParseTree::try_parse(&mut tokenizer));
}
