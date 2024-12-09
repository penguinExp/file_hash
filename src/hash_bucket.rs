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
    // index - u8 [0 - NULL; 1 - last; 2 - single; 3 - index; 4..n - shards]
    // key - [u8; 3]
    // value - [u8; 7] or [u8; 4]
    // indexes - [u16; 2]
}

impl Bucket {
    // index bucket
    // value bucket
    // single item bucket

    fn _index_bucket(key: &str, indexes: &Vec<u16>) -> [u8; 8] {
        let mut buffer = [b'\0'; 8];

        assert!(indexes.len() <= 2, "Can only contain 2 indexes at max");

        let mut key_bytes = Vec::from(key.as_bytes());
        key_bytes.resize(3, b'\0');

        let index = (3 as u8).to_le_bytes();

        buffer[0..1].clone_from_slice(&index);
        buffer[1..4].clone_from_slice(&key_bytes);
        buffer[4..6].clone_from_slice(&indexes[0].to_le_bytes());

        if indexes.len() == 1 {
            let empty_buffer = [b'\0'; 2];
            buffer[6..8].clone_from_slice(&empty_buffer);
        } else {
            buffer[6..8].clone_from_slice(&indexes[1].to_le_bytes());
        }

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

        let index = (2 as u8).to_le_bytes();

        buffer[0..1].clone_from_slice(&index);
        buffer[1..4].clone_from_slice(&key_bytes);
        buffer[4..8].clone_from_slice(&value_bytes);

        buffer
    }

    fn _split_value(value: Vec<u8>) -> Vec<[u8; 7]> {
        let mut chunks: Vec<[u8; 7]> = Vec::new();

        let chunk_size = 7;

        let val_len = value.len();

        let mut start = 0;

        while start < val_len {
            let end = std::cmp::min(start + chunk_size, val_len);
            let mut chunk = value[start..end].to_vec();

            chunk.resize(chunk_size, b'\0');

            let chunk = chunk.try_into().unwrap();

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
        let mut index = self._get_hash_index(&key);

        let value_bytes = Vec::from(value.as_bytes());
        let key_bytes = Vec::from(key.as_bytes());

        if key_bytes.len() > 3 {
            eprintln!("KEY should be smaller then 3");
            return;
        }

        let load_factor = (self.size as f64 * 0.75) as usize;

        if (self._no_of_taken + value_bytes.len() + 1) >= load_factor {
            // TODO: Extend the fucking kvs 🤬
        }

        // TODO: If the loop is over and no index is found
        // we got to handle the error 🥹
        for _ in 0..self.size {
            let offset = index * 8;
            assert!(offset + 8 <= self._kvs.len(), "Index out of bounds");

            let index_bytes: [u8; 1] = self._kvs[offset..(offset + 1)].try_into().unwrap();

            if index_bytes[0] == b'\0' {
                // Found the index
                break;
            }

            let bucket_index = u8::from_le_bytes(index_bytes);

            let key_bytes = &self._kvs[(offset + 1)..(offset + 4)];

            let saved_key = String::from_utf8_lossy(key_bytes)
                .trim_end_matches('\0')
                .to_string();

            if (bucket_index == 2 || bucket_index == 3) && saved_key == key {
                self.del(key);

                break;
            }

            index = (index + 1) % self.size;
        }

        // single item bucket
        if value_bytes.len() <= 4 {
            let bucket = Bucket::_single_item_bucket(key, value);
            self._write_at_index(bucket, index);

            return;
        }

        let chunks = Bucket::_split_value(value_bytes);
        let indexes = self._get_empty_indexes(chunks.len(), index);

        assert!(
            chunks.len() <= 2,
            "Value can only be parted into 2 chunks; not {}!",
            chunks.len()
        );

        // write an index bucket
        let index_bucket = Bucket::_index_bucket(key, &indexes);

        self._write_at_index(index_bucket, index);

        assert!(
            chunks.len() == indexes.len(),
            "Fuck up happened, [chunks]:{} and [indexes]:{} count does not match",
            chunks.len(),
            indexes.len(),
        );

        let last_index = chunks.len() - 1;

        for (i, chunk) in chunks.iter().enumerate() {
            let bucket: [u8; 8];

            if i == last_index {
                bucket = Bucket::_value_bucket(1, *chunk);
            } else {
                bucket = Bucket::_value_bucket((i + 4) as u8, *chunk);
            }

            self._write_at_index(bucket, indexes[i] as usize);
        }
    }

    pub fn get(&self, key: &str) -> Option<String> {
        let mut index = self._get_hash_index(&key);

        for _ in 0..self.size {
            let offset = index * 8;

            let index_bytes: [u8; 1] = self._kvs[offset..(offset + 1)].try_into().unwrap();

            if index_bytes[0] == b'\0' {
                index = (index + 1) % self.size;
                continue;
            }

            let bucket_index = u8::from_le_bytes(index_bytes);

            let key_bytes = &self._kvs[(offset + 1)..(offset + 4)];

            let saved_key = String::from_utf8_lossy(key_bytes)
                .trim_end_matches('\0')
                .to_string();

            if bucket_index == 2 && key == saved_key {
                let value_bytes = &self._kvs[(offset + 3)..(offset + 8)];

                if key == saved_key {
                    return Some(
                        String::from_utf8_lossy(value_bytes)
                            .trim_end_matches('\0')
                            .to_string(),
                    );
                }
            }

            if bucket_index == 3 && key == saved_key {
                let index_bytes = &self._kvs[(offset + 3)..(offset + 8)];

                let mut indexes: Vec<u16> = Vec::new();

                for win in index_bytes.windows(2) {
                    if win[0] != b'\0' {
                        let i = u16::from_le_bytes((*win).try_into().unwrap());

                        indexes.push(i);
                    }
                }

                let mut value_vec: Vec<u8> = Vec::new();

                for i in indexes {
                    let val_bytes = self._read_value_at_index(i as usize);

                    value_vec.append(&mut val_bytes.try_into().unwrap());
                }

                let val = String::from_utf8_lossy(&value_vec)
                    .trim_end_matches('\0')
                    .to_string();

                return Some(val);
            }

            index = (index + 1) % self.size;
        }

        None
    }

    pub fn del(&mut self, key: &str) -> Option<String> {
        let mut index = self._get_hash_index(&key);

        for _ in 0..self.size {
            let offset = index * 8;

            let index_bytes: [u8; 1] = self._kvs[offset..(offset + 1)].try_into().unwrap();

            if index_bytes[0] == b'\0' {
                return None;
            }

            let bucket_index = u8::from_le_bytes(index_bytes);

            let key_bytes = self._kvs[(offset + 1)..(offset + 4)].to_vec();

            let saved_key = String::from_utf8_lossy(&key_bytes)
                .trim_end_matches('\0')
                .to_string();

            if bucket_index == 2 && key == saved_key {
                let value_bytes = self._kvs[(offset + 3)..(offset + 8)].to_vec();

                self._del_at_index(index);

                return Some(
                    String::from_utf8_lossy(&value_bytes)
                        .trim_end_matches('\0')
                        .to_string(),
                );
            }

            if bucket_index == 3 && key == saved_key {
                let index_bytes = self._kvs[(offset + 3)..(offset + 8)].to_vec();

                self._del_at_index(index);

                let mut indexes: Vec<u16> = Vec::new();

                for win in index_bytes.windows(2) {
                    if win[0] != b'\0' {
                        let i = u16::from_le_bytes((*win).try_into().unwrap());

                        indexes.push(i);
                    }
                }

                let mut value_vec: Vec<u8> = Vec::new();

                for i in indexes {
                    let val_bytes = self._read_value_at_index(i as usize);

                    self._del_at_index(i as usize);

                    value_vec.append(&mut val_bytes.try_into().unwrap());
                }

                let val = String::from_utf8_lossy(&value_vec)
                    .trim_end_matches('\0')
                    .to_string();

                return Some(val);
            }

            index = (index + 1) % self.size;
        }

        None
    }

    fn _del_at_index(&mut self, index: usize) {
        let bucket = [b'\0'; 8];
        let offset = index * 8;

        self._kvs[offset..(offset + 8)].copy_from_slice(&bucket);
        self._no_of_taken -= 1;
    }

    fn _write_at_index(&mut self, bucket: [u8; 8], index: usize) {
        let offset = index * 8;

        self._kvs[offset..(offset + 8)].copy_from_slice(&bucket);
        self._no_of_taken += 1;
    }

    fn _read_value_at_index(&self, index: usize) -> [u8; 7] {
        let mut buffer = [b'\0'; 7];
        let offset = index * 8;

        buffer[0..7].copy_from_slice(&self._kvs[(offset + 1)..(offset + 8)]);

        buffer
    }

    fn _get_empty_indexes(&mut self, n: usize, index: usize) -> Vec<u16> {
        let mut indexes = Vec::new();
        let mut i = 0;

        // do not count the current index
        // it is for the index bucket
        let mut index = index + 1;

        while i < n {
            let offset = index * 8;
            assert!(offset + 8 <= self._kvs.len(), "Index out of bounds");

            let index_byte: [u8; 1] = self._kvs[offset..(offset + 1)].try_into().unwrap();

            if index_byte[0] == b'\0' {
                indexes.push(index as u16);
                i += 1;
            }

            index = (index + 1) % self.size;
        }

        indexes
    }

    fn _get_hash_index(&self, key: &str) -> usize {
        key.hash() % self.size
    }

    pub fn print_kvs(&self) {
        println!("");
        println!("Taken: {}", self._no_of_taken);
        println!("----------------");

        for i in 0..32 {
            let offset = i * 8;
            let buf = &self._kvs[offset..(offset + 8)];

            println!("{:?}", buf);
        }

        println!("----------------");
    }
}
