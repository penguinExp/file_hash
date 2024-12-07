use std::time::Instant;

use hash::HashTable;
// use table::HashTable;

pub mod hash;
pub mod table;

fn main() {
    let mut hash_table = HashTable::new();

    println!("--------------");

    // ----------------------------
    // benchmark insert operation
    // ----------------------------
    let start = Instant::now();

    for i in 0..100000 {
        let key = i.to_string();
        hash_table.set(&key, &key);
    }

    let duration = start.elapsed();

    println!("Time taken for inserting 100K items: {:?}", duration);

    // ----------------------------
    // benchmark get operation
    // ----------------------------
    let start = Instant::now();

    for i in 0..100000 {
        let key = i.to_string();
        hash_table.get(&key);
    }

    let duration = start.elapsed();

    println!("Time taken for retrieval of 100K items: {:?}", duration);

    // ----------------------------
    // benchmark del operation
    // ----------------------------
    let start = Instant::now();

    for i in 0..100000 {
        let key = i.to_string();
        hash_table.del(&key);
    }

    let duration = start.elapsed();

    println!("Time taken for deletion of 100K items: {:?}", duration);

    // usage

    println!("--------------");

    let key = String::from("hello");

    // insert an item
    hash_table.set(&key, "2");

    // get an item
    if let Some(val) = hash_table.get(&key) {
        println!("Value for key {key} is {}", val);
    } else {
        println!("Value for key {key} does not exists");
    }

    // update value for a key
    hash_table.set(&key, "4");

    // get value for updated item
    if let Some(val) = hash_table.get(&key) {
        println!("Value for key {key} is {}", val);
    } else {
        println!("Value for key {key} does not exists");
    }

    // delete item and get value
    if let Some(val) = hash_table.del(&key) {
        println!("Deleted key {key}:{}", val);
    } else {
        println!("Unable to delete Value for key {key}");
    }

    // get value for item which does not exists
    if let Some(val) = hash_table.get(&key) {
        println!("Value for key {key} is {}", val);
    } else {
        println!("Value for key {key} does not exists");
    }

    // update value for a key
    hash_table.set(&key, "5");

    // get value for updated item
    if let Some(val) = hash_table.get(&key) {
        println!("Value for key {key} is {}", val);
    } else {
        println!("Value for key {key} does not exists");
    }

    println!("--------------");

    // -----------------------------------------------

    // let mut hash_table = HashTable::<String, usize>::new();

    // // benchmark insert operation
    // let start = Instant::now();

    // for i in 0..100000 {
    //     let key = i.to_string();
    //     hash_table.insert(key, i);
    // }
    // let duration = start.elapsed();

    // println!("Time taken for inserting 100K items: {:?}", duration);

    // // benchmark get operation
    // let start = Instant::now();

    // for i in 0..100000 {
    //     let key = i.to_string();
    //     hash_table.get(&key);
    // }

    // let duration = start.elapsed();

    // println!("Time taken for retrieval of 100K items: {:?}", duration);

    // // usage

    // let key = String::from("hello");

    // // insert an item
    // hash_table.insert(key.clone(), 2);

    // // get an item
    // if let Some(val) = hash_table.get(&key) {
    //     println!("Value for key {key} is {}", val);
    // } else {
    //     println!("Value for key {key} does not exists")
    // }

    // // update value for a key
    // if let Some(val) = hash_table.get_mut(&key) {
    //     *val = 5;
    //     println!("Updated value for {key} to {}", 5);
    // }

    // // get value for updated item
    // if let Some(val) = hash_table.get(&key) {
    //     println!("Value for key {key} is {}", val);
    // } else {
    //     println!("Value for key {key} does not exists")
    // }
}
