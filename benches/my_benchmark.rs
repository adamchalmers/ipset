use std::net::Ipv4Addr;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use ipnetwork::Ipv4Network;
use ipset::Ipset;
use rand::{thread_rng, Rng};

fn bench_contains(c: &mut Criterion) {
    let num_networks = 10;
    let mut r = rand::thread_rng();
    let mut set = Ipset::default();
    for _ in 0..num_networks {
        set.insert(&random_network());
    }

    c.bench_function("Contains", |b| {
        b.iter(|| {
            let ip = Ipv4Addr::new(r.gen(), r.gen(), r.gen(), r.gen());
            black_box(set.contains(&ip));
        });
    });
}
fn bench_insert(c: &mut Criterion) {
    let difficulties = vec![1, 10, 100];
    for d in difficulties {
        c.bench_with_input(BenchmarkId::new("Insert", d), &d, |b, d| {
            let mut set = Ipset::default();
            b.iter(|| {
                for _ in 0..*d {
                    set.insert(&random_network())
                }
            });
        });
    }
}

criterion_group!(benches, bench_contains, bench_insert);
criterion_main!(benches);

fn random_network() -> Ipv4Network {
    let mut r = thread_rng();
    let addr = Ipv4Addr::new(r.gen(), r.gen(), r.gen(), r.gen());
    let prefix = r.gen_range(0..32);
    Ipv4Network::new(addr, prefix).unwrap()
}
