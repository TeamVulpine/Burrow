use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    hash::Hash,
    sync::{Arc, Mutex},
};

/// A pool for immutable interned strings
///
/// This is different from ReferencePool, because this deduplicates values, and values in RefrerencePool aren't guarunteed to be immutable
pub struct StringPool {
    value_map: Mutex<HashMap<Arc<str>, usize>>,
    values: Mutex<Vec<Option<StringPoolValue>>>,
    free_indices: Mutex<Vec<usize>>,
}

struct StringPoolValue {
    value: Arc<str>,
    ref_count: usize,
}

pub struct StrReference {
    pool: Arc<StringPool>,
    index: usize,
}

impl StringPool {
    pub fn new() -> Arc<Self> {
        return Arc::new(Self {
            value_map: Mutex::new(HashMap::new()),
            values: Mutex::new(vec![]),
            free_indices: Mutex::new(vec![]),
        });
    }

    pub fn acquire<'a>(
        self: &'a Arc<Self>,
        s: Arc<str>,
    ) -> Result<StrReference, Box<dyn Error + 'a>> {
        let mut value_map = self.value_map.lock()?;

        let mut values = self.values.lock()?;

        if let Some(index) = value_map.get(&s) {
            let Some(Some(value)) = values.get_mut(*index) else {
                panic!("h");
            };

            value.ref_count += 1;

            return Ok(StrReference {
                pool: self.clone(),
                index: *index,
            });
        }

        {
            let mut indices = self.free_indices.lock()?;

            if let Some(index) = indices.pop() {
                values[index] = Some(StringPoolValue {
                    value: s.clone(),
                    ref_count: 1,
                });

                value_map.insert(s, index);

                return Ok(StrReference {
                    pool: self.clone(),
                    index,
                });
            }
        }

        let index = values.len();

        values.push(Some(StringPoolValue {
            value: s.clone(),
            ref_count: 1,
        }));

        value_map.insert(s, index);

        return Ok(StrReference {
            pool: self.clone(),
            index,
        });
    }

    fn clone_reference<'a>(
        self: &'a Arc<Self>,
        index: usize,
    ) -> Result<StrReference, Box<dyn Error + 'a>> {
        let mut values = self.values.lock()?;

        let Some(Some(value)) = values.get_mut(index) else {
            panic!("h");
        };

        value.ref_count += 1;

        return Ok(StrReference {
            pool: self.clone(),
            index,
        });
    }

    fn get<'a>(&'a self, index: usize) -> Result<Option<Arc<str>>, Box<dyn Error + 'a>> {
        let values = self.values.lock()?;

        let Some(Some(value)) = values.get(index) else {
            return Ok(None);
        };

        return Ok(Some(value.value.clone()));
    }

    fn drop_reference<'a>(self: &'a Arc<Self>, index: usize) -> Result<(), Box<dyn Error + 'a>> {
        let mut values = self.values.lock()?;

        let Some(Some(value)) = values.get_mut(index) else {
            return Ok(());
        };

        value.ref_count -= 1;

        if value.ref_count != 0 {
            return Ok(());
        }

        let mut value_map = self.value_map.lock()?;
        let mut indices = self.free_indices.lock()?;

        value_map.remove(&value.value);

        indices.push(index);

        values[index] = None;

        return Ok(());
    }
}

impl StrReference {
    pub fn get(&self) -> Arc<str> {
        return self.pool.get(self.index).unwrap().unwrap();
    }
}

impl Drop for StrReference {
    fn drop(&mut self) {
        self.pool.drop_reference(self.index).unwrap();
    }
}

impl Clone for StrReference {
    fn clone(&self) -> Self {
        return self.pool.clone_reference(self.index).unwrap();
    }
}

impl Debug for StrReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{:?}", self.get()));
    }
}

impl Display for StrReference {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return f.write_fmt(format_args!("{}", self.get()));
    }
}

impl PartialEq for StrReference {
    fn eq(&self, other: &Self) -> bool {
        if Arc::ptr_eq(&self.pool, &other.pool) {
            return self.index == other.index;
        }

        return self.get().as_str() == other.get().as_str();
    }
}

impl Eq for StrReference {}

impl Hash for StrReference {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (Arc::as_ptr(&self.pool) as usize).hash(state);
        self.index.hash(state);
    }
}
