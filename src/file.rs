use std::fs::{File, OpenOptions};
use std::io::Write;
use std::path::Path;

const BUCKET_SIZE: u16 = 258;
const BUCKET_COUNT: u16 = 1024;
const FILE_PATH: &str = "hash.tc";

/// Bucket structure (minimal serialization logic)
#[derive(Debug)]
pub struct Bucket {
    index_indicator: u16, // 0 (end), 1 (single bucket), 2..n (for index)
    key: [u8; 128],       // Fixed-size key
    value: [u8; 128],     // Fixed-size value
}

impl Bucket {
    /// Creates an empty bucket
    fn new() -> Self {
        Self {
            index_indicator: 0,
            key: [b'\0'; 128],
            value: [b'\0'; 128],
        }
    }

    /// Converts a bucket to a byte vector
    fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(BUCKET_SIZE as usize);

        bytes.extend_from_slice(&self.index_indicator.to_le_bytes());
        bytes.extend_from_slice(&self.key);
        bytes.extend_from_slice(&self.value);

        bytes
    }

    /// Creates a bucket from a byte slice
    fn from_bytes(bytes: &[u8]) -> Self {
        let index_indicator: u16 = u16::from_le_bytes([bytes[0], bytes[1]]);
        let key = bytes[2..130].try_into().unwrap();
        let value = bytes[130..258].try_into().unwrap();

        Self {
            index_indicator,
            key,
            value,
        }
    }
}

/// File handler for the hash table
pub struct HashFile {
    file: File,
}

impl HashFile {
    pub fn init() -> Self {
        let file = if !Path::new(FILE_PATH).exists() {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .open(FILE_PATH)
                .expect("Failed to create file");

            let empty_bucket = Bucket::new().to_bytes();

            for _ in 0..BUCKET_COUNT {
                file.write_all(&empty_bucket).unwrap();
            }

            file
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(FILE_PATH)
                .expect("Failed to open file")
        };

        Self { file }
    }

    pub fn get(&self) -> Option<Bucket> {
        todo!()
    }

    pub fn set(&self) -> bool {
        todo!()
    }

    pub fn delete(&self) -> Option<Bucket> {
        todo!()
    }
}
