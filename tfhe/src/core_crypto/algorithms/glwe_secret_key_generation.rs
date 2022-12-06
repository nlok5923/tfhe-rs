use crate::core_crypto::commons::generators::SecretRandomGenerator;
use crate::core_crypto::commons::math::random::{RandomGenerable, UniformBinary};
use crate::core_crypto::commons::numeric::Numeric;
use crate::core_crypto::commons::parameters::*;
use crate::core_crypto::commons::traits::*;
use crate::core_crypto::entities::*;

pub fn allocate_and_generate_new_binary_glwe_secret_key<Scalar, Gen>(
    glwe_dimension: GlweDimension,
    polynomial_size: PolynomialSize,
    generator: &mut SecretRandomGenerator<Gen>,
) -> GlweSecretKeyOwned<Scalar>
where
    Scalar: RandomGenerable<UniformBinary> + Numeric,
    Gen: ByteRandomGenerator,
{
    let mut glwe_secret_key =
        GlweSecretKeyOwned::new(Scalar::ZERO, glwe_dimension, polynomial_size);

    generate_binary_glwe_secret_key(&mut glwe_secret_key, generator);

    glwe_secret_key
}

pub fn generate_binary_glwe_secret_key<Scalar, InCont, Gen>(
    glwe_secret_key: &mut GlweSecretKey<InCont>,
    generator: &mut SecretRandomGenerator<Gen>,
) where
    Scalar: RandomGenerable<UniformBinary>,
    InCont: ContainerMut<Element = Scalar>,
    Gen: ByteRandomGenerator,
{
    generator.fill_slice_with_random_uniform_binary(glwe_secret_key.as_mut())
}
