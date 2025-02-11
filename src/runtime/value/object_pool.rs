use std::{
    collections::HashSet,
    error::Error,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, RwLock,
    },
};

use indexmap::IndexMap;

use crate::runtime;

use super::{string_pool::StrReference, NativeValue, Value};

/// A pool for reference counted objects
/// Objects use a cycle detection scheme to properly dispose of values that have cycles
pub struct ObjectPool {
    finalize: Mutex<HashSet<usize>>,
    values: RwLock<Vec<ObjectPoolValue>>,
    free_indices: Mutex<Vec<usize>>,
}

pub struct ObjectPoolValue {
    value: RwLock<Option<Arc<Object>>>,
    ref_count: AtomicUsize,
}

pub struct ObjectReference {
    pool: Arc<ObjectPool>,
    index: usize,
}

pub struct Object {
    pub values: RwLock<IndexMap<StrReference, RwLock<Property>>>,
    pub prototype: RwLock<Option<ObjectReference>>,
    pub native_value: RwLock<Option<Arc<dyn NativeValue>>>,
}

pub enum Property {
    Value(Value),
    GetSet { get: Value, set: Value },
}

pub struct MarkChildren<'a> {
    pool: Arc<ObjectPool>,
    values: &'a Vec<ObjectPoolValue>,
    base_index: usize,
    count: usize,
    visited: HashSet<usize>,
}

impl ObjectPool {
    pub fn new() -> Arc<Self> {
        return Arc::new(Self {
            finalize: Mutex::new(HashSet::new()),
            values: RwLock::new(vec![]),
            free_indices: Mutex::new(vec![]),
        });
    }

    fn emplace<'a, TFn: FnOnce() -> Object>(
        self: &'a Arc<Self>,
        f: TFn,
    ) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        {
            let mut indices = self.free_indices.lock()?;

            let values = self.values.read().unwrap();

            if let Some(index) = indices.pop() {
                let value = values.get(index).unwrap();

                value.ref_count.store(1, Ordering::Relaxed);
                let mut value = value.value.write().unwrap();
                *value = Some(Arc::new(f()));

                return Ok(ObjectReference {
                    pool: self.clone(),
                    index,
                });
            }
        }

        let mut values = self.values.write()?;

        let index = values.len();

        values.push(ObjectPoolValue {
            value: RwLock::new(Some(Arc::new(f()))),
            ref_count: AtomicUsize::new(1),
        });

        return Ok(ObjectReference {
            pool: self.clone(),
            index,
        });
    }

    pub fn new_object<'a>(self: &'a Arc<Self>) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        return self.emplace(|| Object {
            values: RwLock::new(IndexMap::new()),
            prototype: RwLock::new(None),
            native_value: RwLock::new(None),
        });
    }

    pub fn new_object_proto<'a>(self: &'a Arc<Self>, prototype: ObjectReference) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        return self.emplace(|| Object {
            values: RwLock::new(IndexMap::new()),
            prototype: RwLock::new(Some(prototype)),
            native_value: RwLock::new(None),
        });
    }

    pub fn new_native_object<'a>(self: &'a Arc<Self>, value: Arc<dyn NativeValue>) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        return self.emplace(|| Object {
            values: RwLock::new(IndexMap::new()),
            prototype: RwLock::new(None),
            native_value: RwLock::new(Some(value)),
        });
    }

    pub fn new_native_object_prototype<'a>(self: &'a Arc<Self>, value: Arc<dyn NativeValue>, prototype: ObjectReference) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        return self.emplace(|| Object {
            values: RwLock::new(IndexMap::new()),
            prototype: RwLock::new(Some(prototype)),
            native_value: RwLock::new(Some(value)),
        });
    }

    fn get<'a>(
        self: &'a Arc<Self>,
        index: usize,
    ) -> Result<Option<Arc<Object>>, Box<dyn Error + 'a>> {
        let values = self.values.read()?;

        let Some(value) = values.get(index) else {
            return Ok(None);
        };

        return Ok(value.value.read().unwrap().clone());
    }

    fn clone_reference<'a>(
        self: &'a Arc<Self>,
        index: usize,
    ) -> Result<ObjectReference, Box<dyn Error + 'a>> {
        let values = self.values.read()?;

        let Some(value) = values.get(index) else {
            panic!("h");
        };

        value.ref_count.fetch_add(1, Ordering::Relaxed);

        return Ok(ObjectReference {
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

        let Some(value) = values.get(index) else {
            return Ok(());
        };

        value.ref_count.fetch_sub(1, Ordering::Relaxed);

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

                    let mut marker = MarkChildren::new(self.clone(), &values, base_index);

                    println!("Counting cycles for reference {}", base_index);

                    marker.mark_index(base_index);

                    let cycle_count = marker.count;

                    let ref_count = value.ref_count.load(Ordering::Relaxed);
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
                    *values[*index].value.write().unwrap() = None;
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

impl ObjectReference {
    pub fn get(&self) -> Arc<Object> {
        return self.pool.get(self.index).unwrap().unwrap();
    }

    pub fn get_property(&self, runtime: Arc<runtime::Runtime>, this_obj: &Value, property: &Value) -> Result<Value, Value> {
        let obj = self.get();

        if let Value::String(str) = property {
            let values = obj.values.read().unwrap();
            if values.contains_key(str) {
                let prop = values.get(str).unwrap();
                let prop = prop.read().unwrap();

                match (&prop) as &Property {
                    Property::Value(val) => {
                        let Value::Uninitialized = val else {
                            return Ok(val.clone());
                        };
                    },
                    Property::GetSet { get, set: _ } => {
                        let Value::Uninitialized = get else {
                            return get.invoke(runtime, this_obj, &[property.clone()]);
                        };
                    }
                }
            }
        }

        {
            let values = obj.values.read().unwrap();

            let get_index = runtime.string_pool.acquire("__get_index__".into()).unwrap();

            if values.contains_key(&get_index) {
                let prop = values.get(&get_index).unwrap();
                let prop = prop.read().unwrap();

                if let Property::Value(value) = (&prop) as &Property {
                    let result = value.invoke(runtime.clone(), this_obj, &[property.clone()])?;

                    let Value::Uninitialized = result else {
                        return Ok(result);
                    };
                }
            }
        }
        
        {
            let proto = obj.prototype.read().unwrap();
            
            if let Some(proto) = (&proto) as &Option<ObjectReference> {
                return proto.get_property(runtime, this_obj, property);
            }
        }

        return Ok(Value::Uninitialized);
    }

    fn set_index(&self, runtime: Arc<runtime::Runtime>, this_obj: &Value, property: &Value, value: &Value) -> Result<Value, Value>{
        let obj = self.get();

        {
            let values = obj.values.read().unwrap();

            let get_index = runtime.string_pool.acquire("__set_index__".into()).unwrap();

            if values.contains_key(&get_index) {
                let prop = values.get(&get_index).unwrap();
                let prop = prop.read().unwrap();

                if let Property::Value(value) = (&prop) as &Property {
                    let result = value.invoke(runtime.clone(), this_obj, &[property.clone(), value.clone()])?;

                    let Value::Uninitialized = result else {
                        return Ok(result);
                    };
                }
            }
        }
        
        {
            let proto = obj.prototype.read().unwrap();
            
            if let Some(proto) = (&proto) as &Option<ObjectReference> {
                return proto.set_index(runtime, this_obj, property, value);
            }
        }

        return Ok(Value::Uninitialized);
    }

    pub fn set_property(&self, runtime: Arc<runtime::Runtime>, this_obj: &Value, property: &Value, value: &Value) -> Result<Value, Value> {
        let obj = self.get();

        if let Value::String(str) = property {
            let values = obj.values.read().unwrap();
            if values.contains_key(str) {
                let prop = values.get(str).unwrap();
                let mut prop = prop.write().unwrap();

                match (&mut prop) as &mut Property {
                    Property::Value(val) => {
                        *val = value.clone();
                    },
                    Property::GetSet { get: _, set } => {
                        let Value::Uninitialized = set else {
                            return set.invoke(runtime, this_obj, &[property.clone(), value.clone()]);
                        };
                    }
                }
            }
        }

        return self.set_index(runtime, this_obj, property, value);
    }
}

impl Drop for ObjectReference {
    fn drop(&mut self) {
        self.pool.drop_reference(self.index).unwrap();
    }
}

impl Clone for ObjectReference {
    fn clone(&self) -> Self {
        return self.pool.clone_reference(self.index).unwrap();
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        if let Some(native_value) = self.native_value.read().unwrap().clone() {
            native_value.cleanup();
        }
    }
}

impl<'a> MarkChildren<'a> {
    fn new(pool: Arc<ObjectPool>, values: &'a Vec<ObjectPoolValue>, base_index: usize) -> Self {
        return Self {
            pool,
            values,
            base_index,
            count: 0,
            visited: HashSet::new(),
        };
    }

    pub fn mark_value(&mut self, child: &Value) {
        if let Value::Object(reference) = child {
            self.mark_reference(reference);
        }
    }

    pub fn mark_reference(&mut self, reference: &ObjectReference) {
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

    fn mark_index(&mut self, index: usize) {
        println!("Marking reference {}", index);

        self.visited.insert(index);

        let value = &self.values[index];

        if let Some(obj) = &*value.value.read().unwrap() {
            {
                let values = obj.values.read().unwrap();
                for child in values.iter() {
                    let property = child.1.read().unwrap();

                    match (&property) as &Property {
                        Property::Value(value) => self.mark_value(value),

                        Property::GetSet { get, set } => {
                            self.mark_value(get);
                            self.mark_value(set);
                        }
                    }
                }
            }

            {
                let proto = obj.prototype.read().unwrap();

                if let Some(proto) = (&proto) as &Option<ObjectReference> {
                    self.mark_reference(proto);
                }
            }

            {
                let native_value = obj.native_value.read().unwrap();

                if let Some(native_value) = native_value.clone() {
                    native_value.mark_children(self);
                }
            }
        }
    }
}
