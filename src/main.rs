use file::FileHash;

mod file;
mod hash;

fn main() {
    let mut file_hash = FileHash::init("hash.tc").expect("Unable to open the file");

    let key = [42u8; 64];
    let value = b"Hello, FileHash!";

    file_hash.add(key, value).expect("Failed to add entry");

    let retrieved = file_hash.get(&key).expect("Failed to get entry");

    println!("Fetched {:?}", retrieved);

    match file_hash.delete(&key) {
        Err(e) => {
            eprintln!("{e}");
        }
        Ok(val) => {
            println!("Deleted {:?}", val);
        }
    }
}
