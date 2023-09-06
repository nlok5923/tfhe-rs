#![allow(dead_code)]

#[path = "../utilities.rs"]
mod utilities;

use crate::utilities::{write_to_json, IntegerRepresentation, OperatorType};
use std::env;

use criterion::{criterion_group, Criterion};
use itertools::iproduct;
use rand::prelude::*;
use rand::Rng;
use std::vec::IntoIter;
use tfhe::integer::keycache::KEY_CACHE;
use tfhe::integer::{CrtCiphertext, ServerKey};
use tfhe::keycache::NamedParam;

use tfhe::integer::U256;

#[allow(unused_imports)]
use tfhe::shortint::parameters::{
    PARAM_MESSAGE_1_CARRY_1_KS_PBS, PARAM_MESSAGE_2_CARRY_2_KS_PBS, PARAM_MESSAGE_3_CARRY_3_KS_PBS,
    PARAM_MESSAGE_4_CARRY_4_KS_PBS, PARAM_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_2_KS_PBS,
};

/// An iterator that yields a succession of combinations
/// of parameters and a num_block to achieve a certain bit_size ciphertext
/// in radix decomposition
struct ParamsAndNumBlocksIter {
    params_and_bit_sizes:
        itertools::Product<IntoIter<tfhe::shortint::PBSParameters>, IntoIter<usize>>,
}

impl Default for ParamsAndNumBlocksIter {
    fn default() -> Self {
        let is_multi_bit = match env::var("__TFHE_RS_BENCH_TYPE") {
            Ok(val) => val.to_lowercase() == "multi_bit",
            Err(_) => false,
        };

        let is_fast_bench = match env::var("__TFHE_RS_FAST_BENCH") {
            Ok(val) => val.to_lowercase() == "true",
            Err(_) => false,
        };

        if is_multi_bit {
            let params = vec![PARAM_MULTI_BIT_MESSAGE_2_CARRY_2_GROUP_2_KS_PBS.into()];

            let bit_sizes = if is_fast_bench {
                vec![32]
            } else {
                vec![8, 16, 32, 40, 64]
            };

            let params_and_bit_sizes = iproduct!(params, bit_sizes);
            Self {
                params_and_bit_sizes,
            }
        } else {
            // FIXME One set of parameter is tested since we want to benchmark only quickest
            // operations.
            let params = vec![
                PARAM_MESSAGE_2_CARRY_2_KS_PBS.into(),
                // PARAM_MESSAGE_3_CARRY_3_KS_PBS.into(),
                // PARAM_MESSAGE_4_CARRY_4_KS_PBS.into(),
            ];

            let bit_sizes = if is_fast_bench {
                vec![32]
            } else {
                vec![8, 16, 32, 40, 64, 128, 256]
            };

            let params_and_bit_sizes = iproduct!(params, bit_sizes);
            Self {
                params_and_bit_sizes,
            }
        }
    }
}

impl Iterator for ParamsAndNumBlocksIter {
    type Item = (tfhe::shortint::PBSParameters, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        let (param, bit_size) = self.params_and_bit_sizes.next()?;
        let num_block =
            (bit_size as f64 / (param.message_modulus().0 as f64).log(2.0)).ceil() as usize;

        Some((param, num_block, bit_size))
    }
}

fn make_basis(message_modulus: u64) -> Vec<u64> {
    const two_pow_16: u64 = 1u64 << 16;
    const two_pow_32: u64 = 1u64 << 32;

    match message_modulus {
        2 => vec![2],
        3 => vec![2],
        n if n < 8 => vec![2, 3],
        n if n < 16 => vec![2, 5, 7],
        two_pow_16 => vec![7, 8, 9, 11, 13], // params 4_1
        _ => vec![3, 7, 13],
    }
}

/// Base function to bench a server key function that is a binary operation, input ciphertext will
/// contain only zero carries
fn bench_server_key_binary_function<F>(
    c: &mut Criterion,
    bench_name: &str,
    display_name: &str,
    binary_op: F,
) where
    F: Fn(&ServerKey, &mut CrtCiphertext, &mut CrtCiphertext),
{
    let mut bench_group = c.benchmark_group(bench_name);
    bench_group
        .sample_size(15)
        .measurement_time(std::time::Duration::from_secs(60));
    let mut rng = rand::thread_rng();

    for (param, num_block, bit_size) in ParamsAndNumBlocksIter::default() {
        let param_name = param.name();

        let bench_id = format!("{bench_name}::{param_name}::{bit_size}_bits");
        bench_group.bench_function(&bench_id, |b| {
            let (cks, sks) = KEY_CACHE.get_from_params(param);

            let encrypt_two_values = || {
                // Define CRT basis, and global modulus
                let basis = make_basis(param.message_modulus().0);
                let modulus = basis.iter().product::<u64>();

                let clear_0 = rng.gen::<u64>() % modulus;
                let ct_0 = cks.encrypt_crt(clear_0, basis.clone());

                let clear_1 = rng.gen::<u64>() % modulus;
                let ct_1 = cks.encrypt_crt(clear_1, basis);

                (ct_0, ct_1)
            };

            b.iter_batched(
                encrypt_two_values,
                |(mut ct_0, mut ct_1)| {
                    binary_op(&sks, &mut ct_0, &mut ct_1);
                },
                criterion::BatchSize::SmallInput,
            )
        });

        write_to_json::<u64, _>(
            &bench_id,
            param,
            param.name(),
            display_name,
            &OperatorType::Atomic,
            bit_size as u32,
            vec![param.message_modulus().0.ilog2(); num_block],
            IntegerRepresentation::Crt,
        );
    }

    bench_group.finish()
}

/// Base function to bench a server key function that is a unary operation, input ciphertext will
/// contain only zero carries
fn bench_server_key_unary_function<F>(
    c: &mut Criterion,
    bench_name: &str,
    display_name: &str,
    unary_fn: F,
) where
    F: Fn(&ServerKey, &mut CrtCiphertext),
{
    let mut bench_group = c.benchmark_group(bench_name);
    bench_group
        .sample_size(15)
        .measurement_time(std::time::Duration::from_secs(60));

    let mut rng = rand::thread_rng();

    for (param, num_block, bit_size) in ParamsAndNumBlocksIter::default() {
        let param_name = param.name();

        let bench_id = format!("{bench_name}::{param_name}::{bit_size}_bits");
        bench_group.bench_function(&bench_id, |b| {
            let (cks, sks) = KEY_CACHE.get_from_params(param);

            let encrypt_one_value = || {
                let basis = make_basis(param.message_modulus().0);
                let modulus = basis.iter().product::<u64>();

                let clear_0 = rng.gen::<u64>() % modulus;

                cks.encrypt_crt(clear_0, basis)
            };

            b.iter_batched(
                encrypt_one_value,
                |mut ct_0| {
                    unary_fn(&sks, &mut ct_0);
                },
                criterion::BatchSize::SmallInput,
            )
        });

        write_to_json::<u64, _>(
            &bench_id,
            param,
            param.name(),
            display_name,
            &OperatorType::Atomic,
            bit_size as u32,
            vec![param.message_modulus().0.ilog2(); num_block],
            IntegerRepresentation::Crt,
        );
    }

    bench_group.finish()
}

fn bench_server_key_binary_scalar_function<F, G>(
    c: &mut Criterion,
    bench_name: &str,
    display_name: &str,
    binary_op: F,
    rng_func: G,
) where
    F: Fn(&ServerKey, &mut CrtCiphertext, u64),
    G: Fn(&mut ThreadRng, usize) -> u64,
{
    let mut bench_group = c.benchmark_group(bench_name);
    bench_group
        .sample_size(15)
        .measurement_time(std::time::Duration::from_secs(60));
    let mut rng = rand::thread_rng();

    for (param, num_block, bit_size) in ParamsAndNumBlocksIter::default() {
        let param_name = param.name();

        let bench_id = format!("{bench_name}::{param_name}::{bit_size}_bits_scalar_{bit_size}");
        bench_group.bench_function(&bench_id, |b| {
            let (cks, sks) = KEY_CACHE.get_from_params(param);

            let encrypt_one_value = || {
                // Define CRT basis, and global modulus
                let basis = make_basis(param.message_modulus().0);
                let modulus = basis.iter().product::<u64>();

                let clear_0 = rng.gen::<u64>() % modulus;
                let ct_0 = cks.encrypt_crt(clear_0, basis.clone());

                let clear_1 = rng_func(&mut rng, bit_size) % modulus;

                (ct_0, clear_1)
            };

            b.iter_batched(
                encrypt_one_value,
                |(mut ct_0, clear_1)| {
                    binary_op(&sks, &mut ct_0, clear_1);
                },
                criterion::BatchSize::SmallInput,
            )
        });

        write_to_json::<u64, _>(
            &bench_id,
            param,
            param.name(),
            display_name,
            &OperatorType::Atomic,
            bit_size as u32,
            vec![param.message_modulus().0.ilog2(); num_block],
            IntegerRepresentation::Crt,
        );
    }

    bench_group.finish()
}

// Functions used to apply different way of selecting a scalar based on the context.
fn default_scalar(rng: &mut ThreadRng, _clear_bit_size: usize) -> u64 {
    rng.gen::<u64>()
}

macro_rules! define_server_key_bench_unary_fn (
    (method_name: $server_key_method:ident, display_name:$name:ident) => {
        fn $server_key_method(c: &mut Criterion) {
            bench_server_key_unary_function(
                c,
                concat!("integer::", stringify!($server_key_method)),
                stringify!($name),
                |server_key, lhs| {
                  server_key.$server_key_method(lhs);
            })
        }
    }
);

macro_rules! define_server_key_bench_fn (
    (method_name: $server_key_method:ident, display_name:$name:ident) => {
        fn $server_key_method(c: &mut Criterion) {
          bench_server_key_binary_function(
                c,
                concat!("integer::", stringify!($server_key_method)),
                stringify!($name),
                |server_key, lhs, rhs| {
                  server_key.$server_key_method(lhs, rhs);
            })
        }
    }
  );

macro_rules! define_server_key_bench_scalar_fn (
    (method_name: $server_key_method:ident, display_name:$name:ident, rng_func:$($rng_fn:tt)*) => {
        fn $server_key_method(c: &mut Criterion) {
            bench_server_key_binary_scalar_function(
                c,
                concat!("integer::", stringify!($server_key_method)),
                stringify!($name),
                |server_key, lhs, rhs| {
                  server_key.$server_key_method(lhs, rhs);
                },
                $($rng_fn)*
            )
        }
    }
  );

define_server_key_bench_fn!(method_name: smart_crt_add, display_name: add);
define_server_key_bench_fn!(method_name: smart_crt_sub, display_name: sub);
define_server_key_bench_fn!(method_name: smart_crt_mul, display_name: mul);

define_server_key_bench_fn!(method_name: smart_crt_add_parallelized, display_name: add);
define_server_key_bench_fn!(method_name: smart_crt_sub_parallelized, display_name: sub);
define_server_key_bench_fn!(method_name: smart_crt_mul_parallelized, display_name: mul);

define_server_key_bench_fn!(method_name: unchecked_crt_add, display_name: add);
define_server_key_bench_fn!(method_name: unchecked_crt_sub, display_name: sub);
define_server_key_bench_fn!(method_name: unchecked_crt_mul, display_name: mul);

define_server_key_bench_fn!(method_name: unchecked_crt_add_parallelized, display_name: add);
define_server_key_bench_fn!(method_name: unchecked_crt_sub_parallelized, display_name: sub);
define_server_key_bench_fn!(method_name: unchecked_crt_mul_parallelized, display_name: mul);

define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_add,
    display_name: add,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_sub,
    display_name: sub,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_mul,
    display_name: mul,
    rng_func: default_scalar
);

define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_add_parallelized,
    display_name: add,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_sub_parallelized,
    display_name: sub,
    rng_func: default_scalar,
);
define_server_key_bench_scalar_fn!(
    method_name: smart_crt_scalar_mul_parallelized,
    display_name: mul,
    rng_func: default_scalar
);

define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_add,
    display_name: add,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_sub,
    display_name: sub,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_mul,
    display_name: mul,
    rng_func: default_scalar
);

define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_add_parallelized,
    display_name: add,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_sub_parallelized,
    display_name: sub,
    rng_func: default_scalar
);
define_server_key_bench_scalar_fn!(
    method_name: unchecked_crt_scalar_mul_parallelized,
    display_name: mul,
    rng_func: default_scalar
);

define_server_key_bench_unary_fn!(method_name: smart_crt_neg, display_name: negation);
define_server_key_bench_unary_fn!(method_name: smart_crt_neg_parallelized, display_name: negation);
define_server_key_bench_unary_fn!(method_name: unchecked_crt_neg, display_name: negation);
define_server_key_bench_unary_fn!(method_name: unchecked_crt_neg_parallelized, display_name: negation);

criterion_group!(
    smart_crt_ops,
    smart_crt_neg,
    smart_crt_add,
    smart_crt_mul,
    smart_crt_sub,
);

criterion_group!(
    smart_crt_parallelized_ops,
    smart_crt_neg_parallelized,
    smart_crt_add_parallelized,
    smart_crt_mul_parallelized,
    smart_crt_sub_parallelized,
);

criterion_group!(
    smart_crt_scalar_ops,
    smart_crt_scalar_add,
    smart_crt_scalar_sub,
    smart_crt_scalar_mul,
);

criterion_group!(
    smart_crt_scalar_parallelized_ops,
    smart_crt_scalar_add_parallelized,
    smart_crt_scalar_sub_parallelized,
    smart_crt_scalar_mul_parallelized,
);

criterion_group!(
    unchecked_crt_ops,
    unchecked_crt_neg,
    unchecked_crt_add,
    unchecked_crt_sub,
    unchecked_crt_mul,
);

criterion_group!(
    unchecked_crt_parallelized_ops,
    unchecked_crt_neg_parallelized,
    unchecked_crt_add_parallelized,
    unchecked_crt_sub_parallelized,
    unchecked_crt_mul_parallelized,
);

criterion_group!(
    unchecked_crt_scalar_ops,
    unchecked_crt_scalar_add,
    unchecked_crt_scalar_sub,
    unchecked_crt_scalar_mul,
    unchecked_crt_scalar_add_parallelized,
    unchecked_crt_scalar_sub_parallelized,
    unchecked_crt_scalar_mul_parallelized,
);

fn main() {
    match env::var("__TFHE_RS_BENCH_OP_FLAVOR") {
        Ok(val) => {
            match val.to_lowercase().as_str() {
                "smart" => smart_crt_ops(),
                "smart_scalar" => smart_crt_scalar_ops(),
                "smart_parallelized" => smart_crt_parallelized_ops(),
                "smart_scalar_parallelized" => smart_crt_scalar_parallelized_ops(),
                "unchecked" => {
                    unchecked_crt_ops();
                    unchecked_crt_parallelized_ops()
                }
                "unchecked_scalar" => unchecked_crt_scalar_ops(),
                _ => panic!("unknown benchmark operations flavor"),
            };
        }
        Err(_) => {
            smart_crt_parallelized_ops();
            smart_crt_scalar_parallelized_ops()
        }
    };

    Criterion::default().configure_from_args().final_summary();
}
