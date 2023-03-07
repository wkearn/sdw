use criterion::{criterion_group, criterion_main, Criterion};
use sdw::{locker::Locker, model::Channel};
use time::OffsetDateTime;

pub fn locker_open(c: &mut Criterion) {
    c.bench_function("locker_open", |b| {
        b.iter(|| {
            Locker::open("assets/HE501").unwrap();
        })
    });
}

pub fn iterator_filter(c: &mut Criterion) {
    let locker = Locker::open("assets/HE501").unwrap();

    c.bench_function("iterator_filter", |b| {
        b.iter(|| {
            locker.iter().filter(|(k, _)| k.0 == "Ping").count();
        })
    });
}

pub fn range_filter(c: &mut Criterion) {
    let locker = Locker::open("assets/HE501").unwrap();

    c.bench_function("range_filter", |b| {
        b.iter(|| {
            locker
                .tree()
                .range(
                    (
                        "Ping".to_string(),
                        OffsetDateTime::UNIX_EPOCH,
                        Channel::Port,
                    )
                        ..(
                            "Ping".to_string(),
                            OffsetDateTime::now_utc(),
                            Channel::Starboard,
                        ),
                )
                .count();
        })
    });
}

pub fn get_locker(c: &mut Criterion) {
    let locker = Locker::open("assets/HE501").unwrap();

    c.bench_function("get_locker", |b| {
        b.iter(|| {
            let (k, _) = locker
                .tree()
                .first_key_value()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Key not found",
                ))
                .unwrap();
            locker.get(k).unwrap()
        })
    });
}

criterion_group!(
    benches,
    locker_open,
    iterator_filter,
    range_filter,
    get_locker
);
criterion_main!(benches);
