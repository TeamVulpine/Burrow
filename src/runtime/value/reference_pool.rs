use std::{
    collections::HashSet,
    error::Error,
    sync::{Arc, Mutex, RwLock},
};

use indexmap::IndexMap;

use crate::bytecode::CompiledModule;

use super::{string_pool::StrReference, Value};

/// A pool for reference counted objects
/// Objects use a cycle detection scheme to properly dispose of values that have cycles
pub struct ReferencePool {
    finalize: Mutex<HashSet<usize>>,
    values: RwLock<Vec<RwLock<Option<ReferencePoolValue>>>>,
    free_indices: Mutex<Vec<usize>>,
}

pub struct ReferencePoolValue {
    value: Arc<ReferenceType>,
    ref_count: usize,
}

pub struct Reference {
    pool: Arc<ReferencePool>,
    index: usize,
}

pub enum ReferenceType {
    Object(Object),
    Array(Array),
    Function(FunctionReference),
}

pub struct Object {
    pub values: RwLock<IndexMap<StrReference, ObjectValue>>,
    pub prototype: RwLock<Value>,
}

pub struct ObjectValue {
    pub is_const: RwLock<bool>,
    pub value: RwLock<Value>,
}

pub struct Array {
    pub values: RwLock<Vec<Value>>,
}

pub struct FunctionReference {
    pub module: Arc<CompiledModule>,
    pub index: usize,
    pub context: Value,
}

pub struct MarkChildren<'a> {
    pool: Arc<ReferencePool>,
    values: &'a Vec<RwLock<Option<ReferencePoolValue>>>,
    base_index: usize,
    count: usize,
    visited: HashSet<usize>,
}

pub trait ChildMarker {
    fn mark_children<'a>(&self, marker: &mut MarkChildren<'a>);
}

impl ReferencePool {
    pub fn new() -> Arc<Self> {
        return Arc::new(Self {
            finalize: Mutex::new(HashSet::new()),
            values: RwLock::new(vec![]),
            free_indices: Mutex::new(vec![]),
        });
    }

    fn emplace<'a, TFn: FnOnce() -> ReferenceType>(
        self: &'a Arc<Self>,
        f: TFn,
    ) -> Result<Reference, Box<dyn Error + 'a>> {
        let mut values = self.values.write()?;

        {
            let mut indices = self.free_indices.lock()?;

            if let Some(index) = indices.pop() {
                let lock = values.get(index).unwrap();
                let mut value = lock.write().unwrap();
                *value = Some(ReferencePoolValue {
                    value: Arc::new(f()),
                    ref_count: 1,
                });

                return Ok(Reference {
                    pool: self.clone(),
                    index,
                });
            }
        }

        let index = values.len();

        values.push(RwLock::new(Some(ReferencePoolValue {
            value: Arc::new(f()),
            ref_count: 1,
        })));

        return Ok(Reference {
            pool: self.clone(),
            index,
        });
    }

    pub fn new_object<'a>(self: &'a Arc<Self>) -> Result<Reference, Box<dyn Error + 'a>> {
        return self.emplace(|| {
            ReferenceType::Object(Object {
                values: RwLock::new(IndexMap::new()),
                prototype: RwLock::new(Value::None),
            })
        });
    }

    pub fn new_array<'a>(self: &'a Arc<Self>) -> Result<Reference, Box<dyn Error + 'a>> {
        return self.emplace(|| {
            ReferenceType::Array(Array {
                values: RwLock::new(vec![]),
            })
        });
    }

    fn get<'a>(
        self: &'a Arc<Self>,
        index: usize,
    ) -> Result<Option<Arc<ReferenceType>>, Box<dyn Error + 'a>> {
        let values = self.values.read()?;

        let Some(lock) = values.get(index) else {
            return Ok(None);
        };

        let Some(value) = &*lock.read().unwrap() else {
            return Ok(None);
        };

        return Ok(Some(value.value.clone()));
    }

    fn clone_reference<'a>(
        self: &'a Arc<Self>,
        index: usize,
    ) -> Result<Reference, Box<dyn Error + 'a>> {
        let values = self.values.read()?;

        let Some(lock) = values.get(index) else {
            panic!("h");
        };

        let Some(value) = &mut *lock.write().unwrap() else {
            panic!("h");
        };

        value.ref_count += 1;

        return Ok(Reference {
            pool: self.clone(),
            index,
        });
    }

    fn drop_reference<'a>(self: &'a Arc<Self>, index: usize) -> Result<(), Box<dyn Error + 'a>> {
        {
            let finalize = self.finalize.lock()?;

            if finalize.contains(&index) {
                return Ok(());
            }
        }

        let values = self.values.read()?;

        let Some(lock) = values.get(index) else {
            return Ok(());
        };

        if lock.read().unwrap().is_none() {
            return Ok(());
        }

        let Some(value) = &mut *lock.write().unwrap() else {
            return Ok(());
        };

        value.ref_count -= 1;

        return Ok(());
    }

    pub fn collect_garbage<'a>(self: &'a Arc<Self>) -> Result<(), Box<dyn Error + 'a>> {
        loop {
            let mut indices_to_delete = vec![];
            {
                let mut finalize = self.finalize.lock()?;

                let values = self.values.read()?;
                for base_index in 0..values.len() {
                    let value = &values[base_index];

                    let Some(value) = &*value.read().unwrap() else {
                        break;
                    };

                    let mut marker = MarkChildren::new(self.clone(), &values, base_index);

                    println!("Counting cycles for reference {}", base_index);

                    marker.mark_index(base_index);

                    let cycle_count = marker.count;

                    let ref_count = value.ref_count;
                    if ref_count <= cycle_count {
                        println!(
                            "Cycle count ({}) >= reference count ({}), deleting",
                            cycle_count, ref_count
                        );

                        indices_to_delete.push(base_index);
                        finalize.insert(base_index);
                    }
                }
            }

            {
                let values = self.values.read()?;
                for index in &indices_to_delete {
                    *values[*index].write().unwrap() = None;
                }
            }

            {
                let mut finalize = self.finalize.lock()?;
                finalize.clear();
            }

            if indices_to_delete.len() == 0 {
                break;
            }
        }

        return Ok(());
    }
}

impl Reference {
    pub fn get(&self) -> Arc<ReferenceType> {
        return self.pool.get(self.index).unwrap().unwrap();
    }
}

impl Drop for Reference {
    fn drop(&mut self) {
        self.pool.drop_reference(self.index).unwrap();
    }
}

impl Clone for Reference {
    fn clone(&self) -> Self {
        return self.pool.clone_reference(self.index).unwrap();
    }
}

impl<'a> MarkChildren<'a> {
    fn new(
        pool: Arc<ReferencePool>,
        values: &'a Vec<RwLock<Option<ReferencePoolValue>>>,
        base_index: usize,
    ) -> Self {
        return Self {
            pool,
            values,
            base_index,
            count: 0,
            visited: HashSet::new(),
        };
    }

    pub fn mark_child(&mut self, child: &Value) {
        if let Value::Reference(reference) = child {
            if !Arc::ptr_eq(&self.pool, &reference.pool) {
                panic!("Values from different runtimes cannot intermingle.");
            }

            if self.visited.contains(&reference.index) {
                println!("Found cycle of reference {}", reference.index);
                if reference.index == self.base_index {
                    self.count += 1;
                }
                return;
            }

            self.mark_index(reference.index);
        }
    }

    fn mark_index(&mut self, index: usize) {
        println!("Marking reference {}", index);

        self.visited.insert(index);

        let lock = &self.values[index];

        if let Some(value) = &*lock.read().unwrap() {
            match value.value.as_ref() {
                ReferenceType::Array(arr) => {
                    let values = arr.values.read().unwrap();
                    for child_index in 0..values.len() {
                        println!("Marking index {}", child_index);
                        self.mark_child(&values[child_index]);
                    }
                }

                ReferenceType::Object(obj) => {
                    {
                        let values = obj.values.read().unwrap();
                        for child in values.iter() {
                            println!("Marking field {}", child.0);

                            let field = child.1.value.read().unwrap();

                            self.mark_child(&field);
                        }
                    }

                    {
                        let proto = obj.prototype.read().unwrap();

                        println!("Marking prototype");

                        self.mark_child(&proto);
                    }
                }

                ReferenceType::Function(func) => {
                    println!("Marking function context");
                    self.mark_child(&func.context);
                }
            }
        }
    }
}

impl ObjectValue {
    pub fn of_mutable(v: Value) -> Self {
        return Self {
            is_const: RwLock::new(false),
            value: RwLock::new(v),
        };
    }

    pub fn of_immutable(v: Value) -> Self {
        return Self {
            is_const: RwLock::new(false),
            value: RwLock::new(v),
        };
    }
}
