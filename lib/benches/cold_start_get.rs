use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use tempfile::TempDir;

fn cold_start_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("GET");
    group.bench_function("kvs: get key", |b: &mut Bencher<_>| {
        let temp_dir = TempDir::new().unwrap();
        let mut store = KvStore::open(&temp_dir.path()).unwrap();
        store
            .set("key".to_string(), "some_get_val".to_string())
            .unwrap();

        b.iter(|| {
            let _ = store.get("key".to_string()).unwrap();
        })
    });
    group.bench_function("sled: get key", |b: &mut Bencher<_>| {
        let temp_dir = TempDir::new().unwrap();
        let mut store = SledKvsEngine::open(&temp_dir.path()).unwrap();
        store
            .set("key".to_string(), "some_get_val".to_string())
            .unwrap();

        b.iter(|| {
            let _ = store.get("key".to_string()).unwrap();
        })
    });
    group.finish();
}
fn set_many_keys(c: &mut Criterion) {

    // let mut group = c.benchmark_group("Get Group");
    // group.bench_function("kvs: get key", |b: &mut Bencher<_>| {
    //     let temp_dir = TempDir::new().unwrap();
    //     let mut store = KvStore::open(&temp_dir.path()).unwrap();
    //     store.set("key".to_string(), "some_get_val".to_string()).unwrap();
    //     b.iter(|| {
    //         let _ = store.get("key".to_string()).unwrap();
    //     })
    // });
    // group.bench_function("sled: get key", |b: &mut Bencher<_>| {
    //     let temp_dir = TempDir::new().unwrap();
    //     let mut store = SledKvsEngine::open(&temp_dir.path()).unwrap();
    //     store.set("key".to_string(), "some_get_val".to_string()).unwrap();
    //     b.iter(|| {
    //         let _ = store.get("key".to_string()).unwrap();
    //     })
    // });
    // group.finish();
}

// Define a criterion group `kv_benches` with the benchmarks under it
criterion_group!(kv_benches, cold_start_get, set_many_keys);
// Run all benchmarks in a given group
criterion_main!(kv_benches);
