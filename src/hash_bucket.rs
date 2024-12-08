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

struct Bucket {
    // index - u8 [0 - last; 1 - single; 2 - index; 3..n - shards]
    // key - [u8; 3]
    // value - [u8; 7] or [u8; 4]
    // indexes - [u16; 2]
}

impl Bucket {
    // index bucket
    // value bucket
    // single item bucket

    fn _index_bucket(key: &str, indexes: [u16; 2]) -> [u8; 8] {
        let mut buffer = [b'\0'; 8];

        assert!(indexes.len() == 2, "Can only contain 2 indexes at max");

        let mut key_bytes = Vec::from(key.as_bytes());
        key_bytes.resize(3, b'\0');

        let index = (2 as u8).to_le_bytes();

        buffer[0..1].clone_from_slice(&index);
        buffer[1..4].clone_from_slice(&key_bytes);
        buffer[4..6].clone_from_slice(&indexes[0].to_le_bytes());
        buffer[6..8].clone_from_slice(&indexes[1].to_le_bytes());

        buffer
    }

    fn _value_bucket(index: u8, value: [u8; 7]) -> [u8; 8] {
        let mut buffer = [b'\0'; 8];

        let index = index.to_le_bytes();

        buffer[0..1].clone_from_slice(&index);
        buffer[1..8].clone_from_slice(&value);

        buffer
    }

    fn _single_item_bucket(key: &str, value: &str) -> [u8; 8] {
        let mut buffer = [b'\0'; 8];

        let mut key_bytes = Vec::from(key.as_bytes());
        key_bytes.resize(3, b'\0');

        let mut value_bytes = Vec::from(value.as_bytes());
        value_bytes.resize(4, b'\0');

        let index = (1 as u8).to_le_bytes();

        buffer[0..1].clone_from_slice(&index);
        buffer[1..4].clone_from_slice(&key_bytes);
        buffer[4..8].clone_from_slice(&value_bytes);

        buffer
    }

    fn _split_value(value: Vec<u8>) -> Vec<Vec<u8>> {
        let mut chunks: Vec<Vec<u8>> = Vec::new();

        let chunk_size = 7;

        let val_len = value.len();

        let mut start = 0;

        while start < val_len {
            let end = std::cmp::min(start + chunk_size, val_len);
            let mut chunk = value[start..end].to_vec();

            chunk.resize(chunk_size, b'\0');

            chunks.push(chunk);
            start += chunk_size;
        }

        chunks
    }
}

pub struct HashTable {
    _kvs: Vec<u8>,
    size: usize,
    _no_of_taken: usize,
}

impl HashTable {
    pub fn new() -> Self {
        let size = 32;

        Self {
            _kvs: vec![b'\0'; size * 8],
            size,
            _no_of_taken: 0,
        }
    }

    pub fn set(&mut self, key: &str, value: &str) {
        let _ = self._get_hash_index(&key);

        let value_bytes = Vec::from(value.as_bytes());

        if value_bytes.len() <= 4 {
            // TODO: Need a single bucket
        } else if value_bytes.len() <= 7 {
            // TODO: Only one index bucket will be needed
        }

        let chunks = Bucket::_split_value(value_bytes);

        for chunk in chunks {
            println!("{:?}", chunk);
        }
    }

    fn _get_hash_index(&self, key: &str) -> usize {
        key.hash() % self.size
    }
}
