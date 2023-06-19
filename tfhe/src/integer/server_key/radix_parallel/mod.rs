mod add;
mod bitwise_op;
mod comparison;
mod mul;
mod neg;
mod rotate;
mod scalar_add;
mod scalar_comparison;
mod scalar_mul;
mod scalar_rotate;
mod scalar_shift;
mod scalar_sub;
mod shift;
mod sub;

#[cfg(test)]
mod tests;

use super::ServerKey;
use crate::core_crypto::prelude::lwe_ciphertext_sub_assign;
use crate::integer::ciphertext::RadixCiphertext;

use rayon::prelude::*;

// parallelized versions
impl ServerKey {
    /// Propagate the carry of the 'index' block to the next one.
    ///
    /// # Example
    ///
    ///```rust
    /// use tfhe::integer::{gen_keys_radix, IntegerCiphertext};
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg = 7u64;
    ///
    /// let ct1 = cks.encrypt(msg);
    /// let ct2 = cks.encrypt(msg);
    ///
    /// // Compute homomorphically an addition:
    /// let mut ct_res = sks.unchecked_add(&ct1, &ct2);
    /// sks.propagate_parallelized(&mut ct_res, 0);
    ///
    /// // Decrypt one block:
    /// let res: u64 = cks.decrypt_one_block(&ct_res.blocks()[1]);
    /// assert_eq!(3, res);
    /// ```
    pub fn propagate_parallelized(&self, ctxt: &mut RadixCiphertext, index: usize) {
        let (carry, message) = rayon::join(
            || self.key.carry_extract(&ctxt.blocks[index]),
            || self.key.message_extract(&ctxt.blocks[index]),
        );
        ctxt.blocks[index] = message;

        //add the carry to the next block
        if index < ctxt.blocks.len() - 1 {
            self.key
                .unchecked_add_assign(&mut ctxt.blocks[index + 1], &carry);
        }
    }

    pub fn partial_propagate_parallelized(&self, ctxt: &mut RadixCiphertext, start_index: usize) {
        if self.is_eligible_for_parallel_carryless_add() {
            let mut carries = ctxt.blocks[start_index..]
                .par_iter_mut()
                .map(|block| {
                    let carry = self.key.carry_extract(block);
                    // Re-align the carry with where it is in the block
                    // so we can subtract it
                    let message_modulus = block.message_modulus.0 as u8;
                    let shifted_carry = self.key.unchecked_scalar_mul(&carry, message_modulus);
                    // We need the true lwe sub
                    // We know the value of ct is >= to the value of shifted carry
                    lwe_ciphertext_sub_assign(&mut block.ct, &shifted_carry.ct);

                    carry
                })
                .collect::<Vec<_>>();

            // Align ouput to input
            carries.rotate_right(1);
            // First block takes no input carry
            self.key.create_trivial_assign(&mut carries[0], 0);

            let carries = RadixCiphertext::from(carries);
            self.unchecked_add_assign(ctxt, &carries);
            self.propagate_single_carry_parallelized_low_latency(ctxt)
        } else {
            let len = ctxt.blocks.len();
            for i in start_index..len {
                self.propagate_parallelized(ctxt, i);
            }
        }
    }

    /// Propagate all the carries.
    ///
    /// # Example
    ///
    ///```rust
    /// use tfhe::integer::gen_keys_radix;
    /// use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2;
    ///
    /// // Generate the client key and the server key:
    /// let num_blocks = 4;
    /// let (cks, sks) = gen_keys_radix(PARAM_MESSAGE_2_CARRY_2, num_blocks);
    ///
    /// let msg = 10u64;
    ///
    /// let mut ct1 = cks.encrypt(msg);
    /// let mut ct2 = cks.encrypt(msg);
    ///
    /// // Compute homomorphically an addition:
    /// let mut ct_res = sks.unchecked_add(&mut ct1, &mut ct2);
    /// sks.full_propagate_parallelized(&mut ct_res);
    ///
    /// // Decrypt:
    /// let res: u64 = cks.decrypt(&ct_res);
    /// assert_eq!(msg + msg, res);
    /// ```
    pub fn full_propagate_parallelized(&self, ctxt: &mut RadixCiphertext) {
        self.partial_propagate_parallelized(ctxt, 0)
    }
}
