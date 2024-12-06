//!
//! # FileHash
//!
//! Store KV pairs on file
//!
//! ## Structure
//!
//! - Fixed sized buckets should be stored,
//! - 64 buckets in a file
//! - fixed size keys, values can be of any size
//! - for larger values shard across various buckets
//!
//! ```rust
//!
//! /// Bucket structure (minimal serialization logic)
//! #[derive(Debug)]
//! pub struct Bucket {
//!     index_indicator: u16, // 2 bytes; 0 (end), 1 (single bucket), 2..n (for index)
//!     pub key: [u8; 64],    // 64 bytes
//!     pub value: [u8; 190], // 190 bytes
//! }
//!
//! ```
//! ## API
//!
//! - `init` -> Create and fill file if not already
//! - `add` -> Add KV entry into file
//! - `get` -> Read the value
//! - `delete` -> Del the pair and return the value
//!
//! ## Notes
//!
//! - Only FileHash and it's methods needs to be public
//! - Do not expose unnecessary components
//! - Add good docs and notes
//! - Write a test module to test the functionality
//!

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::Path;

/// FileHash provides a disk-based key-value storage system with fixed-size buckets
pub struct FileHash {
    file: File,
    num_buckets: usize,
    bucket_size: usize,
}

/// Bucket structure for storing key-value pairs
#[derive(Debug, Clone)]
struct Bucket {
    /// Indicates the bucket's state and potential chaining
    /// 0: End/Empty, 1: Single bucket, 2..n: Chained bucket index
    index_indicator: u16,

    /// Fixed-size key storage (64 bytes)
    key: [u8; 64],

    /// Fixed-size value storage (190 bytes)
    value: [u8; 190],
}

impl FileHash {
    /// Create a new FileHash or open an existing one
    ///
    /// # Arguments
    /// * `path` - File path for the hash table storage
    ///
    /// # Returns
    /// Result with the initialized FileHash or an error
    pub fn init<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)?;

        let num_buckets = 64;
        let bucket_size = std::mem::size_of::<Bucket>();

        // Initialize file with empty buckets if it's empty
        if file.metadata()?.len() == 0 {
            let empty_bucket = Bucket {
                index_indicator: 0,
                key: [0; 64],
                value: [0; 190],
            };

            let mut file_handle = file.try_clone()?;
            for _ in 0..num_buckets {
                Self::write_bucket(&mut file_handle, &empty_bucket)?;
            }
        }

        Ok(Self {
            file,
            num_buckets,
            bucket_size,
        })
    }

    /// Add a key-value pair to the hash table
    ///
    /// # Arguments
    /// * `key` - Fixed-size 64-byte key
    /// * `value` - Value to store (up to 190 bytes)
    ///
    /// # Returns
    /// Result indicating success or failure of the operation
    pub fn add(&mut self, key: [u8; 64], value: &[u8]) -> io::Result<()> {
        // Validate input
        if value.len() > 190 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Value exceeds maximum size of 190 bytes",
            ));
        }

        // Simple hash function (for demonstration)
        let bucket_index = self.hash(&key);

        // Try to find an empty or matching bucket
        let mut current_index = bucket_index;
        loop {
            let mut current_bucket = self.read_bucket(current_index)?;

            // Empty bucket found
            if current_bucket.index_indicator == 0 {
                current_bucket.index_indicator = 1;
                current_bucket.key = key;

                // Zero-fill and copy value
                current_bucket.value = [0; 190];
                current_bucket.value[..value.len()].copy_from_slice(value);

                self.write_bucket_at_index(current_index, &current_bucket)?;
                return Ok(());
            }

            // TODO: Implement more sophisticated collision handling
            // For now, this is a basic linear probing approach
            current_index = (current_index + 1) % self.num_buckets;

            // Prevent infinite loop if no buckets are available
            if current_index == bucket_index {
                return Err(io::Error::new(io::ErrorKind::Other, "No available buckets"));
            }
        }
    }

    /// Retrieve a value by its key
    ///
    /// # Arguments
    /// * `key` - 64-byte key to search for
    ///
    /// # Returns
    /// Option containing the value if found
    pub fn get(&mut self, key: &[u8; 64]) -> io::Result<Option<Vec<u8>>> {
        let bucket_index = self.hash(key);
        let mut current_index = bucket_index;

        loop {
            let current_bucket = self.read_bucket(current_index)?;

            // Bucket matches key
            if current_bucket.index_indicator > 0 && current_bucket.key == *key {
                // Trim trailing zeros to get actual value
                let value = current_bucket
                    .value
                    .iter()
                    .cloned()
                    .take_while(|&x| x != 0)
                    .collect();
                return Ok(Some(value));
            }

            // End of search chain
            if current_index == bucket_index {
                return Ok(None);
            }

            current_index = (current_index + 1) % self.num_buckets;
        }
    }

    /// Delete a key-value pair
    ///
    /// # Arguments
    /// * `key` - 64-byte key to delete
    ///
    /// # Returns
    /// Option containing the deleted value if found
    pub fn delete(&mut self, key: &[u8; 64]) -> io::Result<Option<Vec<u8>>> {
        let bucket_index = self.hash(key);
        let mut current_index = bucket_index;

        loop {
            let mut current_bucket = self.read_bucket(current_index)?;

            // Bucket matches key
            if current_bucket.index_indicator > 0 && current_bucket.key == *key {
                // Extract value before clearing
                let value = current_bucket
                    .value
                    .iter()
                    .cloned()
                    .take_while(|&x| x != 0)
                    .collect();

                // Reset bucket
                current_bucket.index_indicator = 0;
                current_bucket.key = [0; 64];
                current_bucket.value = [0; 190];

                self.write_bucket_at_index(current_index, &current_bucket)?;
                return Ok(Some(value));
            }

            // End of search chain
            if current_index == bucket_index {
                return Ok(None);
            }

            current_index = (current_index + 1) % self.num_buckets;
        }
    }

    /// Simple hash function to determine bucket index
    fn hash(&self, key: &[u8; 64]) -> usize {
        // Basic hash: sum of key bytes modulo number of buckets
        key.iter().map(|&x| x as usize).sum::<usize>() % self.num_buckets
    }

    /// Read a bucket at a specific index
    fn read_bucket(&mut self, index: usize) -> io::Result<Bucket> {
        let offset = index * self.bucket_size;
        self.file.seek(SeekFrom::Start(offset as u64))?;

        let mut buffer = [0u8; std::mem::size_of::<Bucket>()];
        self.file.read_exact(&mut buffer)?;

        Ok(Bucket {
            index_indicator: u16::from_le_bytes([buffer[0], buffer[1]]),
            key: buffer[2..66].try_into().unwrap(),
            value: buffer[66..256].try_into().unwrap(),
        })
    }

    /// Write a bucket to a specific index
    fn write_bucket_at_index(&mut self, index: usize, bucket: &Bucket) -> io::Result<()> {
        let offset = index * self.bucket_size;
        self.file.seek(SeekFrom::Start(offset as u64))?;
        Self::write_bucket(&mut self.file, bucket)
    }

    /// Helper method to write a bucket to a file
    fn write_bucket(file: &mut File, bucket: &Bucket) -> io::Result<()> {
        // Convert components to bytes
        let indicator_bytes = bucket.index_indicator.to_le_bytes();

        // Combine all components
        let mut buffer = [0u8; std::mem::size_of::<Bucket>()];
        buffer[0..2].copy_from_slice(&indicator_bytes);
        buffer[2..66].copy_from_slice(&bucket.key);
        buffer[66..256].copy_from_slice(&bucket.value);

        file.write_all(&buffer)?;
        file.flush()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_file_hash_basic_operations() {
        let test_file = "test_hash.tc";

        // Cleanup any existing test file
        let _ = fs::remove_file(test_file);

        // Initialize
        let mut file_hash = FileHash::init(test_file).expect("Failed to initialize");

        // Test adding and retrieving
        let key = [42u8; 64];
        let value = b"Hello, FileHash!";
        file_hash.add(key, value).expect("Failed to add entry");

        let retrieved = file_hash.get(&key).expect("Failed to get entry");
        assert_eq!(retrieved, Some(value.to_vec()));

        // Test deleting
        let deleted = file_hash.delete(&key).expect("Failed to delete entry");
        assert_eq!(deleted, Some(value.to_vec()));

        let after_delete = file_hash.get(&key).expect("Failed to check after delete");
        assert_eq!(after_delete, None);

        // Cleanup
        let _ = fs::remove_file(test_file);
    }
}
