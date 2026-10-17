use std::path::Path;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use kvs::engines::{KvStore, KvsEngine, SledKvsEngine};
use rand::{
    RngExt, SeedableRng,
    distr::{Alphanumeric, SampleString},
    rngs::StdRng,
};
use tempfile::TempDir;

type Record = (String, String);

fn random_records(count: usize, seed: u64) -> Vec<Record> {
    let mut rng = StdRng::seed_from_u64(seed);

    (0..count)
        .map(|_| {
            let key_len = rng.random_range(1..=100_000);
            let value_len = rng.random_range(1..=100_000);

            let key = Alphanumeric.sample_string(&mut rng, key_len);
            let value = Alphanumeric.sample_string(&mut rng, value_len);

            (key, value)
        })
        .collect()
}

fn bench_write<E, F>(c: &mut Criterion, name: &str, records: &[Record], open: F)
where
    E: KvsEngine,
    F: Fn(&Path) -> E + Copy,
{
    c.bench_function(name, |b| {
        b.iter_batched(
            || {
                let dir = tempfile::tempdir().unwrap();
                let engine = open(dir.path());
                (engine, dir)
            },
            |(mut engine, dir)| {
                for (key, value) in records {
                    engine
                        .set(
                            std::hint::black_box(key.clone()),
                            std::hint::black_box(value.clone()),
                        )
                        .unwrap();
                }

                (engine, dir)
            },
            BatchSize::PerIteration,
        )
    });
}

fn bench_read<E, F>(c: &mut Criterion, name: &str, records: &[Record], open: F)
where
    E: KvsEngine,
    F: Fn(&Path) -> E,
{
    let dir: TempDir = tempfile::tempdir().unwrap();
    let mut engine = open(dir.path());

    for (key, value) in records {
        engine.set(key.clone(), value.clone()).unwrap();
    }

    c.bench_function(name, |b| {
        b.iter(|| {
            for (key, expected) in records {
                let actual = engine
                    .get(std::hint::black_box(key.clone()))
                    .unwrap()
                    .expect("benchmark key should exist");

                assert_eq!(actual.as_str(), expected.as_str());
                std::hint::black_box(actual);
            }
        })
    });

    drop(dir);
}

fn criterion_benchmark(c: &mut Criterion) {
    let writes = random_records(100, 1);
    let reads = random_records(1_000, 2);

    bench_write(c, "kvs_write", &writes, |path| KvStore::open(path).unwrap());
    bench_write(c, "sled_write", &writes, |path| {
        let sled_path = path.join("sled_db");
        SledKvsEngine::open(&sled_path).unwrap()
    });

    bench_read(c, "kvs_read", &reads, |path| KvStore::open(path).unwrap());
    bench_read(c, "sled_read", &reads, |path| {
        let sled_path = path.join("sled_db");
        SledKvsEngine::open(&sled_path).unwrap()
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
