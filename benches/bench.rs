#![feature(test)]
#![allow(unused)]

extern crate test;
use assert_cmd::prelude::*;
use kvs::KvStore;
use std::process::Command;
use tempfile::TempDir;
use test::{bench, Bencher};

#[bench]
fn bench_kvstore_open(b: &mut Bencher) {
    let temp_dir = TempDir::new().unwrap();
    b.iter(|| {
        let mut store = test::black_box(KvStore::open(&temp_dir.path()).unwrap());
    })
}

#[bench]
fn bench_kvstore_set(b: &mut Bencher) {
    let temp_dir = TempDir::new().unwrap();
    let mut store = KvStore::open(&temp_dir.path()).unwrap();
    b.iter(|| {
        let key = test::black_box("key".to_string());
        let value = test::black_box("value".to_string());
        store.set(key, value);
    })
}

#[bench]
fn bench_kvstore_get(b: &mut Bencher) {
    let temp_dir = TempDir::new().unwrap();
    let mut store = KvStore::open(&temp_dir.path()).unwrap();
    store.set("key".to_string(), "some_get_val".to_string());
    b.iter(|| {
        store.get("key".to_string());
    })
}


#[bench]
fn bench_kvstore_remove(b: &mut Bencher) {
    let temp_dir = TempDir::new().unwrap();
    let mut store = KvStore::open(&temp_dir.path()).unwrap();
    store.set("key".to_string(), "some_get_val".to_string());
    b.iter(|| {
        store.remove("key".to_string());
    })
}
