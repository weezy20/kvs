use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use kvs::{KvStore, SledKvsEngine, KvsEngine};
use tempfile::TempDir;

fn cold_start_get(c: &mut Criterion) {
    let temp_dir = TempDir::new().unwrap();
    c.bench_function("kvs: get key", |b: &mut Bencher<_>| {
        let mut store = KvStore::open(&temp_dir.path()).unwrap();
        store.set("key".to_string(), "some_get_val".to_string());
        b.iter(|| {
            store.get("key".to_string());
        })
    });
    c.bench_function("sled: get key", |b: &mut Bencher<_>| {
        let mut store = SledKvsEngine::open(&temp_dir.path()).unwrap();
        store.set("key".to_string(), "some_get_val".to_string());
        b.iter(|| {
            store.get("key".to_string());
        })
    });
}

// Define a criterion group `kv_benches` with the benchmarks under it
criterion_group!(kv_benches, cold_start_get);
// Run all benchmarks in a given group
criterion_main!(kv_benches);
