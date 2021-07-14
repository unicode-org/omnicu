// This file is part of ICU4X. For terms of use, please see the file
// called LICENSE at the top level of the ICU4X source tree
// (online at: https://github.com/unicode-org/icu4x/blob/main/LICENSE ).

use rand::SeedableRng;
use rand_distr::{Distribution, Triangular};
use rand_pcg::Lcg64Xsh32;
use std::str::FromStr;

use criterion::{black_box, criterion_group, criterion_main, Criterion};

use fixed_decimal::FixedDecimal;

fn triangular_nums(range: f64) -> Vec<isize> {
    // Use Lcg64Xsh32, a small, fast PRNG.
    // Generate 1000 numbers between -range and +range, weighted around 0.
    let rng = Lcg64Xsh32::seed_from_u64(2020);
    let dist = Triangular::new(-range, range, 0.0).unwrap();
    dist.sample_iter(rng)
        .take(1000)
        .map(|v| v as isize)
        .collect()
}

fn overview_bench(c: &mut Criterion) {
    let nums = triangular_nums(1e4);
    let values: Vec<_> = nums.iter().map(|n| n.to_string()).collect();
    c.bench_function("fixed_decimal/overview", |b| {
        #[allow(clippy::suspicious_map)]
        b.iter(|| {
            // This benchmark focuses on short numbers and performs:
            // * Construction of FixedDecimals from isize
            // * Construction of FixedDecimals from strings
            // * Serialization of FixedDecimal to string
            nums.iter()
                .map(|v| black_box(*v))
                .map(FixedDecimal::from)
                .count();
            let fds: Vec<_> = values
                .iter()
                .map(black_box)
                .map(|v| FixedDecimal::from_str(&v).expect("Failed to parse"))
                .collect();
            fds.iter().map(black_box).map(|v| v.to_string()).count();
        });
    });

    #[cfg(feature = "bench")]
    {
        smaller_isize_benches(c);
        larger_isize_benches(c);
        to_string_benches(c);
        from_string_benches(c);
        from_float_ryu_benches(c);
        from_double_ryu_benches(c);
    }
}

#[cfg(feature = "bench")]
fn smaller_isize_benches(c: &mut Criterion) {
    // Smaller nums: -1e4 to 1e4
    let nums = triangular_nums(1e4);

    // Note: this could be bench_function_with_inputs, but there are 1000 random inputs.
    // Instead, consider all inputs together in the same benchmark.
    c.bench_function("isize/smaller", |b| {
        b.iter(|| {
            #[allow(clippy::suspicious_map)]
            nums.iter()
                .map(|v| black_box(*v))
                .map(FixedDecimal::from)
                .count()
        });
    });
}

#[cfg(feature = "bench")]
fn larger_isize_benches(c: &mut Criterion) {
    // Larger nums: -1e16 to 1e16
    let nums = triangular_nums(1e16);

    // Note: this could be bench_function_with_inputs, but there are 1000 random inputs.
    // Instead, consider all inputs together in the same benchmark.
    c.bench_function("isize/larger", |b| {
        b.iter(|| {
            #[allow(clippy::suspicious_map)]
            nums.iter()
                .map(|v| black_box(*v))
                .map(FixedDecimal::from)
                .count()
        });
    });
}

#[cfg(feature = "bench")]
fn to_string_benches(c: &mut Criterion) {
    use criterion::BenchmarkId;
    use writeable::Writeable;

    let objects = [
        FixedDecimal::from(2250).multiplied_pow10(-2).unwrap(),
        FixedDecimal::from(908070605040302010u128),
    ];

    {
        let mut group = c.benchmark_group("to_string/to_string");
        for object in objects.iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(object.to_string()),
                object,
                |b, object| b.iter(|| object.to_string()),
            );
        }
        group.finish();
    }

    {
        let mut group = c.benchmark_group("to_string/write_to");
        for object in objects.iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(object.to_string()),
                object,
                |b, object| {
                    b.iter(|| {
                        let mut result = String::with_capacity(object.write_len().capacity());
                        object.write_to(&mut result)
                    })
                },
            );
        }
        group.finish();
    }
}

#[cfg(feature = "bench")]
fn from_string_benches(c: &mut Criterion) {
    use criterion::BenchmarkId;

    let objects = [
        "0012.3400",
        "00.0012216734340",
        "00002342561123400.0",
        "-00123400",
        "922337203685477580898230948203840239384.9823094820384023938423424",
        "0.000000001",
        "1000000001",
        &{
            let mut x = format!("{:0fill$}", 0, fill = 32768);
            x.push('.');
            x.push_str(&format!("{:0fill$}", 0, fill = 32768));
            x
        },
    ];

    {
        let mut group = c.benchmark_group("from_string");
        for object in objects.iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(object.to_string()),
                object,
                |b, object| b.iter(|| FixedDecimal::from_str(object).unwrap()),
            );
        }
        group.finish();
    }
}

#[cfg(feature = "bench")]
fn from_float_ryu_benches(c: &mut Criterion) {
    use criterion::BenchmarkId;

    let objects = [
        12.34,
        0.001221673434,
        234256110.0,
        -123400.0,
        123.45,
        0.000000001,
        10001.0,
    ];

    {
        let mut group = c.benchmark_group("from_float_ryu");
        for object in objects.iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(object.to_string()),
                object,
                |b, object| b.iter(|| FixedDecimal::from_float_ryu(*object, 10).unwrap()),
            );
        }
        group.finish();
    }
}

#[cfg(feature = "bench")]
fn from_double_ryu_benches(c: &mut Criterion) {
    use criterion::BenchmarkId;

    let objects = [
        12.34,
        0.001221673434,
        2342561123400.0,
        -123400.0,
        1234.56789,
        0.000000001,
        1000000001.0,
    ];

    {
        let mut group = c.benchmark_group("from_double_ryu");
        for object in objects.iter() {
            group.bench_with_input(
                BenchmarkId::from_parameter(object.to_string()),
                object,
                |b, object| b.iter(|| FixedDecimal::from_double_ryu(*object, 10).unwrap()),
            );
        }
        group.finish();
    }
}

criterion_group!(benches, overview_bench,);
criterion_main!(benches);
