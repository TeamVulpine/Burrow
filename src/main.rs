#![feature(decl_macro, negative_impls, str_as_str, let_chains)]
#![allow(clippy::needless_return)]

use std::sync::{Arc, RwLock};

use runtime::{value::{object_pool::Property, Value}, Runtime};

pub mod bytecode;
pub mod parse_tree;
pub mod runtime;
pub mod string;
pub mod tokenizer;

fn test(
    runtime: Arc<Runtime>,
    this_obj: &Value,
    params: &[Value],
) -> Result<Value, Value> {
    return Ok(Value::None);
}

fn main() {
    let rt = Runtime::new();

    {
        let array_prototype_ref = rt.object_pool.new_object().unwrap();

        let array_prototype = array_prototype_ref.get();

        let test_value = Value::Object(rt.object_pool.new_native_object(Arc::new(test)).unwrap());

        let mut values = array_prototype.values.write().unwrap();

        values.insert(rt.string_pool.acquire("__get_index__".into()).unwrap(), RwLock::new(Property::Value(test_value.clone())));
        values.insert(rt.string_pool.acquire("__set_index__".into()).unwrap(), RwLock::new(Property::Value(test_value.clone())));
        values.insert(rt.string_pool.acquire("length".into()).unwrap(), RwLock::new(Property::GetSet {
            get: test_value.clone(),
            set: test_value.clone(),
        }));
    }

    // let mut tokenizer = Tokenizer::new(include_str!("../test.bur").into());
    // let tree = ParseTree::try_parse(&mut tokenizer).unwrap().unwrap();

    // let mut bytecode = vec![];
    // tree.generate_init_bytecode(&mut bytecode).unwrap();
    // println!("{:#?}", bytecode);
}
