use tfhe::core_crypto::prelude::*;
use tfhe::shortint::engine::ShortintEngine;
use tfhe::shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS;
use tfhe::shortint::server_key::{MaxDegree, ShortintCompressedBootstrappingKey};
use tfhe::shortint::{
    ClientKey as ShortintClientKey, CompressedServerKey as ShortintCompressedServerKey,
};

fn main() {
    {
        let cks = ShortintClientKey::new(PARAM_MESSAGE_2_CARRY_2_KS_PBS);
        // let compressed_sks = ShortintCompressedServerKey::new(&cks);
        let mut engine = ShortintEngine::new();
        // let compressed_sks = engine.new_compressed_server_key(&cks).unwrap();

        // Plaintext Max Value
        let max_value = cks.parameters.message_modulus().0 * cks.parameters.carry_modulus().0 - 1;

        // The maximum number of operations before we need to clean the carry buffer
        let max_degree = MaxDegree(max_value);
        // UNCOMMENT TO PRODUCE THE MISCOMPILE
        let compressed_sks = engine.new_compressed_server_key_with_max_degree(&cks, max_degree);

        // THIS BELOW IS THE SAME AS THE ABOVE FUNCTION INLINED
        let compressed_sks = {
            let bootstrapping_key = match cks.parameters.pbs_parameters().unwrap() {
                tfhe::shortint::PBSParameters::PBS(pbs_params) => {
                    let bootstrapping_key = allocate_and_generate_new_seeded_lwe_bootstrap_key(
                        &cks.small_lwe_secret_key,
                        &cks.glwe_secret_key,
                        pbs_params.pbs_base_log,
                        pbs_params.pbs_level,
                        pbs_params.glwe_modular_std_dev,
                        pbs_params.ciphertext_modulus,
                        &mut engine.seeder,
                    );

                    ShortintCompressedBootstrappingKey::Classic(bootstrapping_key)
                }
                tfhe::shortint::PBSParameters::MultiBitPBS(pbs_params) => {
                    let bootstrapping_key =
                        par_allocate_and_generate_new_seeded_lwe_multi_bit_bootstrap_key(
                            &cks.small_lwe_secret_key,
                            &cks.glwe_secret_key,
                            pbs_params.pbs_base_log,
                            pbs_params.pbs_level,
                            pbs_params.glwe_modular_std_dev,
                            pbs_params.grouping_factor,
                            pbs_params.ciphertext_modulus,
                            &mut engine.seeder,
                        );

                    ShortintCompressedBootstrappingKey::MultiBit {
                        seeded_bsk: bootstrapping_key,
                        deterministic_execution: pbs_params.deterministic_execution,
                    }
                }
            };

            // Creation of the key switching key
            let key_switching_key = allocate_and_generate_new_seeded_lwe_keyswitch_key(
                &cks.large_lwe_secret_key,
                &cks.small_lwe_secret_key,
                cks.parameters.ks_base_log(),
                cks.parameters.ks_level(),
                cks.parameters.lwe_modular_std_dev(),
                cks.parameters.ciphertext_modulus(),
                &mut engine.seeder,
            );

            // Pack the keys in the server key set:
            ShortintCompressedServerKey {
                key_switching_key,
                bootstrapping_key,
                message_modulus: cks.parameters.message_modulus(),
                carry_modulus: cks.parameters.carry_modulus(),
                max_degree,
                ciphertext_modulus: cks.parameters.ciphertext_modulus(),
                pbs_order: cks.parameters.encryption_key_choice().into(),
            }
        };
    }

    println!("MIRI run done");
}
