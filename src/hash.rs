trait Hashable {
    fn hash(&self) -> usize;
}

impl Hashable for &str {
    // using the djb2 algo (https://theartincode.stanis.me/008-djb2/)
    fn hash(&self) -> usize {
        let mut result: usize = 5381;

        for c in self.chars() {
            result = ((result << 5).wrapping_add(result)).wrapping_add(c as usize);
        }

        result
    }
}

struct HashItem {
    key: [u8; 32],
    value: [u8; 96],
}

impl HashItem {
    fn new(key: &str, value: &str) -> [u8; 128] {
        let mut buffer = [b'\0'; 128];

        let mut key_bytes = Vec::from(key.as_bytes());
        let mut value_bytes = Vec::from(value.as_bytes());

        key_bytes.resize(32, b'\0');
        value_bytes.resize(96, b'\0');

        buffer[0..32].copy_from_slice(&key_bytes);
        buffer[32..128].copy_from_slice(&value_bytes);

        buffer
    }

    fn from_bytes(bytes: &[u8; 128]) -> Option<Self> {
        if bytes[0] == b'\0' {
            None
        } else {
            Some(Self {
                key: bytes[0..32].try_into().unwrap(),
                value: bytes[32..128].try_into().unwrap(),
            })
        }
    }
}

pub struct HashTable {
    kvs: Vec<u8>,
    size: usize,
    no_of_taken: usize,
}

impl HashTable {
    pub fn new() -> Self {
        Self {
            kvs: vec![b'\0'; 4096],
            size: 32,
            no_of_taken: 0,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let load_factor = (self.size as f64 * 0.75) as usize;

        if self.no_of_taken >= load_factor {
            self.extend();
        }

        let mut index = self.get_hash_index(&key);
        let bucket = HashItem::new(key, value);

        for _ in 0..self.size {
            let offset = index * 128;
            assert!(offset + 128 <= self.kvs.len(), "Index out of bounds");

            let bytes = self.kvs[offset..(offset + 128)].try_into().unwrap();

            match HashItem::from_bytes(bytes) {
                Some(item) => {
                    let stored_key = String::from_utf8_lossy(&item.key)
                        .trim_end_matches('\0')
                        .to_string();

                    if &stored_key == key {
                        self.kvs[offset..(offset + 128)].copy_from_slice(&bucket);
                        break;
                    }
                }
                None => {
                    self.kvs[offset..(offset + 128)].clone_from_slice(&bucket);

                    self.no_of_taken += 1;
                    break;
                }
            }

            index = (index + 1) % self.size;
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut index = self.get_hash_index(key);

        for _ in 0..self.size {
            let offset = index * 128;
            assert!(offset + 128 <= self.kvs.len(), "Index out of bounds");

            let bytes = self.kvs[offset..(offset + 128)].try_into().unwrap();

            match HashItem::from_bytes(bytes) {
                Some(item) => {
                    let stored_key = String::from_utf8_lossy(&item.key)
                        .trim_end_matches('\0')
                        .to_string();

                    if &stored_key == key {
                        let stored_value = String::from_utf8_lossy(&item.value)
                            .trim_end_matches('\0')
                            .to_string();

                        return Some(stored_value);
                    }
                }
                None => {
                    return None;
                }
            }

            index = (index + 1) % self.size;
        }

        None
    }

    pub fn del(&mut self, key: &str) -> Option<String> {
        let mut index = self.get_hash_index(&key);

        for _ in 0..self.size {
            let offset = index * 128;
            assert!(offset + 128 <= self.kvs.len(), "Index out of bounds");

            let bytes = self.kvs[offset..(offset + 128)].try_into().unwrap();
            let bucket = [b'\0'; 128];

            match HashItem::from_bytes(bytes) {
                Some(item) => {
                    let stored_key = String::from_utf8_lossy(&item.key)
                        .trim_end_matches('\0')
                        .to_string();

                    if &stored_key == key {
                        self.kvs[offset..(offset + 128)].copy_from_slice(&bucket);

                        let stored_value = String::from_utf8_lossy(&item.value)
                            .trim_end_matches('\0')
                            .to_string();

                        self.no_of_taken -= 1;

                        return Some(stored_value);
                    }
                }
                None => {
                    return None;
                }
            }

            index = (index + 1) % self.size;
        }

        None
    }

    fn extend(&mut self) {
        let new_size = self.size * 2;

        let mut new_self = HashTable {
            kvs: vec![b'\0'; new_size * 128],
            size: new_size,
            no_of_taken: 0,
        };

        self.size = self.size * 2;
        self.kvs = vec![b'\0'; self.size * 128];

        let mut offset: usize = 0;

        for i in 1..=self.size {
            let end_offset: usize = i * 128;
            let bytes: &[u8; 128] = self.kvs[offset..end_offset].try_into().unwrap();
            let bucket = HashItem::from_bytes(bytes);

            match bucket {
                Some(item) => {
                    let key = String::from_utf8_lossy(&item.key)
                        .trim_end_matches('\0')
                        .to_string();

                    let value = String::from_utf8_lossy(&item.value)
                        .trim_end_matches('\0')
                        .to_string();

                    new_self.set(&key, &value);
                }
                None => {}
            }

            offset = end_offset;
        }

        *self = new_self;
    }

    fn get_hash_index(&self, key: &str) -> usize {
        key.hash() % self.size
    }
}
