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
    fn new(key: &str, value: &str, index: u16) -> [u8; 128] {
        let mut key_bytes = Vec::from(key.as_bytes());
        let mut value_bytes = Vec::from(value.as_bytes());

        key_bytes.resize(32, b'\0');
        value_bytes.resize(94, b'\0');

        let mut buffer = [b'\0'; 128];

        buffer[0..2].copy_from_slice(&(index.to_le_bytes()));
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

        let hash = Self::hash(key);
        let offset: u64 = hash * BUCKET_SIZE as u64;

        // if value is too large to fit in one bucket,
        // split and store it across multiple buckets
        // for simplicity just using linear probing
        if value.len() > 94 {
            let val_chunks = Self::split_value(value);

            // NOTE:
            //
            // Index starts at `0` here,
            // in a bucket, `0` is for the last bucket, `1` is for single buckets
            // and for sharded buckets, index starts from `2`
            for (index, val) in val_chunks.iter().enumerate() {
                let i: u16;

                if index == val_chunks.len() - 1 {
                    i = 0; // if it's last shard
                } else {
                    i = index as u16 + 2; // count starts from 2
                }

                self.write_shard(key, *val, offset, i)?;
            }

            return Ok(());
        }

        let buffer = Bucket::new(key, value, 1);

        let mut pos = offset;

        loop {
            let mut buf = [b'\0'; 1];

            // TODO: I can also `SeekFrom::Current` to save computation cost
            self.file.seek(SeekFrom::Start(pos))?;
            self.file.read_exact(&mut buf)?;

            if buf[0] != b'\0' {
                pos += BUCKET_SIZE as u64;
                continue;
            }

            break;
        }

        // self.file.seek(SeekFrom::Start(offset))?;
        self.file.write_all(&buffer)?;

        Ok(())
    }

    fn write_shard(&mut self, key: &str, value: &str, offset: u64, index: u16) -> io::Result<()> {
        let mut pos = offset;

        loop {
            pos += BUCKET_SIZE as u64;

            let mut buffer = [b'\0'; 1];

            self.file.seek(SeekFrom::Start(pos))?;
            self.file.read_exact(&mut buffer)?;

            if buffer[0] == b'\0' {
                continue;
            }

            let buffer = Bucket::new(key, value, index);

            self.file.write_all(&buffer)?;

            break;
        }

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

    // fn read_shard(&mut self, key: &str) -> Option<String> {
    //     None
    // }

    fn hash(key: &str) -> u64 {
        let mut hasher = DefaultHasher::new();

        key.hash(&mut hasher);

        let val = hasher.finish();

        return val % BUCKETS_COUNT;
    }

    fn split_value(value: &str) -> Vec<&str> {
        let mut chunks = Vec::new();
        let mut start: usize = 0;

        let chunk_size = 94;
        let val_len = value.len();

        while start < val_len {
            let end = std::cmp::min(start + chunk_size, val_len);
            chunks.push(&value[start..end]);

            start += chunk_size;
        }

        chunks
    }
}
