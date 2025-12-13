use crate::chunk::Value;

const HashTableMaxLoad: f64 = 0.75;

pub struct HashTable {
    entries: Vec<(Option<Value>, Option<Value>)>,
    capacity: usize,
    count: usize,
}

impl HashTable {
    pub fn new(capacity: usize) -> Self {
        HashTable {
            entries: vec![(None, None); capacity],
            capacity,
            count: 0,
        }
    }

    pub fn insert(&mut self, key: Value, value: Value) {
        if self.count as f64 / self.capacity as f64 > HashTableMaxLoad {
            self.resize(self.capacity * 2);
        }

        if let Some((index, set)) = self.find_entry(&key) {
            self.entries[index] = (Some(key), Some(value));
            if set == false {
                self.count += 1;
            }
        } else {
            panic!("HashTable is full.");
        }
    }

    pub fn get(&self, key: &Value) -> Option<&Value> {
        if let Some((index, _)) = self.find_entry(key) {
            let (ref entry_key, ref entry_value) = self.entries[index];
            if (entry_key.is_none()) {
                return None;
            }
            entry_value.as_ref()
        } else {
            None
        }
    }

    /// Linear probing to find the index of the key
    /// Returns Some((index, found)) where found is true if the key exists
    /// Returns None if the table is full
    fn find_entry(&self, key: &Value) -> Option<(usize, bool)> {
        let key_hash = hash_fnv1a(key);
        let mut index = (key_hash % self.capacity as u64) as usize;
        for _ in 0..self.capacity {
            let (ref entry_key, _) = self.entries[index];
            if entry_key.is_none() {
                return Some((index, false));
            }
            if hash_fnv1a(entry_key.as_ref().unwrap()) == key_hash {
                return Some((index, true));
            }
            index = (index + 1) % self.capacity;
        }
        None
    }

    fn resize(&mut self, new_capacity: usize) {
        let mut new_table = HashTable::new(new_capacity);
        for (entry_key, entry_value) in &self.entries {
            if let Some(key) = entry_key {
                if let Some(value) = entry_value {
                    new_table.insert(key.clone(), value.clone());
                }
            }
        }
        *self = new_table;
    }
}

pub fn hash_fnv1a(value: &Value) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    let type_bytes = match value {
        Value::Number(_) => b"NUM",
        Value::String(_) => b"STR",
        Value::Bool(_) => b"BOO",
        Value::Nil => b"NIL",
    };
    for byte in type_bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    let values_bytes = match value {
        Value::Number(n) => n.to_le_bytes().to_vec(),
        Value::String(s) => s.as_bytes().to_vec(),
        Value::Bool(b) => vec![*b as u8],
        Value::Nil => vec![],
    };
    for byte in values_bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

mod tests {
    use super::*;

    #[test]
    fn test_hash_table_insert_and_get() {
        let mut table = HashTable::new(16);
        let key = Value::String("key".into());
        let value = Value::Number(42.0);
        table.insert(key.clone(), value.clone());
        assert_eq!(table.get(&key), Some(&value));
    }

    #[test]
    fn test_hash_table_overflow() {
        let mut table = HashTable::new(2);
        for i in 0..10 {
            let key = Value::Number(i as f64);
            let value = Value::Number((i * 10) as f64);
            table.insert(key.clone(), value.clone());
            assert_eq!(table.get(&key), Some(&value));
        }
    }

    #[test]
    fn test_hash_table_overwrite() {
        let mut table = HashTable::new(16);
        let key = Value::String("key".into());
        let value1 = Value::Number(42.0);
        let value2 = Value::Number(84.0);
        table.insert(key.clone(), value1.clone());
        assert_eq!(table.get(&key), Some(&value1));
        table.insert(key.clone(), value2.clone());
        assert_eq!(table.get(&key), Some(&value2));
    }

    #[test]
    fn test_hash_table_not_found() {
        let table = HashTable::new(16);
        let key = Value::String("nonexistent".into());
        assert_eq!(table.get(&key), None);
    }

    #[test]
    fn test_hash_fnv1a_numbers() {
        let value1 = Value::Number(42.0);
        let value2 = Value::Number(42.0);
        let value3 = Value::Number(43.0);

        let hash1 = hash_fnv1a(&value1);
        let hash2 = hash_fnv1a(&value2);
        let hash3 = hash_fnv1a(&value3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_hash_fnv1a_strings() {
        let value1 = Value::String("Hello".into());
        let value2 = Value::String("Hello".into());
        let value3 = Value::String("World".into());

        let hash1 = hash_fnv1a(&value1);
        let hash2 = hash_fnv1a(&value2);
        let hash3 = hash_fnv1a(&value3);

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }
}
