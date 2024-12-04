use std::{
    fs::{File, OpenOptions},
    hash::{DefaultHasher, Hash, Hasher},
    io::{self, Read, Seek, SeekFrom, Write},
};

const FILE_PATH: &str = "hash.tc";
const BUCKETS_COUNT: u64 = 32;
const BUCKET_SIZE: usize = 128;

pub struct FileHash {
    file: File,
}

// 128 (2 + 32 + 92)
struct Bucket {
    // index: u16,
    // key: [u8; 32],
    // value: [u8; 92],
}

impl Bucket {
    fn new(key: &str, value: &str) -> [u8; 128] {
        let mut key_bytes = Vec::from(key.as_bytes());
        let mut value_bytes = Vec::from(value.as_bytes());

        key_bytes.resize(32, b'\0');
        value_bytes.resize(94, b'\0');

        let mut buffer = [b'\0'; 128];

        buffer[0..2].copy_from_slice(&((1 as u16).to_le_bytes()));
        buffer[2..34].copy_from_slice(&key_bytes);
        buffer[34..128].copy_from_slice(&value_bytes);

        buffer
    }
}

impl FileHash {
    pub fn init() -> io::Result<Self> {
        let mut file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(FILE_PATH)?;

        if file.metadata()?.len() == 0 {
            let buffer = [b'\0'; BUCKET_SIZE];

            for _ in 0..BUCKETS_COUNT {
                file.write_all(&buffer)?;
            }
        }

        Ok(Self { file })
    }

    pub fn write(&mut self, key: &str, value: &str) -> io::Result<()> {
        if key.len() > 32 {
            eprintln!("Key is too long");
            return Ok(());
        }

        if value.len() > 94 {
            eprintln!("value is too long");
            return Ok(());
        }

        let index = Self::hash(key);

        let buffer = Bucket::new(key, value);

        self.file
            .seek(SeekFrom::Start(index * BUCKET_SIZE as u64))?;

        self.file.write_all(&buffer)?;

        Ok(())
    }

    pub fn read(&mut self, key: &str) -> io::Result<String> {
        let index = Self::hash(key);

        self.file
            .seek(SeekFrom::Start(index * BUCKET_SIZE as u64))?;

        let mut buffer = [b'\0'; 128];

        self.file.read_exact(&mut buffer)?;

        let value = &buffer[34..128];

        Ok(String::from_utf8(value.to_vec()).unwrap())
    }

    fn hash(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();

        key.hash(&mut hasher);

        let val = hasher.finish();

        return val % BUCKETS_COUNT;
    }
}
