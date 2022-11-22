use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::Hash;

pub struct Cache<K: Eq + Hash, V> {
    data: RefCell<HashMap<K, V>>,

    generator: Box<dyn Fn(&K) -> Option<V>>
}

impl<K: Eq + Hash, V> Cache<K, V> {
    pub fn new<F: Fn(&K) -> Option<V> + 'static>(generator: F) -> Cache<K, V> {
        Cache {
            data: RefCell::new(HashMap::new()),
            generator: Box::new(generator)
        }
    }
}

impl<K: Eq + Hash + Clone, V: Clone> Cache<K, V> {
    pub fn get(&self, key: &K) -> Option<V> {
        self.data.borrow().get(key).cloned()
            .or_else(|| {
                let value = (self.generator)(key);
                if let Some(value) = value {
                    self.data.borrow_mut().insert(key.clone(), value.clone());
                    Some(value)
                } else {
                    None
                }
            })
    }
}