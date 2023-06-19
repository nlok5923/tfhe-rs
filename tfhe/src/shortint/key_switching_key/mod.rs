//! This module defines KeySwitchingKey
//!
//! - [KeySwitchingKey] allows switching the keys of a ciphertext, from a cleitn key to another.

use crate::shortint::engine::ShortintEngine;
use crate::shortint::parameters::ShortintKeySwitchingParameters;
use crate::shortint::{Ciphertext, ClientKey, ServerKey};

use crate::core_crypto::prelude::{keyswitch_lwe_ciphertext, LweKeyswitchKeyOwned};

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod test;

/// A structure containing the casting public key.
///
/// The casting key is generated by the client and is meant to be published: the client
/// sends it to the server so it can cast from one set of parameters to another.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KeySwitchingKey {
    pub(crate) key_switching_key: LweKeyswitchKeyOwned<u64>,
    pub(crate) dest_server_key: ServerKey,
    pub(crate) src_server_key: ServerKey,
    pub cast_rshift: i8,
}

impl KeySwitchingKey {
    /// Generate a casting key. This can cast to several kinds of keys (shortint, integer, hlapi),
    /// depending on input.
    ///
    /// # Example
    ///
    /// ```rust
    /// use tfhe::shortint::parameters::{PARAM_MESSAGE_1_CARRY_1, PARAM_MESSAGE_2_CARRY_2};
    /// use tfhe::shortint::prelude::*;
    /// use tfhe::shortint::{gen_keys, KeySwitchingKey};
    ///
    /// // Generate the client keys and server keys:
    /// let (ck1, sk1) = gen_keys(PARAM_MESSAGE_1_CARRY_1);
    /// let (ck2, sk2) = gen_keys(PARAM_MESSAGE_2_CARRY_2);
    ///
    /// // Generate the server key:
    /// let ksk = KeySwitchingKey::new((&ck1, &sk1), (&ck2, &sk2), PARAM_KEYSWITCH_1_1_TO_2_2);
    /// ```
    pub fn new(
        key_pair_1: (&ClientKey, &ServerKey),
        key_pair_2: (&ClientKey, &ServerKey),
        params: ShortintKeySwitchingParameters,
    ) -> Self {
        // Creation of the key switching key
        let key_switching_key = ShortintEngine::with_thread_local_mut(|engine| {
            engine.new_key_switching_key(key_pair_1.0, key_pair_2.0, params)
        });

        let full_message_modulus_1 =
            key_pair_1.0.parameters.carry_modulus().0 * key_pair_1.0.parameters.message_modulus().0;
        let full_message_modulus_2 =
            key_pair_2.0.parameters.carry_modulus().0 * key_pair_2.0.parameters.message_modulus().0;
        if !full_message_modulus_1.is_power_of_two() || !full_message_modulus_2.is_power_of_two() {
            panic!("Cannot create casting key if the full messages moduli are not a power of 2");
        }

        let nb_bits_1: i8 = full_message_modulus_1.ilog2().try_into().unwrap();
        let nb_bits_2: i8 = full_message_modulus_2.ilog2().try_into().unwrap();

        // Pack the keys in the casting key set:
        Self {
            key_switching_key: key_switching_key.unwrap(),
            dest_server_key: key_pair_2.1.clone(),
            src_server_key: key_pair_1.1.clone(),
            cast_rshift: nb_bits_2 - nb_bits_1,
        }
    }

    /// Cast a ciphertext from the source parameter set to the dest parameter set,
    /// using provided &mut.
    ///
    /// # Example (the following code won't actually run because this function is private)
    ///
    /// ```rust
    /// use tfhe::shortint::parameters::{PARAM_MESSAGE_1_CARRY_1, PARAM_MESSAGE_2_CARRY_2};
    /// use tfhe::shortint::prelude::*;
    /// use tfhe::shortint::{gen_keys, KeySwitchingKey};
    ///
    /// // Generate the client keys and server keys:
    /// let (ck1, sk1) = gen_keys(PARAM_MESSAGE_1_CARRY_1);
    /// let (ck2, sk2) = gen_keys(PARAM_MESSAGE_2_CARRY_2);
    ///
    /// // Generate the server key:
    /// let ksk = KeySwitchingKey::new((&ck1, &sk1), (&ck2, &sk2), PARAM_KEYSWITCH_1_1_TO_2_2);
    ///
    /// let cipher = ck1.encrypt(1);
    /// let mut cipher_2 = sk2.create_trivial(0);
    /// ksk.cast_into(&cipher, &mut cipher_2);
    /// ```
    pub fn cast_into(&self, ct: &Ciphertext, ct_dest: &mut Ciphertext) {
        match self.cast_rshift {
            // Same bit size: only key switch
            0 => keyswitch_lwe_ciphertext(&self.key_switching_key, &ct.ct, &mut ct_dest.ct),

            // Cast to bigger bit length: keyswitch, then right shift
            i if i > 0 => {
                keyswitch_lwe_ciphertext(&self.key_switching_key, &ct.ct, &mut ct_dest.ct);

                let acc = self.dest_server_key.generate_accumulator(|n| n >> i);
                self.dest_server_key
                    .apply_lookup_table_assign(ct_dest, &acc);
            }

            // Cast to smaller bit length: left shift, then keyswitch
            i if i < 0 => {
                let acc = self.src_server_key.generate_accumulator(|n| n << -i);
                let shifted_cipher = self.src_server_key.apply_lookup_table(ct, &acc);

                keyswitch_lwe_ciphertext(
                    &self.key_switching_key,
                    &shifted_cipher.ct,
                    &mut ct_dest.ct,
                );
            }

            _ => unreachable!(),
        };
    }

    /// Cast a ciphertext from the source parameter set to the dest parameter set,
    /// returning a new ciphertext.
    ///
    /// # Example (the following code won't actually run because this function is private)
    ///
    /// ```rust
    /// use tfhe::shortint::parameters::{PARAM_MESSAGE_1_CARRY_1, PARAM_MESSAGE_2_CARRY_2};
    /// use tfhe::shortint::prelude::*;
    /// use tfhe::shortint::{gen_keys, KeySwitchingKey};
    ///
    /// // Generate the client keys and server keys:
    /// let (ck1, sk1) = gen_keys(PARAM_MESSAGE_1_CARRY_1);
    /// let (ck2, sk2) = gen_keys(PARAM_MESSAGE_2_CARRY_2);
    ///
    /// // Generate the server key:
    /// let ksk = KeySwitchingKey::new((&ck1, &sk1), (&ck2, &sk2), PARAM_KEYSWITCH_1_1_TO_2_2);
    ///
    /// let cipher = ck1.encrypt(1);
    /// let cipher_2 = ksk.cast(&cipher);
    /// ```
    pub fn cast(&self, ct: &Ciphertext) -> Ciphertext {
        let mut ret = self.dest_server_key.create_trivial(0);
        self.cast_into(ct, &mut ret);
        ret
    }
}
