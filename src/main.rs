use hash::FileHash;

pub mod file;
pub mod file_new;
pub mod hash;

fn main() {
    let mut file_hash = FileHash::init().expect("Unable to read the file");

    let _ = file_hash.write("key", "value");
    let _ = file_hash.write("key2", "value2");

    match file_hash.read("key2") {
        Ok(val) => {
            println!("{}", val);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }

    let _ = file_hash.write("key2", "value3");

    match file_hash.read("key2") {
        Ok(val) => {
            println!("{}", val);
        }
        Err(e) => {
            eprintln!("{e}");
        }
    }
}
