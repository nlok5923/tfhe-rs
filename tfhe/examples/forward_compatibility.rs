// fn check_boolean() {
//     let serialized_stuff = {
//         use bincode;
//         use tfhe::boolean;

//         let params_ks_pbs = boolean::parameters::DEFAULT_PARAMETERS_KS_PBS;
//         let ser_params_ks_pbs = bincode::serialize(&params_ks_pbs);

//         let cks_ks_pbs = boolean::client_key::ClientKey::new(&params_ks_pbs);
//         let ser_cks_ks_pbs = bincode::serialize(&cks_ks_pbs);

//         let compressed_sks_ks_pbs = boolean::server_key::CompressedServerKey::new(&cks_ks_pbs);
//         let ser_compressed_sks_ks_pbs = bincode::serialize(&compressed_sks_ks_pbs).unwrap();

//         let sks_ks_pbs: boolean::server_key::ServerKey = compressed_sks_ks_pbs.my_into();
//         let ser_sks_ks_pbs = bincode::serialize(&sks_ks_pbs);

//         let mut serialized_ciphertexts = vec![];

//         for msg in [false, true] {
//             let ct = cks_ks_pbs.encrypt(msg);
//             let ser_ct = bincode::serialize(&ct).unwrap();
//             serialized_ciphertexts.push(ser_ct);

//             let compressed_ct = cks_ks_pbs.encrypt_compressed(msg);
//             let ser_ct = bincode::serialize(&compressed_ct).unwrap();
//             serialized_ciphertexts.push(ser_ct);
//         }

//         let params_pbs_ks = boolean::parameters::DEFAULT_PARAMETERS;
//         let ser_params_pbs_ks = bincode::serialize(&params_pbs_ks);

//         let cks_pbs_ks = boolean::client_key::ClientKey::new(&params_pbs_ks);
//         let ser_cks_pbs_ks = bincode::serialize(&cks_pbs_ks);
//     };
// }

pub trait MyFrom<FromType>: Sized {
    #[must_use]
    fn my_from(value: FromType) -> Self;
}

pub trait MyInto<T>: Sized {
    /// Converts this type my_into the (usually inferred) input type.
    #[must_use]
    fn my_into(self) -> T;
}

impl<IntoType: MyFrom<FromType>, FromType: Sized> MyInto<IntoType> for FromType {
    fn my_into(self) -> IntoType {
        IntoType::my_from(self)
    }
}

// Have a macro for parameters
impl MyFrom<tfhe::shortint::ciphertext::Degree> for next_tfhe::shortint::ciphertext::Degree {
    fn my_from(value: tfhe::shortint::ciphertext::Degree) -> Self {
        Self(value.0)
    }
}

impl MyFrom<tfhe::shortint::parameters::MessageModulus>
    for next_tfhe::shortint::parameters::MessageModulus
{
    fn my_from(value: tfhe::shortint::parameters::MessageModulus) -> Self {
        Self(value.0)
    }
}

impl MyFrom<tfhe::shortint::parameters::CarryModulus>
    for next_tfhe::shortint::parameters::CarryModulus
{
    fn my_from(value: tfhe::shortint::parameters::CarryModulus) -> Self {
        Self(value.0)
    }
}

impl MyFrom<tfhe::shortint::ciphertext::PBSOrder> for next_tfhe::shortint::ciphertext::PBSOrder {
    fn my_from(value: tfhe::shortint::ciphertext::PBSOrder) -> Self {
        match value {
            tfhe::shortint::PBSOrder::KeyswitchBootstrap => Self::KeyswitchBootstrap,
            tfhe::shortint::PBSOrder::BootstrapKeyswitch => Self::BootstrapKeyswitch,
        }
    }
}

impl<Scalar> MyFrom<tfhe::core_crypto::commons::ciphertext_modulus::CiphertextModulus<Scalar>>
    for next_tfhe::core_crypto::commons::ciphertext_modulus::CiphertextModulus<Scalar>
where
    Scalar: tfhe::core_crypto::commons::numeric::UnsignedInteger
        + next_tfhe::core_crypto::commons::numeric::UnsignedInteger,
{
    fn my_from(
        value: tfhe::core_crypto::commons::ciphertext_modulus::CiphertextModulus<Scalar>,
    ) -> Self {
        if value.is_native_modulus() {
            Self::new_native()
        } else {
            Self::new(value.get_custom_modulus())
        }
    }
}

impl<Scalar, C> MyFrom<tfhe::core_crypto::entities::LweCiphertext<C>>
    for next_tfhe::core_crypto::entities::LweCiphertext<C>
where
    Scalar: tfhe::core_crypto::commons::numeric::UnsignedInteger
        + next_tfhe::core_crypto::commons::numeric::UnsignedInteger,
    C: tfhe::core_crypto::commons::traits::Container<Element = Scalar>
        + next_tfhe::core_crypto::commons::traits::Container<Element = Scalar>,
{
    fn my_from(value: tfhe::core_crypto::entities::LweCiphertext<C>) -> Self {
        let ciphertext_modulus = value.ciphertext_modulus();
        let container = value.into_container();

        Self::from_container(container, ciphertext_modulus.my_into())
    }
}

fn check_shortint_forward_compat() {
    let (ser_cks, ser_ct) = {
        use next_tfhe::shortint as next_shortint;
        use tfhe::shortint;
        let (cks, _sks) = shortint::gen_keys(shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS);
        let ct = cks.encrypt(0);

        let next_ct = next_shortint::Ciphertext::new(
            ct.ct.my_into(),
            ct.degree.my_into(),
            // To be safe when operations are done on the ciphertext
            next_shortint::ciphertext::NoiseLevel::NOMINAL,
            ct.message_modulus.my_into(),
            ct.carry_modulus.my_into(),
            ct.pbs_order.my_into(),
        );

        let ser_cks = bincode::serialize(&cks).unwrap();
        let ser_ct = bincode::serialize(&next_ct).unwrap();
        (ser_cks, ser_ct)
    };

    {
        use next_tfhe::shortint;

        let _ct: shortint::Ciphertext = bincode::deserialize(&ser_ct).unwrap();
        let _cks: shortint::ClientKey = bincode::deserialize(&ser_cks).unwrap();
    }
}

fn check_shortint_broken() {
    let (ser_cks, ser_ct) = {
        use tfhe::shortint;
        let (cks, _sks) = shortint::gen_keys(shortint::parameters::PARAM_MESSAGE_2_CARRY_2_KS_PBS);
        let ser_cks = bincode::serialize(&cks).unwrap();
        let ct = cks.encrypt(0);

        let ser_ct = bincode::serialize(&ct).unwrap();
        (ser_cks, ser_ct)
    };

    {
        use next_tfhe::shortint;

        let _ct: shortint::Ciphertext = bincode::deserialize(&ser_ct).unwrap();
        let _cks: shortint::ClientKey = bincode::deserialize(&ser_cks).unwrap();
    }
}

pub fn main() {
    check_shortint_forward_compat();
    check_shortint_broken();
}
