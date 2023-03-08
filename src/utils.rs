
use chrono::{self, NaiveDate}; // for get_utc
use nanoid::nanoid;


pub fn gen_nano_id(size: usize) -> String {
    let id = nanoid!(size);
    id
}

pub fn today() -> NaiveDate {
    // Give a NaiveDate for the current local time
    let now = chrono::offset::Local::now();
    let now_str = now.to_string()[0..10].to_string();
    NaiveDate::parse_from_str(&now_str, "%Y-%m-%d").unwrap()
}

use std::hash::{Hash, Hasher};
use seahash::hash as shash;
use std::collections::hash_map::DefaultHasher;


pub fn hash_string(string: &str) -> u64 {
    let h: u64 = shash(&string.as_bytes());
    h
}

pub fn hash_str_i32(string: &str) -> i32 {
    // hashes love u64 but Postgres loves i32
    let h = shash(&string.as_bytes());
    h as i32 // see https://stackoverflow.com/questions/28273169/how-do-i-convert-between-numeric-types-safely-and-idiomatically
}


pub fn hashify<T>(obj: T) -> u64
where
    T: Hash,
{
    let mut hasher = DefaultHasher::new();
    obj.hash(&mut hasher);
    hasher.finish()
}
