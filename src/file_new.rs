use std::{
    fs::{File, OpenOptions},
    hash::{DefaultHasher, Hash, Hasher},
    io::{Read, Seek, SeekFrom, Write},
};

const FILE_PATH: &str = "hash.tc";
const BUCKETS_COUNT: u64 = 32;
const BUCKET_SIZE: usize = 128;
const KEY_SIZE: usize = 16;
const VALUE_SIZE: usize = 110;
const INDEX_SIZE: usize = 2;

pub struct FileHash {
    file: File,
}

// 128 (2 + 16 + 110) bytes
struct Bucket {
    // index: [u8; INDEX_SIZE],
    // key: [u8; KEY_SIZE],
    // value: [u8; VALUE_SIZE],
}

impl Bucket {
    fn new(key: &str, value: &str, index: u16) -> [u8; 128] {
        let mut key_bytes = Vec::from(key.as_bytes());
        let mut value_bytes = Vec::from(value.as_bytes());
        let index = index.to_le_bytes();

        key_bytes.resize(32, b'\0');
        value_bytes.resize(94, b'\0');

        let mut buffer = [b'\0'; 128];

        buffer[0..INDEX_SIZE].copy_from_slice(&index);
        buffer[INDEX_SIZE..(INDEX_SIZE + KEY_SIZE)].copy_from_slice(&key_bytes);
        buffer[(INDEX_SIZE + KEY_SIZE)..(BUCKET_SIZE)].copy_from_slice(&value_bytes);

        buffer
    }

    // fn from_bytes(bytes: [u8; BUCKET_SIZE]) -> Self {
    //     Self {
    //         index: bytes[0..INDEX_SIZE].try_into().unwrap(),
    //         key: bytes[INDEX_SIZE..(INDEX_SIZE + KEY_SIZE)]
    //             .try_into()
    //             .unwrap(),
    //         value: bytes[(INDEX_SIZE + KEY_SIZE)..(BUCKET_SIZE)]
    //             .try_into()
    //             .unwrap(),
    //     }
    // }

    fn get_index_from_bytes(bytes: [u8; INDEX_SIZE]) -> u16 {
        u16::from_le_bytes(bytes)
    }

    fn get_key_from_bytes(bytes: [u8; KEY_SIZE]) -> String {
        String::from_utf8_lossy(&bytes)
            .trim_end_matches('\0')
            .to_string()
    }

    fn get_value_from_bytes(bytes: [u8; VALUE_SIZE]) -> String {
        String::from_utf8_lossy(&bytes)
            .trim_end_matches('\0')
            .to_string()
    }
}

impl FileHash {
    pub fn init() -> Self {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(FILE_PATH)
            .expect("Unable to open the file");

        if file.metadata().expect("Expected metadata").len() == 0 {
            let count = (0 as u64).to_be_bytes();

            file.write_all(&count).expect("Unable to write count");

            let buffer = [b'\0'; BUCKET_SIZE];

            for _ in 0..BUCKETS_COUNT {
                file.write_all(&buffer)
                    .expect("Unable to write empty bucket");
            }
        }

        Self { file }
    }

    pub fn write(&mut self, key: &str, value: &str) {
        if key.len() > KEY_SIZE {
            eprintln!("[ERR] Key size should be less then {KEY_SIZE}");
            return;
        }

        let mut index = Self::hash(key);

        if value.len() > VALUE_SIZE {
            eprintln!("[ERR] Value size should be less then {VALUE_SIZE}");

            return;
        }

        loop {
            let bucket_index = self.read_index_at_offset(index);

            match bucket_index {
                Some(_) => {
                    // update the value for the same key
                    index = (index + 1) % BUCKETS_COUNT;
                }
                None => {
                    break;
                }
            }
        }

        let bucket = Bucket::new(key, value, 1);

        self.file
            .write_all(&bucket)
            .expect("Unable to write bucket");

        self.update_count();
    }

    fn update_count(&mut self) {
        self.file.seek(SeekFrom::Start(0)).expect("Unable to seek");

        let mut buf = [b'\0'; 8];

        self.file
            .read_exact(&mut buf)
            .expect("Unable to read count");

        let count = u64::from_be_bytes(buf) + 1;

        self.file.seek(SeekFrom::Start(0)).expect("Unable to seek");

        self.file
            .write_all(&count.to_le_bytes())
            .expect("Unable to write count");
    }

    fn read_index_at_offset(&mut self, index: u64) -> Option<u16> {
        self.file
            .seek(SeekFrom::Start(index * (BUCKET_SIZE as u64) + 8))
            .expect("Unable to seek");

        let mut buffer = [b'\0'; 2];

        self.file.read_exact(&mut buffer).expect("Unable to read");

        if buffer[0] != b'\0' {
            Some(Bucket::get_index_from_bytes(buffer))
        } else {
            None
        }
    }

    fn hash(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();

        key.hash(&mut hasher);

        let val = hasher.finish();

        return val % BUCKETS_COUNT;
    }
}
