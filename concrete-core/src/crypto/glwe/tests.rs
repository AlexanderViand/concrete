use crate::crypto::encoding::PlaintextList;
use crate::crypto::glwe::GlweList;
use crate::crypto::secret::GlweSecretKey;
use crate::crypto::UnsignedTorus;
use crate::math::dispersion::LogStandardDev;
use crate::math::random;
use crate::test_tools;
use crate::test_tools::assert_delta_std_dev;

fn test_glwe<T: UnsignedTorus>() {
    // random settings
    let nb_ct = test_tools::random_ciphertext_count(200);
    let dimension = test_tools::random_glwe_dimension(200);
    let polynomial_size = test_tools::random_polynomial_size(200);
    let noise_parameter = LogStandardDev::from_log_standard_dev(-20.);

    // generates a secret key
    let sk = GlweSecretKey::generate(dimension, polynomial_size);

    // generates random plaintexts
    let plaintexts =
        PlaintextList::from_tensor(random::random_uniform_tensor(nb_ct.0 * polynomial_size.0));

    // encrypts
    let mut ciphertext = GlweList::allocate(T::ZERO, polynomial_size, dimension, nb_ct);
    sk.encrypt_glwe_list(&mut ciphertext, &plaintexts, noise_parameter.clone());

    // decrypts
    let mut decryptions =
        PlaintextList::from_tensor(random::random_uniform_tensor(nb_ct.0 * polynomial_size.0));
    sk.decrypt_glwe_list(&mut decryptions, &ciphertext);

    // test
    assert_delta_std_dev(&plaintexts, &decryptions, noise_parameter);
}

#[test]
fn test_glwe_encrypt_decrypt_u32() {
    test_glwe::<u32>();
}

#[test]
fn test_glwe_encrypt_decrypt_u64() {
    test_glwe::<u64>();
}
