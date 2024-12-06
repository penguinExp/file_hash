pub trait Hashable {
    fn hash(&self) -> usize;
}

impl Hashable for String {
    // using the djb2 algo (https://theartincode.stanis.me/008-djb2/)
    fn hash(&self) -> usize {
        let mut result: usize = 5381;

        for c in self.chars() {
            result = ((result << 5).wrapping_add(result)).wrapping_add(c as usize);
        }

        result
    }
}

#[derive(Default, Clone, Copy)]
struct HashItem<Key, Value> {
    key: Key,
    value: Value,
    is_taken: bool,
}

pub struct HashTable<Key, Value> {
    kvs: Vec<HashItem<Key, Value>>,
    size: usize,
    no_of_taken: usize,
}

impl<Key: Default + Clone + PartialEq + Hashable, Value: Default + Clone> HashTable<Key, Value> {
    pub fn new() -> Self {
        const INITIAL_SIZE: usize = 61;

        Self {
            kvs: vec![HashItem::<_, _>::default(); INITIAL_SIZE],
            size: INITIAL_SIZE,
            no_of_taken: 0,
        }
    }

    pub fn insert(&mut self, key: Key, value: Value) {
        let load_factor = (self.size as f64 * 0.75) as usize;

        if self.no_of_taken >= load_factor {
            self.extend();
        }

        let mut index = self.get_hash_index(&key);

        for _ in 0..self.size {
            if !self.kvs[index].is_taken {
                self.kvs[index] = HashItem {
                    key: key.to_owned(),
                    value: value.to_owned(),
                    is_taken: true,
                };
                self.no_of_taken += 1;

                break;
            }

            if self.kvs[index].key == key {
                self.kvs[index].value = value.to_owned();
            }

            index = (index + 1) % self.size;
        }
    }

    pub fn get(&self, key: &Key) -> Option<&Value> {
        if let Some(index) = self.get_index(&key) {
            Some(&self.kvs[index].value)
        } else {
            None
        }
    }

    pub fn get_mut(&mut self, key: &Key) -> Option<&mut Value> {
        if let Some(index) = self.get_index(&key) {
            Some(&mut self.kvs[index].value)
        } else {
            None
        }
    }

    pub fn extend(&mut self) {
        let new_size = (self.size * 2) + 1;

        let mut new_self = Self {
            kvs: vec![HashItem::<_, _>::default(); new_size],
            size: new_size,
            no_of_taken: self.no_of_taken,
        };

        for item in self.kvs.iter() {
            if item.is_taken {
                new_self.insert(item.key.to_owned(), item.value.to_owned());
            }
        }

        *self = new_self;
    }

    fn get_index(&self, key: &Key) -> Option<usize> {
        let mut index: usize = self.get_hash_index(key);

        for _ in 0..self.size {
            // if no item found
            if !self.kvs[index].is_taken {
                break;
            }

            // if item found
            if self.kvs[index].key == *key {
                break;
            }

            index = (index + 1) % self.size;
        }

        if self.kvs[index].is_taken && self.kvs[index].key == *key {
            Some(index)
        } else {
            None
        }
    }

    fn get_hash_index(&self, key: &Key) -> usize {
        key.hash() % self.size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_and_get() {
        let mut hash_table = HashTable::<String, usize>::new();
        hash_table.insert("test".to_string(), 42);
        hash_table.insert("hello".to_string(), 100);

        assert_eq!(hash_table.get(&"test".to_string()), Some(&42));
        assert_eq!(hash_table.get(&"hello".to_string()), Some(&100));
        assert_eq!(hash_table.get(&"missing".to_string()), None); // key not present
    }

    #[test]
    fn test_insert_overwrite() {
        let mut hash_table = HashTable::<String, usize>::new();
        hash_table.insert("key".to_string(), 10);
        assert_eq!(hash_table.get(&"key".to_string()), Some(&10));

        hash_table.insert("key".to_string(), 20); // Overwrite existing key
        assert_eq!(hash_table.get(&"key".to_string()), Some(&20)); // Updated value
    }

    #[test]
    fn test_get_mut() {
        let mut hash_table = HashTable::<String, usize>::new();
        hash_table.insert("mutable".to_string(), 5);

        // Modify the value through a mutable reference
        if let Some(value) = hash_table.get_mut(&"mutable".to_string()) {
            *value = 15;
        }

        assert_eq!(hash_table.get(&"mutable".to_string()), Some(&15)); // Check updated value
    }

    #[test]
    fn test_extend() {
        let mut hash_table = HashTable::<String, usize>::new();

        // Insert enough items to trigger table extension
        for i in 0..100 {
            let key = format!("key_{}", i);
            hash_table.insert(key.clone(), i);
        }

        // Check a few keys to ensure they're still accessible after resizing
        assert_eq!(hash_table.get(&"key_0".to_string()), Some(&0));
        assert_eq!(hash_table.get(&"key_50".to_string()), Some(&50));
        assert_eq!(hash_table.get(&"key_99".to_string()), Some(&99));
    }

    #[test]
    fn test_extend_and_overwrite() {
        let mut hash_table = HashTable::<String, usize>::new();

        // Insert a large number of items to trigger table resizing
        for i in 0..100 {
            let key = format!("key_{}", i);
            hash_table.insert(key.clone(), i);
        }

        // Overwrite an existing key
        hash_table.insert("key_50".to_string(), 500);
        assert_eq!(hash_table.get(&"key_50".to_string()), Some(&500));
    }
}
