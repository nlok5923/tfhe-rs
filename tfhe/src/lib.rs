//! Welcome to the TFHE-rs API documentation!
//!
//! TFHE-rs is a fully homomorphic encryption (FHE) library that implements Zama's variant of TFHE.

#![cfg_attr(feature = "__wasm_api", allow(dead_code))]
#![cfg_attr(feature = "nightly-avx512", feature(stdsimd, avx512_target_feature))]
#![cfg_attr(all(doc, not(doctest)), feature(doc_auto_cfg))]
#![cfg_attr(all(doc, not(doctest)), feature(doc_cfg))]
#![warn(rustdoc::broken_intra_doc_links)]

#[cfg(feature = "__c_api")]
pub mod c_api;

#[cfg(feature = "boolean")]
/// Welcome to the TFHE-rs [`boolean`](`crate::boolean`) module documentation!
///
/// # Special module attributes
/// cbindgen:ignore
pub mod boolean;

/// Welcome to the TFHE-rs [`core_crypto`](`crate::core_crypto`) module documentation!
///
/// # Special module attributes
/// cbindgen:ignore
pub mod core_crypto;

#[cfg(feature = "integer")]
/// Welcome to the TFHE-rs [`integer`](`crate::integer`) module documentation!
///
/// # Special module attributes
/// cbindgen:ignore
pub mod integer;

#[cfg(feature = "shortint")]
/// Welcome to the TFHE-rs [`shortint`](`crate::shortint`) module documentation!
///
/// # Special module attributes
/// cbindgen:ignore
pub mod shortint;

#[cfg(feature = "__wasm_api")]
/// cbindgen:ignore
mod js_on_wasm_api;
#[cfg(feature = "__wasm_api")]
pub use js_on_wasm_api::*;

// #[cfg(all(
//     doctest,
//     feature = "shortint",
//     feature = "boolean",
//     feature = "integer"
// ))]
// mod test_user_docs;

#[cfg(all(doctest))]
mod test_user_docs;

/// cbindgen:ignore
#[cfg(any(feature = "boolean", feature = "shortint", feature = "integer"))]
pub(crate) mod high_level_api;

#[cfg(any(feature = "boolean", feature = "shortint", feature = "integer"))]
pub use high_level_api::*;

/// cbindgen:ignore
#[cfg(any(test, doctest, feature = "internal-keycache"))]
pub mod keycache;

#[cfg(feature = "safe-deserialization")]
pub mod safe_deserialization;

pub mod conformance;

pub mod named;

pub fn minify_no_engine() {
    use crate::core_crypto::commons::math::random::Seed;
    use crate::core_crypto::commons::parameters::*;
    use crate::core_crypto::commons::traits::contiguous_entity_container::{
        ContiguousEntityContainer, ContiguousEntityContainerMut,
    };
    use crate::core_crypto::entities::*;
    // This is 0u128
    let ciphertext_modulus = CiphertextModulus::new_native();
    // Adapted formatting
    println!("expected ciphertext_modulus={ciphertext_modulus:?}",);
    {
        let mut bsk = SeededGgswCiphertextList::new(
            0u64,
            GlweSize(2),
            PolynomialSize(2),
            DecompositionBaseLog(23),
            DecompositionLevelCount(1),
            GgswCiphertextCount(1),
            Seed(0).into(),
            ciphertext_modulus,
        );

        // let entity_count = bsk.entity_count();
        // println!("entity_count={entity_count:?}");

        // Replace iter_mut by iter to have a working doctest when enabling the shortint feature
        for ggsw in bsk.iter_mut() {
            let ciphertext_modulus = ggsw.ciphertext_modulus();

            println!("in_loop_modulus={ciphertext_modulus:?}");

            // Equivalent to check that ciphertext_modulus == 0u128
            assert!(ciphertext_modulus.is_native_modulus());
        }
    };
}
