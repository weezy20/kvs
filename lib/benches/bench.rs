use criterion::{black_box, criterion_group, criterion_main, Bencher, Criterion};
use kvs::{KvStore, KvsEngine, SledKvsEngine};
use rand::Rng;
use tempfile::TempDir;

fn cold_start_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("GET");

    group.bench_function("kvs: get key", |b: &mut Bencher<_>| {
        // Setup for KVS
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
        // Setup for SLED
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
    let mut group = c.benchmark_group("SET & RM");
    let temp_dir = TempDir::new().unwrap();
    let mut store = KvStore::open(&temp_dir.path()).unwrap();
    let test_data: Vec<(String, String)> = generate_test_data();
    group.bench_function("kvs: SET", |b: &mut Bencher<_>| {
        b.iter(|| {
            black_box(for (k, v) in test_data.clone().into_iter() {
                store.set(k, v).unwrap();
            })
        })
    });
    group.bench_function("kvs: REMOVE", |b| {
        b.iter(|| {
            black_box(for (k, _) in test_data.clone().into_iter() {
                store.remove(k).unwrap();
            })
        })
    });
    let temp_dir = TempDir::new().unwrap();
    let mut store = SledKvsEngine::open(&temp_dir.path()).unwrap();
    let test_data: Vec<(String, String)> = generate_test_data();
    group.bench_function("sled: SET", |b: &mut Bencher<_>| {
        b.iter(|| {
            black_box(for (k, v) in test_data.clone().into_iter() {
                store.set(k, v).unwrap();
            })
        })
    });
    group.bench_function("sled: REMOVE", |b| {
        b.iter(|| {
            black_box(for (k, _) in test_data.clone().into_iter() {
                store.remove(k).unwrap();
            })
        })
    });
    group.finish();
}

// Define a criterion group `kv_benches` with the benchmarks under it
criterion_group!(kv_benches, cold_start_get, set_many_keys);
// Run all benchmarks in a given group
criterion_main!(kv_benches);

fn generate_random_string(length: usize) -> String {
    let mut rng = rand::thread_rng();
    let bytes = (0..length)
        .map(|_| rng.sample(rand::distributions::Alphanumeric))
        .collect::<Vec<u8>>();

    String::from_utf8(bytes).unwrap()
}

fn generate_test_data() -> Vec<(String, String)> {
    let mut data: Vec<(String, String)> = vec![];
    const MIN_LENGTH: usize = 1;
    const MAX_LENGTH: usize = 100_000;
    let mut rng = rand::thread_rng();

    for item in &mut data {
        let (k_len, v_len) = (
            rng.gen_range(MIN_LENGTH..=MAX_LENGTH),
            rng.gen_range(MIN_LENGTH..=MAX_LENGTH),
        );
        item.0 = generate_random_string(k_len); // Adjust the length as needed
        item.1 = generate_random_string(v_len); // Adjust the length as needed
    }

    data
}
